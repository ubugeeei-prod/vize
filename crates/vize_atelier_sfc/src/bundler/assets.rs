use vize_armature::{PropNode, TemplateChildNode};
use vize_l0::{Allocator, String, profiler::global_profiler};

use super::blocks::parse_descriptor;

mod names;

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
    let Some(template) = descriptor.template.as_ref() else {
        return Vec::new();
    };
    let content = template.content.as_ref();
    if !names_asset_tag(content, rules) {
        return Vec::new();
    }

    // The template parser owns comments, raw text and quoting, so only real
    // elements contribute asset URLs.
    global_profiler().record_counter("sfc.asset_urls.template_parse", 1);
    let allocator = Allocator::default();
    let (root, _) = vize_armature::Parser::new(&allocator, content).parse();
    let mut urls: Vec<TemplateAssetUrl> = Vec::new();
    collect_element_asset_urls(&root.children, rules, &mut urls);
    // Namespace work is unnecessary until an actual asset import exists.
    if !urls.is_empty() {
        names::avoid_authored_names(&descriptor, &allocator, &root.children, &mut urls);
    }
    urls
}

/// Cheap admission: an asset URL needs an opening tag that a rule names.
fn names_asset_tag(template: &str, rules: Option<&[TemplateAssetTagRule]>) -> bool {
    let opens = |tag: &str| {
        template.match_indices('<').any(|(start, _)| {
            template
                .get(start + 1..start + 1 + tag.len())
                .is_some_and(|name| name.eq_ignore_ascii_case(tag))
        })
    };
    match rules {
        Some(rules) => rules.iter().any(|rule| opens(rule.tag.as_str())),
        None => DEFAULT_ASSET_URL_TAGS.iter().any(|(tag, _)| opens(tag)),
    }
}

fn collect_element_asset_urls(
    children: &[TemplateChildNode<'_>],
    rules: Option<&[TemplateAssetTagRule]>,
    urls: &mut Vec<TemplateAssetUrl>,
) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for attr in attrs_for_tag(rules, element.tag).into_iter().flatten() {
            if let Some(value) = static_attr_value(&element.props, attr)
                && is_importable_asset_url(value)
                && !urls.iter().any(|entry| entry.url.as_str() == value)
            {
                let mut var_name = String::from("_imports_");
                push_usize(&mut var_name, urls.len());
                urls.push(TemplateAssetUrl {
                    url: String::from(value),
                    var_name,
                });
            }
        }
        collect_element_asset_urls(&element.children, rules, urls);
    }
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

fn static_attr_value<'a>(props: &[PropNode<'a>], name: &str) -> Option<&'a str> {
    props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == name => attr.value.as_ref().map(|v| v.content),
        _ => None,
    })
}

fn push_usize(output: &mut String, value: usize) {
    use std::fmt::Write as _;
    let _ = write!(output, "{value}");
}
