use std::borrow::Cow;
use std::path::{Component, Path};

use sha2::{Digest, Sha256};
use vize_carton::{SmallVec, String};

use crate::{SfcCustomBlock, SfcParseOptions, SfcStyleBlock, parse_sfc};

const DEFAULT_ASSET_URL_TAGS: &[(&str, &[&str])] = &[
    ("img", &["src"]),
    ("video", &["src", "poster"]),
    ("source", &["src"]),
    ("image", &["xlink:href", "href"]),
    ("use", &["xlink:href", "href"]),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SfcBlockAttribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlerStyleBlock {
    pub content: String,
    pub src: Option<String>,
    pub lang: Option<String>,
    pub scoped: bool,
    pub module: bool,
    pub module_name: Option<String>,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlerCustomBlock {
    pub block_type: String,
    pub content: String,
    pub src: Option<String>,
    pub attrs: Vec<SfcBlockAttribute>,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SfcSrcInfo {
    pub script_src: Option<String>,
    pub template_src: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateAssetUrl {
    pub url: String,
    pub var_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateAssetTagRule {
    pub tag: String,
    pub attrs: Vec<String>,
}

pub fn generate_bundler_scope_id(
    filename: &str,
    root: Option<&str>,
    is_production: bool,
    source: Option<&str>,
) -> String {
    let input = if let Some(root) = root {
        let relative = relative_scope_path(filename, root);
        if is_production && let Some(source) = source {
            let normalized_source = normalize_newlines(source);
            let mut input = String::with_capacity(relative.len() + normalized_source.len() + 1);
            input.push_str(relative.as_str());
            input.push('\n');
            input.push_str(normalized_source.as_str());
            input
        } else {
            relative
        }
    } else {
        normalize_path(filename)
    };

    sha256_prefix(input.as_str(), 8)
}

pub fn extract_style_blocks(source: &str, filename: Option<&str>) -> Vec<BundlerStyleBlock> {
    parse_descriptor(source, filename).map_or_else(Vec::new, |descriptor| {
        descriptor
            .styles
            .iter()
            .enumerate()
            .map(style_block_to_bundler)
            .collect()
    })
}

pub fn extract_custom_blocks(source: &str, filename: Option<&str>) -> Vec<BundlerCustomBlock> {
    parse_descriptor(source, filename).map_or_else(Vec::new, |descriptor| {
        descriptor
            .custom_blocks
            .iter()
            .enumerate()
            .map(custom_block_to_bundler)
            .collect()
    })
}

pub fn extract_src_info(source: &str, filename: Option<&str>) -> SfcSrcInfo {
    let Some(descriptor) = parse_descriptor(source, filename) else {
        return SfcSrcInfo {
            script_src: None,
            template_src: None,
        };
    };

    let script_src = descriptor
        .script
        .as_ref()
        .or(descriptor.script_setup.as_ref())
        .and_then(|script| script.src.as_deref())
        .map(String::from);
    let template_src = descriptor
        .template
        .as_ref()
        .and_then(|template| template.src.as_deref())
        .map(String::from);

    SfcSrcInfo {
        script_src,
        template_src,
    }
}

pub fn has_scoped_style(source: &str, filename: Option<&str>) -> bool {
    parse_descriptor(source, filename)
        .is_some_and(|descriptor| descriptor.styles.iter().any(|style| style.scoped))
}

pub fn is_importable_asset_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("//")
        || url.starts_with("data:")
    {
        return false;
    }

    url.starts_with("./") || url.starts_with("../") || url.starts_with("@/") || url.starts_with('~')
}

pub fn collect_template_asset_urls(
    source: &str,
    rules: Option<&[TemplateAssetTagRule]>,
    filename: Option<&str>,
) -> Vec<TemplateAssetUrl> {
    let Some(descriptor) = parse_descriptor(source, filename) else {
        return Vec::new();
    };
    let Some(template) = descriptor.template else {
        return Vec::new();
    };

    let mut urls: Vec<TemplateAssetUrl> = Vec::new();
    let mut counter = 0usize;
    scan_template_asset_urls(template.content.as_ref(), rules, &mut urls, &mut counter);
    urls
}

pub fn strip_css_comments_for_scoped(css: &str) -> String {
    if !css.contains("/*") {
        return String::from(css);
    }

    let bytes = css.as_bytes();
    let mut output = String::with_capacity(css.len());
    let mut copy_start = 0usize;
    let mut index = 0usize;
    let mut changed = false;

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if byte == b'\\' {
                        index = (index + 2).min(bytes.len());
                        continue;
                    }
                    index += 1;
                    if byte == quote {
                        break;
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                output.push_str(&css[copy_start..index]);
                output.push_str("  ");
                index += 2;
                while index < bytes.len() {
                    if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        output.push_str("  ");
                        index += 2;
                        break;
                    }
                    output.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
                    index += 1;
                }
                copy_start = index;
                changed = true;
            }
            _ => index += 1,
        }
    }

    if !changed {
        return String::from(css);
    }

    output.push_str(&css[copy_start..]);
    output
}

pub fn wrap_scoped_preprocessor_style(
    content: &str,
    scoped: Option<&str>,
    lang: Option<&str>,
) -> String {
    let Some(scoped) = scoped else {
        return String::from(content);
    };
    let Some(lang) = lang else {
        return String::from(content);
    };
    if lang == "css" {
        return String::from(content);
    }

    let mut hoisted: SmallVec<[&str; 4]> = SmallVec::new();
    let mut body: Vec<&str> = Vec::new();

    for line in content.split('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("@use ")
            || trimmed.starts_with("@forward ")
            || trimmed.starts_with("@import ")
        {
            hoisted.push(line);
        } else {
            body.push(line);
        }
    }

    let mut output = String::with_capacity(content.len() + scoped.len() + 8);
    if !hoisted.is_empty() {
        push_joined_lines(&mut output, &hoisted);
        output.push_str("\n\n");
    }
    output.push('[');
    output.push_str(scoped);
    output.push_str("] {\n");
    push_joined_lines(&mut output, &body);
    output.push_str("\n}");
    output
}

fn parse_descriptor<'a>(
    source: &'a str,
    filename: Option<&str>,
) -> Option<crate::SfcDescriptor<'a>> {
    parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.unwrap_or("anonymous.vue").into(),
            ..Default::default()
        },
    )
    .ok()
}

