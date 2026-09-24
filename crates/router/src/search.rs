//! The query string of a location, mirroring React Router's `useSearchParams`.

use gpui::SharedString;

/// The query string of the current location, parsed into key/value pairs.
///
/// Pairs keep their order and duplicates, like `URLSearchParams`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchParams {
  entries: Vec<(SharedString, SharedString)>,
}

impl SearchParams {
  /// Parses a query string, with or without a leading `?`.
  pub(crate) fn parse(search: &str) -> Self {
    let search = search.strip_prefix('?').unwrap_or(search);
    if search.is_empty() {
      return Self::default();
    }

    let entries = search
      .split('&')
      .filter(|pair| !pair.is_empty())
      .map(|pair| match pair.split_once('=') {
        Some((name, value)) => (decode(name).into(), decode(value).into()),
        None => (decode(pair).into(), SharedString::default()),
      })
      .collect();

    Self { entries }
  }

  /// The first value of `name`, like `URLSearchParams.get`.
  pub fn get(&self, name: &str) -> Option<&str> {
    self
      .entries
      .iter()
      .find(|(key, _)| key.as_ref() == name)
      .map(|(_, value)| value.as_ref())
  }

  /// Every value of `name`, like `URLSearchParams.getAll`.
  pub fn get_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    self
      .entries
      .iter()
      .filter(move |(key, _)| key.as_ref() == name)
      .map(|(_, value)| value.as_ref())
  }

  /// Whether `name` is present at all, like `URLSearchParams.has`.
  pub fn has(&self, name: &str) -> bool {
    self.get(name).is_some()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Every pair, in order.
  pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
    self.entries.iter().map(|(key, value)| (key.as_ref(), value.as_ref()))
  }

  /// Replaces every value of `name` with `value`, like `URLSearchParams.set`.
  pub fn set(mut self, name: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
    let name = name.into();
    let value = value.into();
    self.entries.retain(|(key, _)| key != &name);
    self.entries.push((name, value));
    self
  }

  /// Adds another value for `name`, like `URLSearchParams.append`.
  pub fn append(mut self, name: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
    self.entries.push((name.into(), value.into()));
    self
  }

  /// Removes every value of `name`, like `URLSearchParams.delete`.
  pub fn delete(mut self, name: &str) -> Self {
    self.entries.retain(|(key, _)| key.as_ref() != name);
    self
  }

  /// The query string with its leading `?`, or an empty string when there is
  /// nothing to encode.
  pub fn to_search(&self) -> SharedString {
    if self.entries.is_empty() {
      return SharedString::default();
    }

    let mut search = String::from("?");
    for (index, (name, value)) in self.entries.iter().enumerate() {
      if index > 0 {
        search.push('&');
      }
      search.push_str(&encode(name));
      search.push('=');
      search.push_str(&encode(value));
    }

    SharedString::from(search)
  }
}

impl std::fmt::Display for SearchParams {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(&self.to_search())
  }
}

/// Percent-encodes everything outside the unreserved set. Spaces become `%20`,
/// which every URL parser accepts.
fn encode(value: &str) -> String {
  let mut encoded = String::with_capacity(value.len());

  for byte in value.as_bytes() {
    match byte {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
        encoded.push(*byte as char);
      }
      _ => encoded.push_str(&format!("%{byte:02X}")),
    }
  }

  encoded
}

/// Decodes `%XX` escapes. Invalid escapes are kept as written, so a malformed
/// query string still round-trips instead of panicking.
fn decode(value: &str) -> String {
  let bytes = value.as_bytes();
  let mut decoded = Vec::with_capacity(bytes.len());
  let mut index = 0;

  while index < bytes.len() {
    if bytes[index] == b'%' && index + 2 < bytes.len() {
      let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or_default();
      if let Ok(byte) = u8::from_str_radix(hex, 16) {
        decoded.push(byte);
        index += 3;
        continue;
      }
    }

    decoded.push(bytes[index]);
    index += 1;
  }

  String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
  use super::SearchParams;

  #[test]
  fn test_parse_keeps_order_and_duplicates() {
    let params = SearchParams::parse("?tag=rust&tag=gpui&page=2");

    assert_eq!(params.len(), 3);
    assert_eq!(params.get("tag"), Some("rust"));
    assert_eq!(params.get_all("tag").collect::<Vec<_>>(), ["rust", "gpui"]);
    assert!(params.has("page"));
    assert!(!params.has("missing"));
    assert_eq!(
      params.iter().collect::<Vec<_>>(),
      [("tag", "rust"), ("tag", "gpui"), ("page", "2")]
    );
  }

  #[test]
  fn test_parse_handles_empty_and_nameless_pairs() {
    assert!(SearchParams::parse("").is_empty());
    assert!(SearchParams::parse("?").is_empty());

    let params = SearchParams::parse("?flag&next=1");
    assert_eq!(params.get("flag"), Some(""));
    assert_eq!(params.get("next"), Some("1"));
  }

  #[test]
  fn test_set_append_and_delete() {
    let params = SearchParams::parse("?tag=rust&tag=gpui")
      .set("tag", "router")
      .append("page", "2")
      .delete("missing");

    assert_eq!(params.get_all("tag").collect::<Vec<_>>(), ["router"]);
    assert_eq!(params.get("page"), Some("2"));
    assert_eq!(params.to_search(), "?tag=router&page=2");
  }

  #[test]
  fn test_encoding_round_trips() {
    let params = SearchParams::default()
      .append("q", "gpui router")
      .append("path", "src/lib.rs")
      .append("unicode", "日本語")
      .append("ampersand", "a&b=c");

    let search = params.to_search();
    assert_eq!(
      search.as_ref(),
      "?q=gpui%20router&path=src%2Flib.rs&unicode=%E6%97%A5%E6%9C%AC%E8%AA%9E&ampersand=a%26b%3Dc"
    );
    assert_eq!(SearchParams::parse(&search), params);
  }

  #[test]
  fn test_invalid_escapes_are_kept() {
    let params = SearchParams::parse("?q=%zz&ok=%41");

    assert_eq!(params.get("q"), Some("%zz"));
    assert_eq!(params.get("ok"), Some("A"));
  }
}
