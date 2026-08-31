use vize_s0::{SmallVec, String};

use crate::bundler::assets::TemplateAssetUrl;
use crate::vite_plugin::js_string::push_js_string_literal;

pub(super) struct AssetReferenceReplacement {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) value: String,
}

pub(super) fn asset_expression(asset: &TemplateAssetUrl) -> String {
    let Some(hash_index) = asset.url.find('#') else {
        return asset.var_name.clone();
    };

    let mut output = String::from(asset.var_name.as_str());
    output.push_str(" + ");
    push_js_string_literal(&mut output, &asset.url[hash_index..]);
    output
}

pub(super) fn push_string_part(parts: &mut SmallVec<[String; 8]>, value: &str) {
    let mut output = String::default();
    push_js_string_literal(&mut output, value);
    parts.push(output);
}

pub(super) fn join_expression_parts(parts: SmallVec<[String; 8]>) -> String {
    let mut output = String::default();
    let mut first = true;

    for part in parts {
        if !first {
            output.push_str(" + ");
        }
        first = false;
        output.push_str(part.as_str());
    }

    output
}

pub(super) fn apply_asset_replacements(
    code: &str,
    mut replacements: Vec<AssetReferenceReplacement>,
) -> String {
    if replacements.is_empty() {
        return String::from(code);
    }

    replacements.sort_by_key(|replacement| replacement.start);
    let mut output = String::with_capacity(code.len());
    let mut last = 0usize;
    let mut changed = false;

    for replacement in replacements {
        if replacement.start < last || replacement.end > code.len() {
            continue;
        }

        output.push_str(&code[last..replacement.start]);
        output.push_str(replacement.value.as_str());
        last = replacement.end;
        changed = true;
    }

    if !changed {
        return String::from(code);
    }

    output.push_str(&code[last..]);
    output
}
