mod derive_into_layout;

use proc_macro::TokenStream;

/// #[derive(IntoLayout)] is used to create a Layout with outlet.
#[proc_macro_derive(IntoLayout)]
pub fn derive_into_layout(input: TokenStream) -> TokenStream {
  derive_into_layout::derive_into_layout(input)
}
