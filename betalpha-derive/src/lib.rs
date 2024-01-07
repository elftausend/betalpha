use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::DeriveInput;

#[proc_macro_attribute]
pub fn serialize(attr: TokenStream, input: TokenStream) -> TokenStream {
    let packet_id = syn::parse(attr).unwrap();
    let ast = syn::parse(input).unwrap();
    implement_serialize_trait(&packet_id, &ast)
}

fn implement_serialize_trait(attr: &syn::Expr, ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    if let syn::Data::Struct(data) = &ast.data {
        let fields = &data.fields;
        let fields: Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = fields
            .iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap().to_token_stream();
                let ty = f.ty.to_token_stream();
                (ident, ty)
            })
            .collect();
        let mut body = proc_macro2::TokenStream::new();
        quote!(
            serializer.serialize_u8(#attr)?;
        )
        .to_tokens(&mut body);
        for (ident, ty) in fields {
            let line = match ty.to_string().as_str() {
                "bool" => quote! {serializer.serialize_bool(self.#ident)?;},
                "u8" => quote! {serializer.serialize_u8(self.#ident)?;},
                "u16" => quote! {serializer.serialize_u16(self.#ident)?;},
                "u32" => quote! {serializer.serialize_u32(self.#ident)?;},
                "u64" => quote! {serializer.serialize_u64(self.#ident)?;},
                "f32" => quote! {serializer.serialize_f32(self.#ident)?;},
                "f64" => quote! {serializer.serialize_f64(self.#ident)?;},
                "i8" => quote! {serializer.serialize_i8(self.#ident)?;},
                "i16" => quote! {serializer.serialize_i16(self.#ident)?;},
                "i32" => quote! {serializer.serialize_i32(self.#ident)?;},
                "i64" => quote! {serializer.serialize_i64(self.#ident)?;},
                "Vec < u8 >" => quote! {serializer.serialize_payload(self.#ident.clone())?;},
                "String" => quote! {serializer.serialize_string(self.#ident.clone())?;},
                _ => quote! {serializer.serialize_payload(self.#ident.serialize()?)?;},
            };
            line.to_tokens(&mut body);
        }

        let gen = quote! {
            #ast

            impl crate::packet::parse::Serialize for #name {
                fn serialize(&self) -> Result<Vec<u8>, PacketError> {
                    let mut serializer = PacketSerializer::default();
                    #body
                    Ok(serializer.output)
                }
            }
        };
        gen.into()
    } else {
        panic!("not a struct")
    }
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialization(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    implement_deserialize_trait(&ast)
}

fn implement_deserialize_trait(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    if let syn::Data::Struct(data) = &ast.data {
        let fields = &data.fields;
        let fields: Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = fields
            .iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap().to_token_stream();
                let ty = f.ty.to_token_stream();
                (ident, ty)
            })
            .collect();
        let mut body = proc_macro2::TokenStream::new();
        let mut construction = proc_macro2::TokenStream::new();
        for (ident, ty) in fields {
            let line = match ty.to_string().as_str() {
                "bool" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_bool(cursor)?;}
                }
                "u8" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_u8(cursor)?;}
                }
                "u16" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_u16(cursor)?;}
                }
                "u32" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_u32(cursor)?;}
                }
                "u64" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_u64(cursor)?;}
                }
                "f32" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_f32(cursor)?;}
                }
                "f64" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_f64(cursor)?;}
                }
                "i8" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_i8(cursor)?;}
                }
                "i16" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_i16(cursor)?;}
                }
                "i32" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_i32(cursor)?;}
                }
                "i64" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_i64(cursor)?;}
                }
                "Vec < u8 >" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_payload(cursor)?;}
                }
                "String" => {
                    quote! {let (n, #ident): (usize, #ty) = crate::packet::parse::PacketDeserializer::deserialize_string(cursor)?;}
                }
                _ => quote! {let #ident = #ty::nested_deserialize(cursor)?;},
            };
            (quote! {#ident,}).to_tokens(&mut construction);
            line.to_tokens(&mut body);
        }
        let gen = quote! {
            impl crate::packet::parse::Deserialize for #name {
                fn nested_deserialize(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Self, PacketError> {
                    #body
                    Ok(Self {
                        #construction
                    })
                }
            }
        };
        gen.into()
    } else {
        panic!("not a struct")
    }
}
