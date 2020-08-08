// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    codegen::GenerateCode,
    ir::{Contract, FnArg, Function, FunctionKind, Signature},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub struct Dispatch<'a> {
    contract: &'a Contract,
}

impl<'a> From<&'a Contract> for Dispatch<'a> {
    fn from(contract: &'a Contract) -> Self {
        Self { contract }
    }
}

impl<'a> GenerateCode for Dispatch<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let marker = self.generate_external_fn_marker();
        let traits = self.generate_external_fn_traits();
        let dispatch = self.generate_dispatch();
        let entry_point = self.generate_entry_point();

        quote! {
            #[cfg(not(test))]
            const _: () = {
                #marker
                #traits
                #dispatch
                #entry_point
            };
        }
    }
}

fn generate_input_tys<'a>(sig: &'a Signature) -> Vec<&'a syn::Type> {
    sig.inputs
        .iter()
        .skip(1)
        .map(|arg| match arg {
            FnArg::Typed(ident_type) => &ident_type.ty,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
}

fn generate_input_ty_checker(tys: &[&syn::Type]) -> TokenStream2 {
    let guards = tys.iter().map(|ty| quote! {
        <#ty as liquid_lang::The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type>::T
    });

    quote! { (#(#guards,)*) }
}

impl<'a> Dispatch<'a> {
    fn generate_external_fn_marker(&self) -> TokenStream2 {
        quote! {
            pub struct ExternalMarker<S> {
                marker: core::marker::PhantomData<fn() -> S>,
            }

            pub struct DispatchHelper<S, T> {
                marker_s: core::marker::PhantomData<fn() -> S>,
                marker_t: core::marker::PhantomData<fn() -> T>,
            }
        }
    }

    fn generate_external_fn_traits(&self) -> TokenStream2 {
        let (traits, external_markers): (Vec<_>, Vec<_>) = self
            .contract
            .functions
            .iter()
            .filter(|func| matches!(&func.kind, FunctionKind::External(_)))
            .map(|func| self.generate_external_fn_trait(func))
            .unzip();
        let selectors = external_markers.iter().map(|marker| {
            quote!(
                <#marker as liquid_lang::FnSelectors>::KECCAK256_SELECTOR,
                <#marker as liquid_lang::FnSelectors>::SM3_SELECTOR)
        });
        let selector_conflict_detector = quote! {
            const _: () = liquid_lang::selector_conflict_detect::detect(&[#(#selectors,)*]);
        };

        quote! {
            #(#traits)*

            #selector_conflict_detector
        }
    }

    fn generate_external_fn_trait(
        &self,
        func: &Function,
    ) -> (TokenStream2, TokenStream2) {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => unreachable!(),
        };

        let span = func.span();
        let external_marker = quote! { ExternalMarker<[(); #fn_id]> };
        let sig = &func.sig;

        let input_tys = generate_input_tys(sig);
        let input_ty_checker = generate_input_ty_checker(input_tys.as_slice());
        let fn_input = quote_spanned! { sig.inputs.span() =>
            impl liquid_lang::FnInput for #external_marker  {
                type Input = #input_ty_checker;
            }
        };

        let output = &sig.output;
        let output_ty_checker = match output {
            syn::ReturnType::Default => quote_spanned! { output.span() => ()},
            syn::ReturnType::Type(_, ty) => match &(**ty) {
                syn::Type::Tuple(tuple_ty) => {
                    let elems = tuple_ty.elems.iter().map(|elem| {
                        quote! {
                            <#elem as liquid_lang::The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type>::T
                        }
                    });

                    quote! { (#(#elems,)*) }
                }
                other_ty => quote! {
                    <#other_ty as liquid_lang::The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type>::T
                },
            },
        };
        let fn_output = quote_spanned! { output.span() =>
            impl liquid_lang::FnOutput for #external_marker {
                type Output = #output_ty_checker;
            }
        };

        let mut selectors = quote_spanned! { span =>
            impl liquid_lang::ty_mapping::SolTypeName for DispatchHelper<#external_marker, ()> {
                const NAME: &'static [u8] = <() as liquid_lang::ty_mapping::SolTypeName>::NAME;
            }
            impl liquid_lang::ty_mapping::SolTypeNameLen for DispatchHelper<#external_marker, ()> {
                const LEN: usize = <() as liquid_lang::ty_mapping::SolTypeNameLen>::LEN;
            }
        };
        for i in 1..=input_tys.len() {
            let tys = &input_tys[..i];
            let first_tys = &tys[0..i - 1];
            let rest_ty = &tys[i - 1];
            if i > 1 {
                selectors.extend(quote_spanned! { span =>
                    impl liquid_lang::ty_mapping::SolTypeName for DispatchHelper<#external_marker, (#(#tys,)*)> {
                        const NAME: &'static [u8] = {
                            const LEN: usize =
                                <(#(#first_tys,)*) as liquid_lang::ty_mapping::SolTypeNameLen<_>>::LEN
                                + <#rest_ty as liquid_lang::ty_mapping::SolTypeNameLen<_>>::LEN
                                + 1;
                            &liquid_lang::ty_mapping::concat::<DispatchHelper<#external_marker, (#(#first_tys,)*)>, #rest_ty, (), _, LEN>(true)
                        };
                    }
                });
            } else {
                selectors.extend(quote_spanned! { span =>
                    impl liquid_lang::ty_mapping::SolTypeName for DispatchHelper<#external_marker, (#rest_ty,)> {
                        const NAME: &'static [u8] = <#rest_ty as liquid_lang::ty_mapping::SolTypeName<_>>::NAME;
                    }
                });
            }
        }

        let fn_name = sig.ident.to_string();
        let fn_name_bytes = fn_name.as_bytes();
        let fn_name_len = fn_name_bytes.len();
        let composite_sig = quote! {
            const SIG_LEN: usize =
                <(#(#input_tys,)*) as liquid_lang::ty_mapping::SolTypeNameLen<_>>::LEN + #fn_name_len
                + 2;
            const SIG: [u8; SIG_LEN] =
                liquid_lang::ty_mapping::composite::<SIG_LEN>(
                    &[#(#fn_name_bytes),*],
                    <DispatchHelper<#external_marker, (#(#input_tys,)*)> as liquid_lang::ty_mapping::SolTypeName<_>>::NAME);
        };
        selectors.extend(quote_spanned! { span =>
            impl liquid_lang::FnSelectors for #external_marker {
                const KECCAK256_SELECTOR: liquid_primitives::Selector = {
                    #composite_sig
                    liquid_primitives::hash::keccak::keccak256(&SIG)
                };
                const SM3_SELECTOR: liquid_primitives::Selector = {
                    #composite_sig
                    liquid_primitives::hash::sm3::sm3(&SIG)
                };
            }
        });

        let is_mut = sig.is_mut();
        let mutability = quote_spanned! { span =>
            impl liquid_lang::FnMutability for #external_marker {
                const IS_MUT: bool = #is_mut;
            }
        };

        (
            quote_spanned! { span =>
                #fn_input
                #fn_output
                #selectors
                #mutability
                impl liquid_lang::ExternalFn for #external_marker {}
            },
            external_marker,
        )
    }

    fn generate_dispatch_fragment(
        &self,
        func: &Function,
        is_getter: bool,
    ) -> TokenStream2 {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => return quote! {},
        };
        let namespace = quote! { ExternalMarker<[(); #fn_id]> };

        let sig = &func.sig;
        let fn_name = &sig.ident;
        let inputs = &sig.inputs;
        let input_idents = inputs
            .iter()
            .skip(1)
            .map(|arg| match arg {
                FnArg::Typed(ident_type) => &ident_type.ident,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        let pat_idents = if input_idents.is_empty() {
            quote! { _ }
        } else {
            quote! { (#(#input_idents,)*) }
        };

        let builder_name = if sig.is_mut() {
            quote! { on_external_mut }
        } else {
            quote! { on_external }
        };

        let attr = if is_getter {
            quote! { #[allow(deprecated)] }
        } else {
            quote! {}
        };

        quote! {
            .#builder_name::<#namespace>(|storage, #pat_idents| {
                #attr
                storage.#fn_name(#(#input_idents,)*)
            })
        }
    }

    fn generate_dispatch(&self) -> TokenStream2 {
        let fragments = self.contract.functions.iter().enumerate().map(|(i, func)| {
            let is_getter = self.contract.functions.len() - i
                <= self.contract.storage.public_fields.len();
            self.generate_dispatch_fragment(func, is_getter)
        });
        let constr = &self.contract.constructor;
        let constr_sig = &constr.sig;
        let constr_ident = &constr_sig.ident;
        let constr_inputs = &constr_sig.inputs;

        let constr_input_idents = constr_sig
            .inputs
            .iter()
            .skip(1)
            .map(|arg| match arg {
                FnArg::Typed(ident_type) => &ident_type.ident,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        let constr_pat_idents = if constr_input_idents.is_empty() {
            quote! { _ }
        } else {
            quote! { (#(#constr_input_idents,)*) }
        };

        let constr_input_tys = generate_input_tys(constr_sig);
        let constr_marker = quote! { ExternalMarker<[(); 0]> };
        let constr_input_ty_checker =
            generate_input_ty_checker(constr_input_tys.as_slice());
        let constr_input_ty_checker = quote_spanned! { constr_inputs.span() =>
            impl liquid_lang::FnInput for #constr_marker  {
                type Input = #constr_input_ty_checker;
            }
        };

        quote! {
            #constr_input_ty_checker

            impl Storage {
                pub fn dispatch() -> liquid_lang::DispatchResult {
                    liquid_lang::Contract::new_builder::<Storage, (#(#constr_input_tys,)*)>(|storage, #constr_pat_idents| {
                        storage.#constr_ident(#(#constr_input_idents,)*);
                    })
                    #(
                        #fragments
                    )*
                    .done()
                    .dispatch()
                }
            }
        }
    }

    #[cfg(feature = "std")]
    fn generate_entry_point(&self) -> TokenStream2 {
        quote!()
    }

    #[cfg(not(feature = "std"))]
    fn generate_entry_point(&self) -> TokenStream2 {
        quote! {
            #[no_mangle]
            fn main() {
                let ret_info = liquid_lang::DispatchRetInfo::from(Storage::dispatch());
                if !ret_info.is_success() {
                    liquid_core::env::revert(&ret_info.get_info_string());
                }
            }
        }
    }
}
