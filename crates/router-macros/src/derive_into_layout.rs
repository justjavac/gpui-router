use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives the `Layout` trait for a struct with a field named `outlet`.
pub fn derive_into_layout(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;

  // Only accept structs with named fields and require a field named `outlet`
  let outlet_field_exists = match &ast.data {
    Data::Struct(data_struct) => match &data_struct.fields {
      Fields::Named(fields_named) => fields_named
        .named
        .iter()
        .any(|f| f.ident.as_ref().map(|ident| ident == "outlet").unwrap_or(false)),
      _ => false,
    },
    _ => false,
  };

  if !outlet_field_exists {
    return syn::Error::new_spanned(&ast, "struct must have a field named `outlet`")
      .to_compile_error()
      .into();
  }

  let tokens = quote! {
      impl gpui_router::Layout for #name {
          fn outlet(&mut self, element: gpui::AnyElement) {
              self.outlet = element.into();
          }

          fn render_layout(self: Box<Self>, window: &mut gpui::Window, cx: &mut gpui::App) -> gpui::AnyElement {
              // Delegate to the render method of the struct
              self.render(window, cx).into_any_element()
          }
      }
  };

  tokens.into()
}
