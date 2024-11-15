use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, Data, DeriveInput, Expr, Field, Fields, Ident, Meta, Type,
};

macro_rules! bail {
    ($text:literal $(, $args:ident)*) => {
        return quote! {
            compile_error!($text $(, #$args)*);
        }
        .into()
    };
}

fn message(
    ident: Ident,
    fields: Fields,
    from_bytes: &mut Vec<TokenStream2>,
    to_bytes: &mut Vec<TokenStream2>,
    len: &mut Vec<TokenStream2>,
    tests: &mut Vec<TokenStream2>,
) -> Result<(), String> {
    let mut from_params = Vec::new();
    let mut names = Vec::new();

    let testname: Ident =
        parse_str(format!("test_{}", ident.to_string().to_lowercase()).as_str()).unwrap();

    match fields {
        Fields::Named(_) => {
            let mut sizes = Vec::new();
            let mut params = Vec::new();
            for field in fields {
                let Some(name) = field.ident else {
                    return Err(format!("Variant {ident} cannot have unnamed fields"));
                };
                let typename = field.ty;

                match typename {
                    Type::Path(ref path) if path.path.get_ident().is_none() => {
                        sizes.push(quote!(#name.len()))
                    }
                    _ => sizes.push(quote!(::core::mem::size_of::<#typename>())),
                }

                params.push(quote! {
                    crate::operand::OperandEncodable::to_bytes(#name, &mut out_params);
                });
                from_params.push(quote! {
                    let #name = <#typename as OperandEncodable>::try_from_bytes(bytes, offset)
                    .map_err(crate::Error::add_offset(offset))?;

                    let offset = offset + #name.len();
                });

                names.push(name);
            }

            to_bytes.push(quote! {
                Message::#ident { #(#names,)* } => {
                    let mut out_params = vec![Opcode::#ident as u8];

                    #(#params)*

                    out_params
                }
            });
            len.push(quote! {
                Message::#ident { #(#names,)* } => {
                    #(let _ = #names;)*
                    1#( + #sizes)*
                }
            });
        }
        Fields::Unnamed(_) => {
            return Err(format!("Variant {ident} cannot have unnamed fields"));
        }
        Fields::Unit => {
            to_bytes.push(quote!(Message::#ident => vec![Opcode::#ident as u8]));
            len.push(quote!(Message::#ident => 1));

            tests.push(quote! {
                #[cfg(test)]
                mod #testname {
                    use crate::Error;
                    use super::*;

                    #[test]
                    fn test_len() {
                        assert_eq!(Message::#ident {}.len(), 1);
                    }

                    #[test]
                    fn test_opcode() {
                        assert_eq!(
                            Message::#ident {}.opcode(),
                            Opcode::#ident
                        );
                    }

                    #[test]
                    fn test_encoding() {
                        assert_eq!(
                            &Message::#ident {}.to_bytes(),
                            &[Opcode::#ident as u8]
                        );
                    }

                    #[test]
                    fn test_decoding() {
                        assert_eq!(
                            Message::try_from_bytes(&[Opcode::#ident as u8]),
                            Ok(Message::#ident {})
                        );
                        assert_eq!(
                            Message::try_from_bytes(&[Opcode::#ident as u8, 0x12]),
                            Ok(Message::#ident {})
                        );
                    }
                }
            });
        }
    }

    from_bytes.push(quote! {
        Opcode::#ident => {
            let offset = 1;

            #(#from_params)*

            Message::#ident {
                #(#names),*
            }
        }
    });
    Ok(())
}

fn bits_u8_encodable(ident: Ident) -> TokenStream {
    quote! {
        impl crate::operand::OperandEncodable for #ident {
            fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                let prim: u8 = self.bits();
                <u8 as crate::operand::OperandEncodable>::to_bytes(&prim, buf);
            }

            fn try_from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                if bytes.len() < offset + 1 {
                    Err(crate::Error::OutOfRange {
                        expected: crate::Range::AtLeast(1),
                        got: bytes.len() - offset,
                        quantity: String::from("bytes"),
                    })
                } else {
                    Ok(#ident::from_bits_retain(bytes[offset]))
                }
            }

            fn len(&self) -> usize {
                1
            }
        }
    }
    .into()
}

fn try_into_u8_encodable(ident: Ident) -> TokenStream {
    quote! {
        impl crate::operand::OperandEncodable for #ident {
            fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                let prim = <Self as Into<u8>>::into(*self);
                <u8 as crate::operand::OperandEncodable>::to_bytes(&prim, buf);
            }

            fn try_from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                if bytes.len() < offset + 1 {
                    Err(crate::Error::OutOfRange {
                        expected: crate::Range::AtLeast(1),
                        got: bytes.len() - offset,
                        quantity: String::from("bytes"),
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

fn into_u8_encodable(ident: Ident) -> TokenStream {
    quote! {
        impl crate::operand::OperandEncodable for #ident {
            fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                let prim = <Self as Into<u8>>::into(*self);
                <u8 as crate::operand::OperandEncodable>::to_bytes(&prim, buf);
            }

            fn try_from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                if bytes.is_empty() {
                    Err(crate::Error::OutOfRange {
                        expected: crate::Range::AtLeast(1),
                        got: bytes.len() - offset,
                        quantity: String::from("bytes"),
                    })
                } else {
                    Ok(#ident::from(bytes[offset]))
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
        Data::Enum(_) => try_into_u8_encodable(ident),
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
                            let #name = <#typename as OperandEncodable>::try_from_bytes(bytes, struct_offset + offset)
                            .map_err(crate::Error::add_offset(struct_offset))?;

                            let struct_offset = struct_offset + #name.len();
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

                        fn try_from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                            let mut struct_offset = 0;
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
            Fields::Unnamed(data) => {
                if let Some(Field {
                    ty: Type::Path(ty), ..
                }) = data.unnamed.first()
                {
                    if ty.qself.is_some() {
                        bits_u8_encodable(ident)
                    } else {
                        into_u8_encodable(ident)
                    }
                } else {
                    todo!();
                }
            }
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
        bail!("This macro only works on the Message enum");
    };
    let mut opcodes = Vec::new();
    let mut from_bytes = Vec::new();
    let mut to_bytes = Vec::new();
    let mut len = Vec::new();
    let mut tests = Vec::new();

    for variant in data.variants {
        let ident = variant.ident;
        let Some((_, discriminant)) = variant.discriminant else {
            bail!("Variant {} missing discriminant", ident);
        };
        opcodes.push(quote!(#ident = #discriminant));

        if let Err(error) = message(
            ident,
            variant.fields,
            &mut from_bytes,
            &mut to_bytes,
            &mut len,
            &mut tests,
        ) {
            bail!("{}", error);
        }
    }

    quote! {
        #[derive(
            Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand,
        )]
        #[repr(u8)]
        pub enum Opcode {
            #(#opcodes,)*
        }

        impl Message {
            pub fn try_from_bytes(bytes: &[u8]) -> Result<Message> {
                if bytes.is_empty() {
                    return Err(crate::Error::OutOfRange {
                        expected: crate::Range::AtLeast(1),
                        got: 0,
                        quantity: String::from("bytes"),
                    })
                }
                Ok(match Opcode::try_from_primitive(bytes[0])? {
                    #(#from_bytes)*
                })
            }

            pub fn to_bytes(&self) -> Vec<u8> {
                match self {
                    #(#to_bytes,)*
                }
            }

            pub fn len(&self) -> usize {
                match self {
                    #(#len,)*
                }
            }
        }

        #(#tests)*
    }
    .into()
}

#[proc_macro_derive(BitfieldSpecifier, attributes(bits, default))]
pub fn bitfield_specifier(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        data: Data::Enum(data),
        ..
    } = parse_macro_input!(input as DeriveInput)
    else {
        bail!("This macro only works on enums");
    };

    let mut ty: Option<Type> = None;
    let mut bits: Option<Expr> = None;
    let mut into_patterns = Vec::new();
    let mut from_patterns = Vec::new();
    let mut default = None;

    // Scan enum attrs for #[repr(..)] and #[bits = ..]
    // Reject invalid repr attributes and ignore all else
    for attr in attrs {
        match attr.meta {
            Meta::List(list) => {
                match list.path.get_ident() {
                    Some(ident) if ident == "repr" => (),
                    _ => continue,
                }
                match list.parse_args() {
                    Ok(parsed_ty) => ty = Some(parsed_ty),
                    Err(e) => {
                        let e = e.to_string();
                        bail!("Invalid repr: {}", e);
                    }
                }
            }
            Meta::NameValue(nv) => {
                match nv.path.get_ident() {
                    Some(ident) if ident == "bits" => (),
                    _ => continue,
                }
                bits = Some(nv.value);
            }
            _ => continue,
        }
    }
    let Some(ty) = ty else {
        bail!("Type repr is required");
    };
    let Some(bits) = bits else {
        bail!("Bits attribute is required");
    };

    for variant in &data.variants {
        let var_ident = &variant.ident;
        match &variant.fields {
            Fields::Unit => (),
            Fields::Unnamed(fields) => {
                for attr in &variant.attrs {
                    let Meta::Path(ref path) = attr.meta else {
                        continue;
                    };
                    match fields.unnamed.first() {
                        Some(field) if ty == field.ty => (),
                        Some(_) => bail!("Default must have type matching repr"),
                        _ => continue,
                    }
                    match path.get_ident() {
                        Some(attr_ident) if attr_ident == "default" => default = Some(var_ident),
                        _ => (),
                    }
                }
                if fields.unnamed.len() != 1 || default.is_none() {
                    bail!("Variant contains fields, which is unsupported");
                }
                continue;
            }
            _ => bail!("Variant contains fields, which is unsupported"),
        };
        let Some((_, ref expr)) = variant.discriminant else {
            bail!("Variant has no explicit value");
        };
        into_patterns.push(quote!(#ident::#var_ident => #expr));
        match expr {
            Expr::Path(_) => from_patterns.push(quote!(#expr => #ident::#var_ident)),
            _ => from_patterns.push(quote!(x if x == #expr => #ident::#var_ident)),
        }
    }
    quote! {
        impl #ident {
            pub const fn into_bits(self) -> #ty {
                match self {
                    #(#into_patterns,)*
                    #ident::#default(x) => x,
                }
            }

            pub const fn from_bits(bits: #ty) -> #ident {
                match bits & ((1 << (#bits)) - 1) {
                    #(#from_patterns,)*
                    x => #ident::#default(x),
                }
            }
        }
    }
    .into()
}
