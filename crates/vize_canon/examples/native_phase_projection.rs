//! Source-only projection evidence; this executable never requests native diagnostics.
//!
//! The document view uses main's public editor API with unchanged defaults and
//! an inline preamble. It is a labeled variant, not the batch CLI's emitted code.
//! Imported runtime prop facts may still read provider files relative to the real
//! source path; the caller must freeze the complete corpus and dependency inputs.
//! The mapper view uses its own unchanged defaults and protocol link filtering.

use std::ops::Range;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_canon::batch::{
    ImportRewriter, VueDocumentVirtualTs, VueDocumentVirtualTsOptions,
    generate_vue_content_mapper_transform, generate_vue_document_virtual_ts_with_options,
};
use vize_canon::virtual_ts::{VirtualTsOptions, VizeSemanticLinkKind};
use vize_l0::cstr;

type ProbeResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> ProbeResult<()> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("missing corpus directory")?,
    );
    let mut paths = Vec::new();
    collect_files(&root, &mut paths)?;
    paths.sort();
    require(!paths.is_empty(), "projection corpus contains no Vue files")?;
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let source = std::fs::read_to_string(&path)?;
        let document = document(&path, &source)?;
        files.push(json!({
            "file": path.strip_prefix(&root)?.to_string_lossy(),
            "document": document_json(&document),
            "mapper": generate_vue_content_mapper_transform(&path, &source)?,
        }));
    }
    let document_options = VueDocumentVirtualTsOptions::default();
    println!(
        "{}",
        serde_json::to_string(&json!({
            "variant": "public-document-defaults-and-public-mapper-defaults",
            "documentHoistsSharedPreamble": false,
            "documentOptions": {
                "optionsApi": document_options.options_api,
                "legacyVue2": document_options.legacy_vue2,
                "experimentalPatternedTemplate": document_options.experimental_patterned_template,
                "preserveEventNavigation": document_options.preserve_event_navigation,
                "preserveMissingVueDiagnostics": document_options.preserve_missing_vue_diagnostics,
                "dialect": cstr!("{:?}", document_options.dialect),
            },
            "virtualTsDefaultsDebug": cstr!("{:?}", VirtualTsOptions::default()),
            "files": files,
            "mappingFreshness": mapping_freshness(&root)?,
        }))?
    );
    Ok(())
}

fn collect_files(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            if !matches!(entry.file_name().to_str(), Some("node_modules" | ".git")) {
                collect_files(&path, paths)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            paths.push(path);
        }
    }
    Ok(())
}

fn document(path: &Path, source: &str) -> ProbeResult<VueDocumentVirtualTs> {
    let document = generate_vue_document_virtual_ts_with_options(
        path,
        source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
        VueDocumentVirtualTsOptions::default(),
    )?;
    for row in document.mapping.rows() {
        checked_range(source, &row.span.src_range)?;
        checked_range(&document.pre_rewrite_code, &row.span.gen_range)?;
        for span in &row.span.sub_spans {
            checked_range(source, &span.src_range)?;
            checked_range(&document.pre_rewrite_code, &span.gen_range)?;
        }
    }
    for link in document.mapping.semantic_links() {
        checked_range(&document.pre_rewrite_code, &link.source_range)?;
        checked_range(&document.pre_rewrite_code, &link.target_range)?;
    }
    Ok(document)
}

