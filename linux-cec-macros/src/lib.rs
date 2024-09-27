use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Expr, Fields, Ident, Type};

#[proc_macro_derive(Message, attributes(parameter))]
pub fn message(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data: Data::Struct(data),
        ..
    } = parse_macro_input!(input as DeriveInput)
    else {
        return quote! {
            compile_error!("This macro only works on structs");
        }
        .into();
    };

    let mut sizes = vec![quote!(1)];
    let mut declarations = Vec::new();

    if let Fields::Named(ref fields) = data.fields {
        for field in data.fields {
            if !field.attrs.iter().any(|attr| {
                let Some(ref ident) = attr.path().get_ident() else {
                    return false;
                };
                *ident == "parameter"
            }) {
                continue;
            }
            let Some(name) = field.ident else {
                todo!("No name");
            };
            let typename = field.ty;
            let getter: Ident = parse_str(format!("{name}").as_str()).unwrap();
            let setter: Ident = parse_str(format!("set_{name}").as_str()).unwrap();

            declarations.push(quote! {
                fn #getter(&self) -> #typename {
                    self.#name
                }

                fn #setter(&mut self, value: #typename) {
                    self.#name = value;
                }
            });

            sizes.push(quote!(+ ::core::mem::size_of::<#typename>()));
        }
    }

    quote! {
        impl #ident {
            #(#declarations)*
        }

        impl MessageEncodable for #ident {
            const OPCODE: Opcode = Opcode::#ident;

            fn len(&self) -> usize {
                #(#sizes)*
            }
        }
    }
    .into()
}
