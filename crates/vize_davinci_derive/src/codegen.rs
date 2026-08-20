//! Code generation: a [`PageModel`] becomes the `Folio` impl.
//!
//! Everything the generated code names is fully qualified through
//! `::vize_davinci` (the deriving crate reaches itself the same way via
//! `extern crate self as vize_davinci`) or `::core` - never `::std` or
//! `::alloc` - because the generated code runs inside `no_std + alloc`
//! crates. The runtime halves live in `vize_davinci::folio::page` and
//! `vize_davinci::folio::value`; this module only wires fields to them in
//! declaration order, which is what makes the field order stable.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::model::{FieldKind, PageModel};

/// Expand the model into `impl Folio for T`.
pub fn expand(model: &PageModel) -> TokenStream {
    let ident = &model.ident;
    let page = model.page.as_str();
    let print_body = print_body(model, page);
    let parse_body = parse_body(model, page);

    quote! {
        #[automatically_derived]
        impl ::vize_davinci::folio::Folio for #ident {
            fn print<W: ::core::fmt::Write>(
                &self,
                w: &mut W,
                _mode: ::vize_davinci::folio::FolioMode,
            ) -> ::core::fmt::Result {
                #print_body
            }

            fn parse(
                input: &str,
            ) -> ::core::result::Result<Self, ::vize_davinci::folio::FolioError> {
                #parse_body
            }
        }
    }
}

/// The `print` body: header with scalars in declaration order, then the
/// list/map sections in declaration order. `_mode` is deliberately unread -
/// a derived page has no `Display` elision, because eliding is a semantic
/// decision and the derive makes none.
fn print_body(model: &PageModel, page: &str) -> TokenStream {
    let mut header = TokenStream::new();
    let mut sections = TokenStream::new();
    for field in &model.fields {
        let ident = &field.ident;
        let name = field.name.as_str();
        match field.kind {
            FieldKind::Scalar => header.extend(quote! {
                printer.scalar(#name, &self.#ident)?;
            }),
            FieldKind::List => sections.extend(quote! {
                printer.list(#name, &self.#ident)?;
            }),
            FieldKind::Map => sections.extend(quote! {
                printer.map(#name, &self.#ident)?;
            }),
        }
    }
    quote! {
        let mut printer = ::vize_davinci::folio::page::PagePrinter::new(w, #page);
        printer.open()?;
        #header
        printer.close_header()?;
        #sections
        ::core::result::Result::Ok(())
    }
}

/// The `parse` body: a line-driven loop over [`ParseState::classify`]
/// events, lenient about scalar-line and section order (print normalizes),
/// strict about everything else.
fn parse_body(model: &PageModel, page: &str) -> TokenStream {
    let mut slots = TokenStream::new();
    let mut field_arms = TokenStream::new();
    let mut section_arms = TokenStream::new();
    let mut entry_arms = TokenStream::new();
    let mut build = TokenStream::new();
    let mut section_index = 0usize;

    for field in &model.fields {
        let ident = &field.ident;
        let name = field.name.as_str();
        let slot = format_ident!("__field_{}", name);
        let ty = &field.ty;
        match field.kind {
            FieldKind::Scalar => {
                slots.extend(quote! {
                    let mut #slot: ::core::option::Option<#ty> = ::core::option::Option::None;
                });
                field_arms.extend(quote! {
                    #name => ::vize_davinci::folio::page::set_scalar(
                        &mut #slot, #name, __value, __line_no,
                    )?,
                });
                build.extend(quote! {
                    #ident: ::vize_davinci::folio::page::require_scalar(#slot, #name)?,
                });
            }
            FieldKind::List | FieldKind::Map => {
                let index = section_index;
                section_index += 1;
                slots.extend(quote! {
                    let mut #slot: #ty = <#ty as ::core::default::Default>::default();
                });
                section_arms.extend(quote! {
                    #name => state.enter_section(#index, #name, __line_no)?,
                });
                let insert = if field.kind == FieldKind::List {
                    quote! {
                        #slot.push(::vize_davinci::folio::value::FolioValue::parse_value(
                            __line, __line_no,
                        )?)
                    }
                } else {
                    quote! {
                        ::vize_davinci::folio::page::map_insert(&mut #slot, __line, __line_no)?
                    }
                };
                entry_arms.extend(quote! { #index => #insert, });
                build.extend(quote! { #ident: #slot, });
            }
        }
    }

    quote! {
        let mut state = ::vize_davinci::folio::page::ParseState::new(#page);
        #slots
        let mut __line_no = 0usize;
        for __line in input.split('\n') {
            __line_no += 1;
            match state.classify(__line, __line_no)? {
                ::vize_davinci::folio::page::LineEvent::Skip => {}
                ::vize_davinci::folio::page::LineEvent::Field => {
                    let (__name, __value) =
                        ::vize_davinci::folio::page::split_field(__line, __line_no)?;
                    match __name {
                        #field_arms
                        _ => {
                            return ::core::result::Result::Err(
                                ::vize_davinci::folio::page::unknown_field(__name, __line_no),
                            );
                        }
                    }
                }
                ::vize_davinci::folio::page::LineEvent::Section(__name) => match __name {
                    #section_arms
                    _ => {
                        return ::core::result::Result::Err(
                            state.unknown_section(__name, __line_no),
                        );
                    }
                },
                ::vize_davinci::folio::page::LineEvent::Entry(__index) => match __index {
                    #entry_arms
                    _ => ::core::unreachable!("entered sections have generated arms"),
                },
            }
        }
        state.require_header()?;
        ::core::result::Result::Ok(Self { #build })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::model::PageModel;

    #[test]
    fn a_full_shape_expands() {
        let model = PageModel::from_input(&parse_quote! {
            struct Sample {
                title: String,
                count: u32,
                notes: Vec<String>,
                weights: FxHashMap<String, u32>,
            }
        })
        .expect("a named struct derives");
        let expanded = super::expand(&model).to_string();
        // The proof the expansion is *right* lives in vize_davinci's TS-16
        // suite, where the generated impls compile and hold the round-trip
        // laws; this test pins only that expansion succeeds and is non-empty.
        assert!(!expanded.is_empty(), "expansion must produce tokens");
    }
}
