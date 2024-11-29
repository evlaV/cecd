use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, Data, DataEnum, DeriveInput, Expr, Field, Fields, Ident, Meta,
    Type, TypeArray,
};

macro_rules! bail {
    ($text:literal) => {
        return quote! {
            compile_error!($text);
        }
        .into()
    };
    ($text:literal $(, $args:ident)*) => {{
        let err = format!($text $(, $args)*);
        return quote! {
            compile_error!(#err);
        }
        .into()
    }};
    ($text:expr) => {{
        let err = $text;
        return quote! {
            compile_error!(#err);
        }
        .into()
    }};
}

struct MessageEnum {
    message: Ident,
    opcode: Ident,
    from_bytes: Vec<TokenStream2>,
    to_bytes: Vec<TokenStream2>,
    len: Vec<TokenStream2>,
    tests: Vec<TokenStream2>,
}

impl MessageEnum {
    fn add_message(&mut self, ident: Ident, fields: Fields) -> Result<(), String> {
        let message = &self.message;
        let opcode = &self.opcode;
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

                self.to_bytes.push(quote! {
                    #message::#ident { #(#names,)* } => {
                        let mut out_params = vec![#opcode::#ident as u8];

                        #(#params)*

                        out_params
                    }
                });
                self.len.push(quote! {
                    #message::#ident { #(#names,)* } => {
                        #(let _ = #names;)*
                        1#( + #sizes)*
                    }
                });
            }
            Fields::Unnamed(_) => {
                return Err(format!("Variant {ident} cannot have unnamed fields"));
            }
            Fields::Unit => {
                self.to_bytes
                    .push(quote!(#message::#ident => vec![#opcode::#ident as u8]));
                self.len.push(quote!(#message::#ident => 1));

                self.tests.push(quote! {
                    #[cfg(test)]
                    mod #testname {
                        use crate::Error;
                        use super::*;

                        #[test]
                        fn test_len() {
                            assert_eq!(#message::#ident {}.len(), 1);
                        }

                        #[test]
                        fn test_opcode() {
                            assert_eq!(
                                #message::#ident {}.opcode(),
                                #opcode::#ident
                            );
                        }

                        #[test]
                        fn test_encoding() {
                            assert_eq!(
                                &#message::#ident {}.to_bytes(),
                                &[#opcode::#ident as u8]
                            );
                        }

                        #[test]
                        fn test_decoding() {
                            assert_eq!(
                                #message::try_from_bytes(&[#opcode::#ident as u8]),
                                Ok(#message::#ident {})
                            );
                            assert_eq!(
                                #message::try_from_bytes(&[#opcode::#ident as u8, 0x12]),
                                Ok(#message::#ident {})
                            );
                        }
                    }
                });
            }
        }

        self.from_bytes.push(quote! {
            #opcode::#ident => {
                let offset = 1;

                #(#from_params)*

                #message::#ident {
                    #(#names),*
                }
            }
        });
        Ok(())
    }

    fn process(mut self, data: DataEnum) -> Result<TokenStream2, String> {
        let mut opcodes = Vec::new();
        for variant in data.variants {
            let ident = variant.ident;
            let Some((_, discriminant)) = variant.discriminant else {
                return Err(format!("Variant {} missing discriminant", ident));
            };
            opcodes.push(quote!(#ident = #discriminant));

            self.add_message(ident, variant.fields)?;
        }

        let message = self.message;
        let opcode = self.opcode;
        let from_bytes = self.from_bytes;
        let to_bytes = self.to_bytes;
        let len = self.len;
        let tests = self.tests;

        Ok(quote! {
            #[derive(
                Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand,
            )]
            #[repr(u8)]
            pub enum #opcode {
                #(#opcodes,)*
            }

            impl #message {
                pub fn try_from_bytes(bytes: &[u8]) -> Result<#message> {
                    if bytes.is_empty() {
                        return Err(crate::Error::OutOfRange {
                            expected: crate::Range::AtLeast(1),
                            got: 0,
                            quantity: "bytes",
                        })
                    }
                    Ok(match #opcode::try_from_primitive(bytes[0])? {
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
        })
    }
}

#[proc_macro_derive(MessageEnum)]
pub fn message_enum(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: message,
        data: Data::Enum(data),
        ..
    } = parse_macro_input!(input as DeriveInput)
    else {
        bail!("This macro only works on the Message enum");
    };
    let opcode: Ident = parse_str(match &message {
        x if x == "Message" => "Opcode",
        _ => bail!("This macro only works on the Message or CdcMessage enum"),
    })
    .unwrap();

    let work = MessageEnum {
        message,
        opcode,
        from_bytes: Vec::new(),
        to_bytes: Vec::new(),
        len: Vec::new(),
        tests: Vec::new(),
    };

    match work.process(data) {
        Ok(tokens) => tokens.into(),
        Err(error) => bail!(error),
    }
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
                        quantity: "bytes",
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
                        quantity: "bytes",
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
                        quantity: "bytes",
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
            Fields::Unnamed(data) => match data.unnamed.first() {
                Some(Field {
                    ty: Type::Path(ty), ..
                }) => {
                    if ty.qself.is_some() {
                        bits_u8_encodable(ident)
                    } else {
                        into_u8_encodable(ident)
                    }
                }
                Some(Field {
                    ty: Type::Array(TypeArray { elem, len, .. }),
                    ..
                }) => quote! {
                    impl crate::operand::OperandEncodable for #ident {
                        fn to_bytes(&self, buf: &mut impl Extend<u8>) {
                            <[#elem; #len] as crate::operand::OperandEncodable>::to_bytes(&self.0, buf);
                        }

                        fn try_from_bytes(bytes: &[u8], offset: usize) -> crate::Result<Self> {
                            if bytes.len() != #len {
                                Err(crate::Error::OutOfRange {
                                    expected: crate::Range::Exact(#len),
                                    got: bytes.len() - offset,
                                    quantity: "bytes",
                                })
                            } else {
                                let buf = bytes[offset..offset + #len].first_chunk::<#len>();
                                Ok(#ident(*buf.unwrap()))
                            }
                        }

                        fn len(&self) -> usize {
                            #len
                        }
                    }
                }
                .into(),
                _ => todo!(),
            },
            Fields::Unit => todo!(),
        },
        _ => todo!(),
    }
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