fn style_block_to_bundler((index, style): (usize, &SfcStyleBlock<'_>)) -> BundlerStyleBlock {
    let module_attr = style.attrs.get("module");
    let module_name = module_attr.and_then(|value| {
        let value = value.as_ref();
        if value.is_empty() {
            None
        } else {
            Some(String::from(value))
        }
    });

    BundlerStyleBlock {
        content: String::from(style.content.as_ref()),
        src: style.src.as_deref().map(String::from),
        lang: style.lang.as_deref().map(String::from),
        scoped: style.scoped,
        module: module_attr.is_some(),
        module_name,
        index: index as u32,
    }
}

fn custom_block_to_bundler((index, block): (usize, &SfcCustomBlock<'_>)) -> BundlerCustomBlock {
    let mut attrs = block_attrs(&block.attrs);
    attrs.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    BundlerCustomBlock {
        block_type: String::from(block.block_type.as_ref()),
        content: String::from(block.content.as_ref()),
        src: block
            .attrs
            .get("src")
            .map(|value| String::from(value.as_ref())),
        attrs,
        index: index as u32,
    }
}

fn block_attrs(
    attrs: &vize_carton::FxHashMap<Cow<'_, str>, Cow<'_, str>>,
) -> Vec<SfcBlockAttribute> {
    attrs
        .iter()
        .map(|(name, value)| SfcBlockAttribute {
            name: String::from(name.as_ref()),
            value: if value.is_empty() {
                None
            } else {
                Some(String::from(value.as_ref()))
            },
        })
        .collect()
}

fn relative_scope_path(filename: &str, root: &str) -> String {
    let file_components = normal_components(Path::new(filename));
    let root_components = normal_components(Path::new(root));
    let mut common = 0usize;
    while common < file_components.len()
        && common < root_components.len()
        && file_components[common].as_str() == root_components[common].as_str()
    {
        common += 1;
    }

    let parent_count = root_components.len().saturating_sub(common);
    let mut parts: Vec<&str> = Vec::with_capacity(parent_count + file_components.len() - common);
    parts.extend(std::iter::repeat_n("..", parent_count));
    parts.extend(file_components[common..].iter().map(String::as_str));

    let mut start = 0usize;
    while parts.get(start) == Some(&"..") {
        start += 1;
    }
    join_slash(&parts[start..])
}

fn normal_components(path: &Path) -> Vec<String> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                components.push(String::from(part.to_string_lossy().as_ref()))
            }
            Component::ParentDir => components.push(String::from("..")),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    components
}