fn document_json(document: &VueDocumentVirtualTs) -> Value {
    let rows: Vec<_> = document
        .mapping
        .rows()
        .map(|row| {
            let sub_spans: Vec<_> = row
                .span
                .sub_spans
                .iter()
                .map(|span| json!({ "generated": span.gen_range, "authored": span.src_range }))
                .collect();
            json!({
                "generated": row.span.gen_range,
                "authored": row.span.src_range,
                "subSpans": sub_spans,
                "features": row.meta.features.bits(),
                "kind": row.meta.kind as u8,
            })
        })
        .collect();
    let semantic_links: Vec<_> = document
        .mapping
        .semantic_links()
        .iter()
        .map(|link| {
            let kind = match link.kind {
                VizeSemanticLinkKind::VueSetupTemplateRefUnwrap => "VueSetupTemplateRefUnwrap",
                VizeSemanticLinkKind::VuePlainScriptExport => "VuePlainScriptExport",
                VizeSemanticLinkKind::VueOptionsApiBinding => "VueOptionsApiBinding",
                VizeSemanticLinkKind::VueComponentPropNavigation => "VueComponentPropNavigation",
                VizeSemanticLinkKind::VueComponentPropCompletion => "VueComponentPropCompletion",
                VizeSemanticLinkKind::VueTemplatePropBinding => "VueTemplatePropBinding",
                VizeSemanticLinkKind::VueSetupImportSpecialization => {
                    "VueSetupImportSpecialization"
                }
            };
            json!({ "source": link.source_range, "target": link.target_range, "kind": kind })
        })
        .collect();
    json!({
        "code": document.code,
        "preRewriteCode": document.pre_rewrite_code,
        "virtualSuffix": document.virtual_suffix,
        "sourceType": {
            "typescript": document.source_type.is_typescript(),
            "jsx": document.source_type.is_jsx(),
            "module": document.source_type.is_module(),
        },
        "sourceTypeDebug": cstr!("{:?}", document.source_type),
        "mapping": {
            "authoredBase": document.mapping.authored_base(),
            "rows": rows,
            "semanticLinks": semantic_links,
        },
        // No public adjustment iterator exists. The derived Debug form includes
        // every original_offset/adjustment, in producer order, without sampling.
        "importSourceMapDebug": cstr!("{:?}", document.import_source_map),
    })
}

fn mapping_freshness(root: &Path) -> ProbeResult<Value> {
    const SOURCE: &str = "<script setup lang=\"ts\">const title = '🌸';</script>\n<template>{{ title }}</template>\n";
    const PREFIX: &str = "<!-- 🌸 -->\n";
    let path = root.join("__MappingFreshness.vue");
    let before = document(&path, SOURCE)?;
    let authored = SOURCE
        .rfind("title")
        .ok_or("freshness fixture has no template anchor")?;
    let generated = before
        .mapping
        .to_generated(authored)
        .ok_or("anchor is not mapped")?;
    require(
        before.mapping.to_authored(generated) == Some(authored),
        "original anchor mapping differs",
    )?;
    require(
        before
            .pre_rewrite_code
            .get(generated..generated + "title".len())
            == Some("title"),
        "original anchor does not name the generated template read",
    )?;

    let shifted_source = cstr!("{PREFIX}{SOURCE}");
    let shifted = document(&path, &shifted_source)?;
    let shifted_authored = authored + PREFIX.len();
    // The producer includes absolute authored offsets in @vize-map comments.
    // Prefix edits legitimately change those bytes and may move later code.
    let shifted_generated = shifted
        .mapping
        .to_generated(shifted_authored)
        .ok_or("shifted anchor is not mapped")?;
    require(
        shifted.mapping.to_authored(shifted_generated) == Some(shifted_authored)
            && shifted
                .pre_rewrite_code
                .get(shifted_generated..shifted_generated + "title".len())
                == Some("title"),
        "same-path UTF-8 edit retained stale mapping coordinates",
    )?;

    let renamed_source = SOURCE.replace("title", "label");
    let renamed = document(&path, &renamed_source)?;
    require(
        renamed_source.len() == SOURCE.len(),
        "rename changed fixture byte length",
    )?;
    require(
        renamed.code != before.code && renamed.code.contains("const label"),
        "same-length edit retained stale generated source",
    )?;
    let renamed_generated = renamed
        .mapping
        .to_generated(authored)
        .ok_or("renamed anchor is not mapped")?;
    require(
        renamed.mapping.to_authored(renamed_generated) == Some(authored)
            && renamed
                .pre_rewrite_code
                .get(renamed_generated..renamed_generated + "label".len())
                == Some("label"),
        "same-length edit retained a stale template-read mapping",
    )?;
    let repeated = document(&path, SOURCE)?;
    require(
        document_json(&repeated) == document_json(&before),
        "returning to original source changed projection",
    )?;
    Ok(json!({
        "samePath": true,
        "utf8PrefixBytes": PREFIX.len(),
        "originalAuthored": authored,
        "shiftedAuthored": shifted_authored,
        "generated": generated,
        "shiftedGenerated": shifted_generated,
        "renamedGenerated": renamed_generated,
        "sameLengthRename": true,
        "originalRestored": true,
    }))
}

fn checked_range(text: &str, range: &Range<usize>) -> ProbeResult<()> {
    require(
        text.get(range.clone()).is_some(),
        "projection range is outside UTF-8 source bounds",
    )
}

fn require(condition: bool, message: &'static str) -> ProbeResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
