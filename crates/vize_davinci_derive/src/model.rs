//! Input validation: a `DeriveInput` becomes a [`PageModel`] or an exact,
//! tested error.
//!
//! The derive is deliberately narrow. It accepts a non-generic struct with
//! named fields and classifies each field by the last path segment of its
//! type: `Vec` is a list section, `FxHashMap` is a sorted map section, and
//! everything else is a scalar line resolved through `FolioValue` (so an
//! unsupported scalar type fails to compile in the deriving crate rather
//! than silently formatting as something).

use syn::{Data, DeriveInput, Error, Fields, Ident, LitStr, Type};

/// How one field lands on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// `name=value` line in the header section.
    Scalar,
    /// `[page.name]` section, one entry per line, order preserved.
    List,
    /// `[page.name]` section, `key=value` lines sorted by printed key.
    Map,
}

/// One field of the deriving struct.
pub struct PageField {
    pub ident: Ident,
    /// The field's name on the page (the identifier, raw-prefix stripped).
    pub name: String,
    pub ty: Type,
    pub kind: FieldKind,
}

/// Everything codegen needs to know about the deriving type.
pub struct PageModel {
    pub ident: Ident,
    /// The page header name: an explicit wire name or kebab-case type name.
    pub page: String,
    pub fields: Vec<PageField>,
}

impl PageModel {
    /// Validate `input` into a model.
    ///
    /// # Errors
    ///
    /// Rejects enums, unions, tuple/unit structs and generic types; the
    /// messages are part of the contract and tested exactly.
    pub fn from_input(input: &DeriveInput) -> Result<Self, Error> {
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(Error::new_spanned(
                &input.generics,
                "#[derive(Folio)] does not support generic types: a folio page is an owned document",
            ));
        }
        let Data::Struct(data) = &input.data else {
            return Err(Error::new_spanned(
                &input.ident,
                "#[derive(Folio)] supports only structs with named fields",
            ));
        };
        let Fields::Named(named) = &data.fields else {
            return Err(Error::new_spanned(
                &input.ident,
                "#[derive(Folio)] supports only structs with named fields",
            ));
        };

        let mut fields = Vec::new();
        let mut sections = 0usize;
        for field in &named.named {
            let Some(ident) = field.ident.clone() else {
                return Err(Error::new_spanned(field, "a named field has no identifier"));
            };
            let kind = classify(&field.ty);
            if kind != FieldKind::Scalar {
                sections += 1;
            }
            fields.push(PageField {
                name: unraw(&ident),
                ident,
                ty: field.ty.clone(),
                kind,
            });
        }
        if sections > 64 {
            return Err(Error::new_spanned(
                &input.ident,
                "#[derive(Folio)] supports at most 64 list/map sections per page",
            ));
        }

        Ok(Self {
            page: page_name(input)?,
            ident: input.ident.clone(),
            fields,
        })
    }
}

/// An explicit stable wire header, or the mechanical type-derived default.
fn page_name(input: &DeriveInput) -> Result<String, Error> {
    let mut name = None;
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("folio"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("name") {
                return Err(meta.error("unsupported folio attribute; expected name"));
            }
            if name.is_some() {
                return Err(meta.error("duplicate folio name"));
            }
            let value: LitStr = meta.value()?.parse()?;
            let text = value.value();
            if text.is_empty()
                || !text
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(Error::new_spanned(
                    value,
                    "folio name must be a non-empty lowercase page identifier",
                ));
            }
            name = Some(text);
            Ok(())
        })?;
    }
    Ok(name.unwrap_or_else(|| kebab_case(&unraw(&input.ident))))
}

/// The identifier's text with any `r#` raw prefix stripped.
fn unraw(ident: &Ident) -> String {
    let text = ident.to_string();
    match text.strip_prefix("r#") {
        Some(stripped) => stripped.to_owned(),
        None => text,
    }
}

