use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Fields, Ident, Type};

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
    let mut params = Vec::new();

    if let Fields::Named(_) = data.fields {
        for field in data.fields {
            if !field.attrs.iter().any(|attr| {
                let Some(ident) = attr.path().get_ident() else {
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
            params.push(quote! {
                crate::operand::OperandEncodable::to_bytes(&self.#name, &mut params);
            });
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

            fn parameters(&self) -> Vec<u8> {
                let mut params = Vec::new();

                #(#params)*

                params
            }
        }
    }
    .into()
}

fn bits_u8_encodable(ident: Ident) -> TokenStream {
    quote! {
        impl crate::operand::OperandEncodable for #ident {
            fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                let prim: u8 = self.bits();
                <u8 as crate::operand::OperandEncodable>::to_bytes(&prim, buf);
            }

            fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self, ()> {
                if bytes.len() < offset + 1 {
                    Err(())
                } else {
                    Ok(#ident::from_bits_retain(bytes[0]))
                }
            }

            fn len(&self) -> usize {
                1
            }
        }
    }
    .into()
}

fn into_u8_encodable(ident: Ident) -> TokenStream {
    quote! {
        impl crate::operand::OperandEncodable for #ident {
            fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                let prim = <Self as Into<u8>>::into(*self);
                <u8 as crate::operand::OperandEncodable>::to_bytes(&prim, buf);
            }

            fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self, ()> {
                if bytes.is_empty() {
                    Err(())
                } else {
                    Ok(#ident::try_from(bytes[offset]).map_err(|_| ())?)
                }
            }

            fn len(&self) -> usize {
                1
            }
        }
    }
    .into()
}

#[proc_macro_derive(Operand)]
pub fn operand(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Enum(_) => into_u8_encodable(ident),
        Data::Struct(data) => match data.fields {
            Fields::Named(_) => {
                let mut to = Vec::new();
                let mut from = Vec::new();
                let mut len = Vec::new();
                let mut fields = Vec::new();
                for field in data.fields {
                    let Some(name) = field.ident else {
                        todo!("No name");
                    };
                    to.push(quote! {
                        self.#name.to_bytes(buf);
                    });
                    let typename = field.ty;
                    match typename {
                        Type::Path(_) => from.push(quote! {
                            let #name = <#typename as OperandEncodable>::from_bytes(bytes, offset)?;
                            offset += ::core::mem::size_of::<#typename>();
                        }),
                        Type::Array(_) => (),
                        _ => todo!(),
                    }
                    fields.push(name);
                    len.push(quote!(::core::mem::size_of::<#typename>()));
                }
                let q = quote! {
                    impl OperandEncodable for #ident {
                        fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                            #(#to)*
                        }

                        fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self, ()> {
                            let mut offset = offset;
                            #(#from)*
                            Ok(Self {
                                #(#fields),*
                            })
                        }

                        fn len(&self) -> usize {
                            #(#len)+*
                        }
                    }
                };
                q.into()
            }
            Fields::Unnamed(_) => bits_u8_encodable(ident),
            Fields::Unit => todo!(),
        },
        _ => todo!(),
    }
}
