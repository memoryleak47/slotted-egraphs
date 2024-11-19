use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::*;

#[proc_macro]
pub fn define_language(input: TokenStream1) -> TokenStream1 {
    let mut ie: ItemEnum = parse(input).unwrap();

    let name = ie.ident.clone();
    let names: Vec<Expr> = ie.variants.iter_mut()
                                      .map(|x| {
                                          x.discriminant.take().expect("Name missing!").1
                                      }).collect();
    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        #ie

        impl Language for #name {
            fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> { todo!() }
            fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> { todo!() }
            fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> { todo!() }
            fn to_op(&self) -> (String, Vec<Child>) { todo!() }
            fn from_op(op: &str, children: Vec<Child>) -> Option<Self> { todo!() }
        }
    }.to_token_stream().into()
}