/// Classify a field by its type's last path segment.
fn classify(ty: &Type) -> FieldKind {
    let Type::Path(path) = ty else {
        return FieldKind::Scalar;
    };
    match path.path.segments.last() {
        Some(segment) if segment.ident == "Vec" => FieldKind::List,
        Some(segment) if segment.ident == "FxHashMap" => FieldKind::Map,
        _ => FieldKind::Scalar,
    }
}

/// Kebab-case a type name: a `-` before every non-leading uppercase letter,
/// then lowercase everything. `BudgetObserver` becomes `budget-observer`.
pub fn kebab_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::{FieldKind, PageModel, kebab_case};

    fn model_of(input: DeriveInput) -> Result<PageModel, syn::Error> {
        PageModel::from_input(&input)
    }

    /// `expect_err` needs `Debug` on the success type, and syn's AST types
    /// only carry `Debug` behind `extra-traits`; match instead.
    fn error_of(input: DeriveInput) -> syn::Error {
        match PageModel::from_input(&input) {
            Ok(_) => panic!("input must not derive"),
            Err(error) => error,
        }
    }

    #[test]
    fn kebab_case_is_mechanical() {
        assert_eq!(kebab_case("BudgetObserver"), "budget-observer");
        assert_eq!(kebab_case("L2Folio"), "l2-folio");
        assert_eq!(kebab_case("Sample"), "sample");
    }

    #[test]
    fn a_named_struct_classifies_by_type_shape() {
        let model = model_of(parse_quote! {
            struct Sample {
                title: String,
                count: u32,
                notes: Vec<String>,
                weights: FxHashMap<String, u32>,
            }
        })
        .expect("a named struct derives");
        assert_eq!(model.page, "sample");
        let kinds: Vec<FieldKind> = model.fields.iter().map(|field| field.kind).collect();
        assert_eq!(
            kinds,
            [
                FieldKind::Scalar,
                FieldKind::Scalar,
                FieldKind::List,
                FieldKind::Map
            ]
        );
    }

    #[test]
    fn explicit_page_names_preserve_headers_after_type_renames() {
        let model = model_of(parse_quote! {
            #[folio(name = "s3-folio")]
            struct L3Folio { phase: String }
        })
        .expect("a stable wire header derives");
        assert_eq!(model.page, "s3-folio");
        for input in [
            parse_quote! { #[folio(name = "bad.name")] struct Sample { x: u32 } },
            parse_quote! { #[folio(name = "")] struct Sample { x: u32 } },
            parse_quote! { #[folio(name = "one", name = "two")] struct Sample { x: u32 } },
            parse_quote! { #[folio(other = "one")] struct Sample { x: u32 } },
        ] {
            let _ = error_of(input);
        }
    }

    #[test]
    fn an_enum_is_rejected_with_the_exact_message() {
        let error = error_of(parse_quote! {
            enum Nope { A, B }
        });
        assert_eq!(
            error.to_string(),
            "#[derive(Folio)] supports only structs with named fields"
        );
    }

    #[test]
    fn a_tuple_struct_is_rejected_with_the_exact_message() {
        let error = error_of(parse_quote! {
            struct Nope(u32, u32);
        });
        assert_eq!(
            error.to_string(),
            "#[derive(Folio)] supports only structs with named fields"
        );
    }

    #[test]
    fn a_unit_struct_is_rejected_with_the_exact_message() {
        let error = error_of(parse_quote! {
            struct Nope;
        });
        assert_eq!(
            error.to_string(),
            "#[derive(Folio)] supports only structs with named fields"
        );
    }

    #[test]
    fn a_generic_struct_is_rejected_with_the_exact_message() {
        let error = error_of(parse_quote! {
            struct Nope<T> { value: T }
        });
        assert_eq!(
            error.to_string(),
            "#[derive(Folio)] does not support generic types: a folio page is an owned document"
        );
    }

    #[test]
    fn a_raw_identifier_field_drops_its_prefix_on_the_page() {
        let model = model_of(parse_quote! {
            struct Sample { r#type: u32 }
        })
        .expect("raw identifiers derive");
        assert_eq!(model.fields[0].name, "type");
    }
}
