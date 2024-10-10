use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(Message)]
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
    let mut from_params = Vec::new();
    let mut names = Vec::new();
    let mut tests = None;

    let testname: Ident =
        parse_str(format!("test_{}", ident.to_string().to_lowercase()).as_str()).unwrap();

    match data.fields {
        Fields::Named(_) => {
            for field in data.fields {
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
                from_params.push(quote! {
                    let #name = <#typename as OperandEncodable>::from_bytes(bytes, offset)?;
                    let offset = offset + #name.len();
                });

                names.push(name);
            }
        }
        Fields::Unnamed(_) => (),
        Fields::Unit => {
            tests = Some(quote! {
                #[cfg(test)]
                mod #testname {
                    use crate::Error;
                    use super::*;

                    #[test]
                    fn test_len() {
                        assert_eq!(#ident {}.len(), 1);
                    }

                    #[test]
                    fn test_encoding() {
                        assert_eq!(
                            &#ident {}.to_bytes(),
                            &[Opcode::#ident as u8]
                        );
                    }

                    #[test]
                    fn test_decoding() {
                        assert_eq!(
                            Message::try_from_bytes(&[Opcode::#ident as u8]),
                            Ok(Message::#ident(#ident {}))
                        );
                        assert_eq!(
                            Message::try_from_bytes(&[Opcode::#ident as u8, 0x12]),
                            Ok(Message::#ident(#ident {}))
                        );
                    }
                }
            });
        }
    }

    quote! {
        impl #ident {
            #(#declarations)*
        }

        impl MessageEncodable for #ident {
            const OPCODE: Opcode = Opcode::#ident;

            fn to_message(&self) -> Message {
                Message::#ident(*self)
            }

            fn into_message(self) -> Message {
                Message::#ident(self)
            }

            fn parameters(&self) -> Vec<u8> {
                let mut params = Vec::new();

                #(#params)*

                params
            }

            fn try_from_parameters(bytes: &[u8]) -> Result<#ident> {
                let offset = 0;

                #(#from_params)*

                Ok(#ident {
                    #(#names),*
                })
            }

            fn len(&self) -> usize {
                #(#sizes)*
            }
        }

        #tests
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

            fn from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                if bytes.len() < offset + 1 {
                    Err(crate::Error::InsufficientLength {
                        required: 1,
                        got: bytes.len() - offset,
                    })
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

            fn from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                if bytes.is_empty() {
                    Err(crate::Error::InsufficientLength {
                        required: 1,
                        got: bytes.len() - offset,
                    })
                } else {
                    Ok(#ident::try_from(bytes[offset])?)
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

                        fn from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
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

#[proc_macro_derive(MessageEnum)]
pub fn message_enum(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data: Data::Enum(data),
        ..
    } = parse_macro_input!(input as DeriveInput)
    else {
        return quote! {
            compile_error!("This macro only works on the Opcode enum");
        }
        .into();
    };
    let mut fields = Vec::new();
    let mut from_bytes = Vec::new();
    let mut idents = Vec::new();
    for variant in data.variants {
        match (variant.fields, variant.discriminant) {
            (Fields::Unit, Some(_)) => (),
            _ => {
                return quote! {
                    compile_error!("This macro only works on the Opcode enum");
                }
                .into()
            }
        };

        let ident = variant.ident;
        fields.push(quote!(#ident(#ident)));
        from_bytes.push(quote! {
            Opcode::#ident => Message::#ident(#ident::try_from_parameters(&bytes[1..])?),
        });
        idents.push(ident);
    }
    quote! {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum Message {
            #(#fields),*
        }

        impl Message {
            pub fn try_from_bytes(bytes: &[u8]) -> Result<Message> {
                if bytes.is_empty() {
                    return Err(crate::Error::InsufficientLength {
                        required: 1,
                        got: 0,
                    })
                }
                Ok(match Opcode::try_from_primitive(bytes[0])? {
                    #(#from_bytes)*
                })
            }

            pub fn to_bytes(&self) -> Vec<u8> {
                match self {
                    #(Message::#idents(message) => message.to_bytes(),)*
                }
            }
        }
    }
    .into()
}
