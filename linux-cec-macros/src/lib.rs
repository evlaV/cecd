use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Message, attributes(parameter))]
pub fn message(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input as DeriveInput);
    quote! {
        impl MessageEncodable for #ident {
            const OPCODE: Opcode = Opcode::#ident;
        }
    }
    .into()
}
