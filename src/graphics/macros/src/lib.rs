use proc_macro::TokenStream;
extern crate proc_macro;
extern crate quote;
use quote::quote;
use proc_macro2::TokenStream as TokenStream2;
use syn::{spanned::Spanned, DataStruct, DeriveInput, Meta};

// #[proc_macro_derive(GraphicsEntity)]
// pub fn graphics_entity_fn(input: TokenStream) -> TokenStream {
//     let ast: DeriveInput = syn::parse(input).expect("Couldn't parse struct");
//     let name = &ast.ident;
//     let generics = &ast.generics;
//     let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

//     let gen = {
//         // Is it a struct?
//         if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
//             let generated = fields.iter().map(|f| generate::implement(f, params));

//             quote! {
//                 impl #impl_generics #name #ty_generics #where_clause {
//                     #(#generated)*
//                 }
//             }
//         } else {
//             // Nope. This is an Enum. We cannot handle these!
//             panic!("#[derive(Getters)] is only defined for structs, not for enums!");
//         }
//     };

//     "fn answer() -> u32 { 42 }".parse().unwrap()
// }
