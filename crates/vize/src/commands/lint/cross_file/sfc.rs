//! SFC parsing for the cross-file lint lane: the Croquis analysis of each
//! `.vue` target and the offsets its script- and template-relative
//! diagnostics shift by.

use std::path::Path;
use vize_armature::Parser;
use vize_atelier_sfc::{
    SfcParseOptions,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor},
    parse_sfc,
};
use vize_croquis::Croquis;
use vize_l0::Allocator;
use vize_relief::{PropNode, TemplateChildNode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FallthroughRoot {
    Teleport,
    Fragment,
    Text,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CrossFileSourceOffsets {
    pub(super) script: u32,
    pub(super) template: u32,
    pub(super) fallthrough_root: Option<FallthroughRoot>,
}

pub(super) fn analyze_sfc_for_cross_file(
    source: &str,
    path: &Path,
) -> Option<(Croquis, CrossFileSourceOffsets)> {
    let filename = path.to_string_lossy();
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_ref().into(),
            ..Default::default()
        },
    )
    .ok()?;

    let mut offsets = CrossFileSourceOffsets::default();

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        offsets.script = if descriptor.script.is_some() {
            descriptor
                .script
                .as_ref()
                .map(|script| script.loc.start as u32)
                .unwrap_or(script_setup.loc.start as u32)
        } else {
            script_setup.loc.start as u32
        };
    } else if let Some(script) = descriptor.script.as_ref() {
        offsets.script = script.loc.start as u32;
    }

    let analysis = if let Some(template) = descriptor.template.as_ref() {
        offsets.template = template.loc.start as u32;
        let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
        let parser = Parser::new(&allocator, template.content.as_ref());
        let (root, parse_errors) = parser.parse();
        let template_ast = if parse_errors.iter().any(|error| !error.is_recoverable()) {
            None
        } else {
            offsets.fallthrough_root = fallthrough_root(&root);
            Some(&root)
        };
        analyze_sfc_descriptor(&descriptor, template_ast, SfcCroquisOptions::full())
    } else {
        analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full())
    };

    Some((analysis, offsets))
}

fn fallthrough_root(root: &vize_relief::RootNode<'_>) -> Option<FallthroughRoot> {
    let mut significant = root
        .children
        .iter()
        .filter(|child| match child {
            TemplateChildNode::Comment(_) => false,
            TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
            _ => true,
        })
        .peekable();
    let first = significant.next()?;
    if has_root_directive(first, &["if"]) {
        while significant
            .peek()
            .is_some_and(|child| has_root_directive(child, &["else-if", "else"]))
        {
            significant.next();
        }
        return significant.next().map(|_| FallthroughRoot::Fragment);
    }
    if significant.next().is_some() {
        return Some(FallthroughRoot::Fragment);
    }
    match first {
        TemplateChildNode::Element(_) if has_root_directive(first, &["for"]) => {
            Some(FallthroughRoot::Fragment)
        }
        TemplateChildNode::Element(element) if element.tag.eq_ignore_ascii_case("teleport") => {
            Some(FallthroughRoot::Teleport)
        }
        // The active branch and the actual rendered root are not known here.
        TemplateChildNode::If(_) | TemplateChildNode::IfBranch(_) => None,
        TemplateChildNode::Element(element)
            if element.tag.eq_ignore_ascii_case("suspense")
                || element.tag.eq_ignore_ascii_case("keepalive") =>
        {
            None
        }
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => {
            Some(FallthroughRoot::Text)
        }
        TemplateChildNode::For(_) => Some(FallthroughRoot::Fragment),
        _ => None,
    }
}

fn has_root_directive(child: &TemplateChildNode<'_>, names: &[&str]) -> bool {
    matches!(child, TemplateChildNode::Element(element) if element.props.iter().any(|prop| {
        matches!(prop, PropNode::Directive(directive) if names.contains(&directive.name))
    }))
}
