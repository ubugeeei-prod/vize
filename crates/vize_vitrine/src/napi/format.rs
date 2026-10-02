//! NAPI bindings for Vue SFC formatting.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_glyph::{Allocator, FormatOptions, GlyphFormatter, VueVersion, resolve_sort_imports};

/// Format options for NAPI.
#[napi(object)]
#[derive(Default)]
pub struct FormatOptionsNapi {
    /// Explicit Vue version; omitted uses Vue 3.
    pub vue_version: Option<String>,
    /// Oxfmt-compatible import sorting; false disables it.
    #[napi(
        ts_type = "false | { partitionByNewline?: boolean; partitionByComment?: boolean; sortSideEffects?: boolean; order?: 'asc' | 'desc'; ignoreCase?: boolean; newlinesBetween?: boolean; internalPattern?: string[]; groups?: (string | string[] | { newlinesBetween: boolean })[]; customGroups?: { groupName: string; elementNamePattern?: string[]; selector?: string; modifiers?: string[] }[] }"
    )]
    pub sort_imports: Option<serde_json::Value>,
    pub print_width: Option<u32>,
    pub tab_width: Option<u8>,
    pub use_tabs: Option<bool>,
    pub semi: Option<bool>,
    pub single_quote: Option<bool>,
    pub sort_attributes: Option<bool>,
    pub single_attribute_per_line: Option<bool>,
    pub max_attributes_per_line: Option<u32>,
    pub normalize_directive_shorthands: Option<bool>,
}

/// Format result for NAPI.
#[napi(object)]
pub struct FormatResultNapi {
    pub code: String,
    pub changed: bool,
}

fn apply_options(opts: FormatOptionsNapi) -> FormatOptions {
    let mut options = FormatOptions::default();

    if let Some(value) = opts.print_width {
        options.print_width = value;
    }
    if let Some(value) = opts.tab_width {
        options.tab_width = value;
    }
    if let Some(value) = opts.use_tabs {
        options.use_tabs = value;
    }
    if let Some(value) = opts.semi {
        options.semi = value;
    }
    if let Some(value) = opts.single_quote {
        options.single_quote = value;
    }
    if let Some(value) = opts.sort_attributes {
        options.sort_attributes = value;
    }
    if let Some(value) = opts.single_attribute_per_line {
        options.single_attribute_per_line = value;
    }
    if let Some(value) = opts.max_attributes_per_line {
        options.max_attributes_per_line = Some(value);
    }
    if let Some(value) = opts.normalize_directive_shorthands {
        options.normalize_directive_shorthands = value;
    }

    options
}

/// Format a Vue SFC source string.
#[napi(js_name = "formatSfc")]
pub fn format_sfc_napi(
    source: String,
    options: Option<FormatOptionsNapi>,
) -> Result<FormatResultNapi> {
    let options = options.unwrap_or_default();
    let vue_version = options
        .vue_version
        .as_deref()
        .map(|version| {
            VueVersion::from_config_str(version)
                .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    let sorting = options
        .sort_imports
        .as_ref()
        .map(|value| serde_json::from_value::<vize_l0::config::SortImportsSetting>(value.clone()))
        .transpose()
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    let sorting = resolve_sort_imports(sorting.as_ref())
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    let options = apply_options(options);
    let allocator = Allocator::with_capacity(source.len() * 2);
    let result = GlyphFormatter::new_with_vue_version(&options, &allocator, vue_version)
        .with_sort_imports(sorting.as_ref())
        .format(&source)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;

    Ok(FormatResultNapi {
        code: result.code.into(),
        changed: result.changed,
    })
}

#[cfg(test)]
mod tests {
    use super::{FormatOptionsNapi, format_sfc_napi};

    #[test]
    fn native_sfc_sorting_uses_the_authored_full_reference() {
        let source = include_str!("../../../vize_glyph/tests/fixtures/sort-imports/UserCard.vue");
        let expected =
            include_str!("../../../vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue");
        let output = format_sfc_napi(
            source.into(),
            Some(FormatOptionsNapi {
                sort_imports: Some(serde_json::json!({})),
                ..Default::default()
            }),
        )
        .expect("native formatting");
        assert_eq!(output.code, expected);
        assert!(output.changed);
    }
}