fn normalize_path(path: &str) -> String {
    let mut output = String::with_capacity(path.len());
    for char in path.chars() {
        output.push(if char == '\\' { '/' } else { char });
    }
    output
}

fn normalize_newlines(source: &str) -> String {
    if !source.contains("\r\n") {
        return String::from(source);
    }
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(char) = chars.next() {
        if char == '\r' && chars.peek() == Some(&'\n') {
            continue;
        }
        output.push(char);
    }
    output
}

fn sha256_prefix(input: &str, len: usize) -> String {
    let digest = Sha256::digest(input.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(len);
    for byte in digest.iter().take(len.div_ceil(2)) {
        output.push(HEX[(byte >> 4) as usize] as char);
        if output.len() == len {
            break;
        }
        output.push(HEX[(byte & 0x0f) as usize] as char);
        if output.len() == len {
            break;
        }
    }
    output
}

fn scan_template_asset_urls(
    template: &str,
    rules: Option<&[TemplateAssetTagRule]>,
    urls: &mut Vec<TemplateAssetUrl>,
    counter: &mut usize,
) {
    let mut cursor = 0usize;
    while let Some(offset) = template[cursor..].find('<') {
        let start = cursor + offset;
        let Some(tag) = parse_opening_tag(template, start) else {
            cursor = start + 1;
            continue;
        };

        if let Some(attrs) = attrs_for_tag(rules, tag.name.as_str()) {
            for attr in attrs {
                if let Some(value) = static_attr_value(tag.attrs, attr)
                    && is_importable_asset_url(value)
                    && !urls.iter().any(|entry| entry.url.as_str() == value)
                {
                    let mut var_name = String::from("_imports_");
                    push_usize(&mut var_name, *counter);
                    *counter += 1;
                    urls.push(TemplateAssetUrl {
                        url: String::from(value),
                        var_name,
                    });
                }
            }
        }

        cursor = tag.end;
    }
}

struct OpeningTag<'a> {
    name: String,
    attrs: &'a str,
    end: usize,
}

fn parse_opening_tag(template: &str, start: usize) -> Option<OpeningTag<'_>> {
    let bytes = template.as_bytes();
    if matches!(bytes.get(start + 1), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let mut index = start + 1;
    while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        index += 1;
    }
    let name_start = index;
    while matches!(
        bytes.get(index),
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b':' | b'_')
    ) {
        index += 1;
    }
    if index == name_start {
        return None;
    }
    let name = String::from(&template[name_start..index]);
    let attrs_start = index;
    let mut quote = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if byte == active_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(OpeningTag {
                name,
                attrs: &template[attrs_start..index],
                end: index + 1,
            });
        }
        index += 1;
    }
    None
}

fn attrs_for_tag<'a>(rules: Option<&'a [TemplateAssetTagRule]>, tag: &str) -> Option<Vec<&'a str>> {
    if let Some(rules) = rules {
        return rules
            .iter()
            .find(|rule| rule.tag.as_str().eq_ignore_ascii_case(tag))
            .map(|rule| rule.attrs.iter().map(String::as_str).collect());
    }

    DEFAULT_ASSET_URL_TAGS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(tag))
        .map(|(_, attrs)| attrs.to_vec())
}

fn static_attr_value<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let bytes = attrs.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r' | b'/')) {
            index += 1;
        }
        let name_start = index;
        while index < bytes.len()
            && !matches!(
                bytes[index],
                b' ' | b'\t' | b'\n' | b'\r' | b'=' | b'/' | b'>'
            )
        {
            index += 1;
        }
        if index == name_start {
            break;
        }
        let attr_name = &attrs[name_start..index];
        while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            index += 1;
        }
        if bytes.get(index) != Some(&b'=') {
            continue;
        }
        index += 1;
        while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            index += 1;
        }
        let quote = bytes.get(index).copied()?;
        if quote != b'"' && quote != b'\'' {
            continue;
        }
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        if attr_name == name {
            return Some(&attrs[value_start..index]);
        }
        index += 1;
    }
    None
}

