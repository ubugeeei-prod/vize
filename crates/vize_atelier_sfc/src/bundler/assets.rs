use vize_carton::String;

use super::blocks::parse_descriptor;

const DEFAULT_ASSET_URL_TAGS: &[(&str, &[&str])] = &[
    ("img", &["src"]),
    ("video", &["src", "poster"]),
    ("source", &["src"]),
    ("image", &["xlink:href", "href"]),
    ("use", &["xlink:href", "href"]),
];

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
