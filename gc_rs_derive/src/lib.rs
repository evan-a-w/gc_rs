use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure, test_derive};

fn trace_derive(s: Structure) -> proc_macro2::TokenStream {
    let trace_body = s.each(|bi| quote! {
        ::gc_rs::traits::Trace::trace(#bi)
    });
    let root_body = s.each(|bi| quote! {
        ::gc_rs::traits::Trace::root_children(#bi)
    });
    let unroot_body = s.each(|bi| quote! {
        ::gc_rs::traits::Trace::unroot_children(#bi)
    });

    s.bound_impl(quote!(::gc_rs::traits::Trace), quote! {
        fn trace(&self) {
            #trace_body
        }
        fn root_children(&self) {
            #root_body
        }
        fn unroot_children(&self) {
            #unroot_body
        }
    })
}

decl_derive!([Trace] => trace_derive);

fn main() {
    test_derive!{
        trace_derive {
            struct A {
                a: u32,
                b: u32,
            };
        }
        expands to {
            impl ::gc_rs::traits::Trace for A {
                fn trace(&self) {
                    ::gc_rs::traits::Trace::trace(&self.a);
                    ::gc_rs::traits::Trace::trace(&self.b);
                }
                fn root_children(&self) {
                    ::gc_rs::traits::Trace::root_children(&self.a);
                    ::gc_rs::traits::Trace::root_children(&self.b);
                }
                fn unroot_children(&self) {
                    ::gc_rs::traits::Trace::unroot_children(&self.a);
                    ::gc_rs::traits::Trace::unroot_children(&self.b);
                }
            }
        } no_build
    }
}
