use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::*;

// We allow the user to use tuples, Slot, Bind<_>, AppliedId and "user-defined types" in their enum variants.
// user-defined types will be understood as slot-independent constants, and ignored by the system.

#[proc_macro]
pub fn define_language(input: TokenStream1) -> TokenStream1 {
    let mut ie: ItemEnum = parse(input).unwrap();

    let name = ie.ident.clone();
    let str_names: Vec<Option<Expr>> = ie.variants.iter_mut()
                                      .map(|x| {
                                          x.discriminant.take().map(|(_, e)| e)
                                      }).collect();

    let ident_names: Vec<Ident> = ie.variants.iter().map(|x| x.ident.clone()).collect();
    let all_slot_occurences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_all_slot_occurences_mut(&name, x)).collect();
    let public_slot_occurences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_public_slot_occurences_mut(&name, x)).collect();
    let applied_id_occurences_mut_arms: Vec<TokenStream2> = ie.variants.iter().map(|x| produce_applied_id_occurences_mut(&name, x)).collect();
    let to_op_arms: Vec<TokenStream2> = ie.variants.iter().zip(&str_names).map(|(x, n)| produce_to_op(&name, &n, x)).collect();
    let from_op_arms: Vec<TokenStream2> = ie.variants.iter().zip(&str_names).map(|(x, n)| produce_from_op(&name, &n, x)).collect();

    quote! {
        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        #ie

        impl Language for #name {
            fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
                match self {
                    #(#all_slot_occurences_mut_arms),*
                }
            }
            fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
                match self {
                    #(#public_slot_occurences_mut_arms),*
                }
            }
            fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
                match self {
                    #(#applied_id_occurences_mut_arms),*
                }
            }

            fn to_op(&self) -> (String, Vec<Child>) {
                match self {
                    #(#to_op_arms),*
                }
            }

            fn from_op(op: &str, mut children: Vec<Child>) -> Option<Self> {
                match op {
                    #(#from_op_arms),*
                    _ => None,
                }
            }

            fn num_children_hint() -> Option<usize> { None }
        }
    }.to_token_stream().into()
}

fn produce_all_slot_occurences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut Slot> = Vec::new();
            #(
                out.extend(#fields .all_slot_occurences_mut());
            )*
            out
        }
    }
}

fn produce_public_slot_occurences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut Slot> = Vec::new();
            #(
                out.extend(#fields .public_slot_occurences_mut());
            )*
            out
        }
    }
}

fn produce_applied_id_occurences_mut(name: &Ident, v: &Variant) -> TokenStream2 {
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<&mut AppliedId> = Vec::new();
            #(
                out.extend(#fields .applied_id_occurences_mut());
            )*
            out
        }
    }
}

fn produce_to_op(name: &Ident, e: &Option<Expr>, v: &Variant) -> TokenStream2 {
    let e = e.as_ref().unwrap();
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();
    quote! {
        #name::#variant_name(#(#fields),*) => {
            let mut out: Vec<Child> = Vec::new();
            #(
                let (l, ch) = #fields.to_op();
                assert!(l == "");
                out.extend(ch);
            )*
            (String::from(#e), out)
        }
    }
}

fn produce_from_op(name: &Ident, e: &Option<Expr>, v: &Variant) -> TokenStream2 {
    let e = e.as_ref().unwrap();
    let variant_name = &v.ident;
    let n = v.fields.len();
    let fields: Vec<Ident> = (0..n).map(|x| Ident::new(&format!("a{x}"), proc_macro2::Span::call_site())).collect();

    let types: Vec<Type> = v.fields.iter().map(|x| x.ty.clone()).collect();

    quote! {
        #e => {
            #(
                let n = <#types>::num_children_hint().unwrap();
                let mut new = children.split_off(n);
                std::mem::swap(&mut new, &mut children);

                let #fields = <#types>::from_op("", new)?;
            )*
            Some(#name::#variant_name(#(#fields),*))
        }
    }
}
