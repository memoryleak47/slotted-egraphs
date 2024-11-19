use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::*;

#[proc_macro]
pub fn define_language(input: TokenStream1) -> TokenStream1 {
    let mut xx: ItemEnum = parse(input).unwrap();
    xx.variants.iter_mut().for_each(|x| {
        x.attrs.clear();
        x.discriminant = None;
    });
    quote! { #xx }.to_token_stream().into()
}
