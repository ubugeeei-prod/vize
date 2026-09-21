//! `html/cross-component-nesting` (Davinci P4-11b): HTML nesting that only
//! exists once components are composed — `<p><MyCard /></p>` where
//! `MyCard`'s root is a `<div>`.
//!
//! Every `.vue` target's template is read as authored, each component usage
//! is resolved through the parent's own imports (never by name alone), and
//! the child template is re-checked in the chain the usage is rendered in.
//! A report is a proof: it names the usage, the element the child renders
//! and the ancestor it collides with, and carries the child's file position.

use std::path::{Path, PathBuf};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis_cf::{CrossFileAnalyzer, FileId};
use vize_patina::html_content_model::{
    ComposedFinding, Family, NodeKind, Skeleton, authored_skeleton, compose,
};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};
use vize_s0::i18n::{Locale, t, t_fmt};
use vize_s0::{Allocator, CompactString, FxHashMap, cstr, line_index::LineIndex};

pub(super) const RULE: &str = "html/cross-component-nesting";

/// One composed template: its skeleton and where the template content starts
/// in the SFC.
struct Template {
    skeleton: Skeleton,
    offset: u32,
}

fn template_of(path: &Path, source: &str) -> Option<Template> {
    let filename = path.to_string_lossy();
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_ref().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let template = descriptor.template.as_ref()?;
    let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
    Some(Template {
        skeleton: authored_skeleton(&allocator, template.content.as_ref()),
        offset: template.loc.start as u32,
    })
}

/// Push every composed finding onto the lint result of the file whose
/// template contains the outermost usage.
pub(super) fn apply<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    analyzer: &CrossFileAnalyzer,
    file_indexes: &FxHashMap<FileId, usize>,
    results: &mut [LintResult],
    help_level: HelpLevel,
) {
    let templates: Vec<Option<Template>> = files
        .iter()
        .map(|(path, source)| template_of(path, source.as_ref()))
        .collect();
    let skeletons: Vec<Skeleton> = templates
        .iter()
        .map(|template| {
            template
                .as_ref()
                .map(|t| t.skeleton.clone())
                .unwrap_or_default()
        })
        .collect();
    let ids: FxHashMap<usize, FileId> = file_indexes
        .iter()
        .map(|(file_id, index)| (*index, *file_id))
        .collect();
    let resolve = |file: u32, tag: &str| -> Option<u32> {
        let file_id = ids.get(&(file as usize))?;
        let target = analyzer.resolve_imported_component(*file_id, tag)?;
        let index = *file_indexes.get(&target)?;
        templates[index].as_ref().map(|_| index as u32)
    };
    for finding in compose(&skeletons, &resolve) {
        let Some(&(file, usage)) = finding.usages.first() else {
            continue;
        };
        let index = file as usize;
        let (Some(template), Some(result)) = (templates[index].as_ref(), results.get_mut(index))
        else {
            continue;
        };
        let diagnostic = describe(files, &templates, &finding, template, usage, help_level);
        result.diagnostics.push(diagnostic);
    }
}

/// `<tag>` for an element node, `text` otherwise; with its span.
fn node_name(skeleton: &Skeleton, node: u32) -> (CompactString, u32, u32) {
    let entry = skeleton.node(node);
    match &entry.kind {
        NodeKind::Element(element) => (
            cstr!("<{}>", element.tag),
            entry.span.start,
            element.name_span.end,
        ),
        NodeKind::Component { name } => (
            cstr!("<{}>", name),
            entry.span.start,
            entry.span.start + 1 + name.len() as u32,
        ),
        _ => (CompactString::new("text"), entry.span.start, entry.span.end),
    }
}

/// `path:line:column` of a byte offset, the path relative to the working
/// directory when it lies below it.
fn position<S: AsRef<str>>(files: &[(PathBuf, S)], file: usize, offset: u32) -> CompactString {
    let (path, source) = &files[file];
    let index = LineIndex::new(source.as_ref());
    let (line, column) = index.line_col(offset as usize);
    let cwd = std::env::current_dir().unwrap_or_default();
    let shown = path.strip_prefix(&cwd).unwrap_or(path);
    cstr!("{}:{}:{}", shown.display(), line + 1, column + 1)
}

fn describe<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    templates: &[Option<Template>],
    finding: &ComposedFinding,
    parent: &Template,
    usage: u32,
    help_level: HelpLevel,
) -> LintDiagnostic {
    let child = templates[finding.file as usize]
        .as_ref()
        .expect("findings point into composed templates");
    let (child_name, child_start, _) = node_name(&child.skeleton, finding.node);
    let rendered_by: Vec<CompactString> = finding
        .usages
        .iter()
        .filter_map(|(file, node)| {
            let skeleton = &templates[*file as usize].as_ref()?.skeleton;
            Some(node_name(skeleton, *node).0)
        })
        .collect();
    let at = position(files, finding.file as usize, child.offset + child_start);
    let subject = cstr!(
        "{} (rendered by {} at {})",
        child_name,
        rendered_by.join(" → "),
        at
    );
    let evidence = finding.evidence.and_then(|(file, node)| {
        let skeleton = &templates[file as usize].as_ref()?.skeleton;
        Some((file, node_name(skeleton, node)))
    });
    let parent_name = evidence
        .as_ref()
        .map_or(CompactString::new(""), |(_, (name, ..))| name.clone());
    let message = t_fmt(
        Locale::En,
        &cstr!("vue/permitted-contents.{}", finding.class.id()),
        &[
            ("child", subject.as_str()),
            ("parent", parent_name.as_str()),
        ],
    );
    let (_, usage_start, usage_end) = node_name(&parent.skeleton, usage);
    let mut diagnostic = LintDiagnostic::error(
        RULE,
        message,
        parent.offset + usage_start,
        parent.offset + usage_end,
    );
    if let Some((file, (name, start, end))) = evidence
        && file == finding.usages[0].0
    {
        let label = t_fmt(
            Locale::En,
            "vue/permitted-contents.evidence",
            &[("parent", name.as_str())],
        );
        diagnostic = diagnostic.with_label(label, parent.offset + start, parent.offset + end);
    }
    let help_key = match finding.class.family() {
        Family::Parser => "html/cross-component-nesting.help",
        Family::ContentModel => "vue/permitted-contents.help.content-model",
    };
    if let Some(help) = help_level.process(t(Locale::En, help_key).as_ref()) {
        diagnostic = diagnostic.with_help(help);
    }
    diagnostic
}
