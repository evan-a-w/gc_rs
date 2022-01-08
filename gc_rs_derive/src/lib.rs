use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

mod tests;

pub(crate) fn trace_derive(s: Structure) -> proc_macro2::TokenStream {
    let trace_body = s.each(|bi| quote! {
        ::gc_rs::Trace::trace(#bi);
    });
    let root_body = s.each(|bi| quote! {
        ::gc_rs::Trace::root(#bi);
        ::gc_rs::Trace::root_children(#bi);
    });
    let deroot_body = s.each(|bi| quote! {
        ::gc_rs::Trace::deroot(#bi);
        ::gc_rs::Trace::deroot_children(#bi);
    });

    s.bound_impl(quote!(::gc_rs::Trace), quote! {
        #[inline]
        fn trace(&self) {
            match self { #trace_body }
        }
        #[inline]
        fn root_children(&self) {
            match self { #root_body }
        }
        #[inline]
        fn deroot_children(&self) {
            match self { #deroot_body }
        }
        #[inline]
        fn root(&self) {}
        #[inline]
        fn deroot(&self) {}
    })
}

decl_derive!([Trace] => trace_derive);