fn push_usize(output: &mut String, mut value: usize) {
    if value == 0 {
        output.push('0');
        return;
    }
    let mut digits = [0u8; 20];
    let mut len = 0usize;
    while value > 0 {
        digits[len] = (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    for digit in digits[..len].iter().rev() {
        output.push((b'0' + *digit) as char);
    }
}

fn join_slash(parts: &[&str]) -> String {
    let len = parts.iter().map(|part| part.len()).sum::<usize>() + parts.len().saturating_sub(1);
    let mut output = String::with_capacity(len);
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            output.push('/');
        }
        output.push_str(part);
    }
    output
}

fn push_joined_lines(output: &mut String, lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_id_matches_sha_prefix_and_normalizes_paths() {
        assert_eq!(
            generate_bundler_scope_id(
                "/repo/src/App.vue",
                Some("/repo"),
                false,
                Some("<template />")
            )
            .as_str(),
            "7a7a37b1"
        );
    }

    #[test]
    fn extracts_sfc_blocks_with_attrs() {
        let source = r#"
<template><img src="./logo.png"></template>
<style module="tokens" scoped src="./style.css"></style>
<i18n lang="json" src="./en.json"></i18n>
"#;
        insta::assert_debug_snapshot!(
            (
                extract_style_blocks(source, None),
                extract_custom_blocks(source, None),
                extract_src_info(source, None),
            ),
            @r###"
        (
            [
                BundlerStyleBlock {
                    content: "",
                    src: Some(
                        "./style.css",
                    ),
                    lang: None,
                    scoped: true,
                    module: true,
                    module_name: Some(
                        "tokens",
                    ),
                    index: 0,
                },
            ],
            [
                BundlerCustomBlock {
                    block_type: "i18n",
                    content: "",
                    src: Some(
                        "./en.json",
                    ),
                    attrs: [
                        SfcBlockAttribute {
                            name: "lang",
                            value: Some(
                                "json",
                            ),
                        },
                        SfcBlockAttribute {
                            name: "src",
                            value: Some(
                                "./en.json",
                            ),
                        },
                    ],
                    index: 0,
                },
            ],
            SfcSrcInfo {
                script_src: None,
                template_src: None,
            },
        )
        "###
        );
    }

    #[test]
    fn extracts_self_closing_custom_blocks() {
        let source = r#"
<template><div></div></template>
<i18n src="./en.json" />
"#;

        insta::assert_debug_snapshot!(
            extract_custom_blocks(source, None),
            @r###"
        [
            BundlerCustomBlock {
                block_type: "i18n",
                content: "",
                src: Some(
                    "./en.json",
                ),
                attrs: [
                    SfcBlockAttribute {
                        name: "src",
                        value: Some(
                            "./en.json",
                        ),
                    },
                ],
                index: 0,
            },
        ]
        "###
        );
    }

    #[test]
    fn collects_template_asset_urls() {
        let source = r#"
<template>
  <img src="./logo.png" />
  <img :src="dynamic" />
  <use href="./icons.svg#home" />
  <img src="./logo.png" />
</template>
"#;
        insta::assert_debug_snapshot!(
            collect_template_asset_urls(source, None, None),
            @r###"
        [
            TemplateAssetUrl {
                url: "./logo.png",
                var_name: "_imports_0",
            },
            TemplateAssetUrl {
                url: "./icons.svg#home",
                var_name: "_imports_1",
            },
        ]
        "###
        );
    }

    #[test]
    fn strips_css_comments_without_touching_strings() {
        let input = ".a { color: red; }\n/* :deep(.x) */\n.b::before { content: \"/* kept */\"; }";
        let output = strip_css_comments_for_scoped(input);
        assert!(!output.contains(":deep("));
        assert!(output.contains("\"/* kept */\""));
        assert_eq!(output.split('\n').count(), input.split('\n').count());
    }

    #[test]
    fn wraps_scoped_preprocessor_styles() {
        insta::assert_snapshot!(
            wrap_scoped_preprocessor_style(
                "@use \"theme\";\n.root { color: red; }",
                Some("data-v-abc"),
                Some("scss"),
            ),
            @r###"
        @use "theme";

        [data-v-abc] {
        .root { color: red; }
        }
        "###
        );
    }
}
