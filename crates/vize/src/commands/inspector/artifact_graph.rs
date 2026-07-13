//! Shared Atlas artifacts for inspector compilation and comparison.

use std::time::Instant;

use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileProduct, SfcCompileRequest,
    SfcCompileSettings, SfcDescriptorProduct, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions,
};
#[cfg(test)]
use vize_atlas::QueryOutcome;
use vize_atlas::{Compilation, CompilationSnapshot, SourceId};
use vize_carton::{FxHashMap, String, cstr};
use vize_relief::TemplateSyntaxMode;

use super::{InspectorArgs, InspectorTarget};
use crate::commands::inspector::curator_inspector;

pub(super) struct VizeCompilerRun {
    pub(super) code: String,
    pub(super) warnings: Vec<String>,
    pub(super) error: Option<String>,
    pub(super) time_ms: f64,
}

#[derive(Clone, Copy)]
struct InspectorCompileOptions {
    target: InspectorTarget,
    custom_renderer: bool,
    template_syntax: TemplateSyntaxMode,
}

pub(super) struct InspectorArtifactGraph {
    snapshot: CompilationSnapshot,
    sources: FxHashMap<String, SourceId>,
}

impl InspectorArtifactGraph {
    pub(super) fn new(
        files: &[curator_inspector::InspectorSourceFile],
        args: &InspectorArgs,
    ) -> Result<Self, String> {
        let options = InspectorCompileOptions {
            target: args.target,
            custom_renderer: args.custom_renderer,
            template_syntax: args.template_syntax.into(),
        };
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        vize_atelier_dom::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        vize_atelier_ssr::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        vize_atelier_vapor::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;

        let mut sources = FxHashMap::default();
        let mut settings = SfcCompileSettings::default();
        for file in files {
            let source = compilation
                .add_source(file.path.as_str(), file.source.as_str())
                .map_err(|error| cstr!("{error}"))?;
            settings.insert(source, compile_request(file, options));
            sources.insert(file.path.clone(), source);
        }
        settings
            .install(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        Ok(Self {
            snapshot: compilation.snapshot(),
            sources,
        })
    }

    #[cfg(test)]
    fn query(
        &self,
        file: &curator_inspector::InspectorSourceFile,
    ) -> Result<QueryOutcome<SfcCompileProduct>, String> {
        let source = self
            .sources
            .get(&file.path)
            .copied()
            .ok_or_else(|| cstr!("inspector source is not registered: {}", file.path))?;
        self.snapshot
            .query_session()
            .query::<SfcCompileProduct>(source)
            .map_err(|error| cstr!("{error}"))
    }

    pub(super) fn compile(&self, file: &curator_inspector::InspectorSourceFile) -> VizeCompilerRun {
        let start = Instant::now();
        let source = self.sources[&file.path];
        let mut session = self.snapshot.query_session();
        let result = match session.query::<SfcDescriptorProduct>(source) {
            Ok(descriptor) => match descriptor.value().diagnostic() {
                Some(error) => Err(error.message.clone()),
                None => session
                    .query::<SfcCompileProduct>(source)
                    .map(|outcome| outcome.value().clone())
                    .map_err(|error| cstr!("{error}")),
            },
            Err(error) => Err(cstr!("{error}")),
        };
        match result {
            Ok(result) => VizeCompilerRun {
                code: result.code,
                warnings: result
                    .warnings
                    .into_iter()
                    .map(|warning| warning.message)
                    .collect(),
                error: format_sfc_errors(result.errors),
                time_ms: elapsed_ms(start),
            },
            Err(error) => VizeCompilerRun {
                code: String::default(),
                warnings: Vec::new(),
                error: Some(error),
                time_ms: elapsed_ms(start),
            },
        }
    }
}

fn compile_request(
    file: &curator_inspector::InspectorSourceFile,
    options: InspectorCompileOptions,
) -> SfcCompileRequest {
    let filename = file.path.clone();
    let is_ts = source_uses_type_script(&file.source);
    let compile = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            ssr: matches!(options.target, InspectorTarget::Ssr),
            is_prod: true,
            is_ts,
            custom_renderer: options.custom_renderer,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            ..Default::default()
        },
        vapor: matches!(options.target, InspectorTarget::Vapor),
        ..Default::default()
    };
    SfcCompileRequest::new(compile, options.template_syntax).with_inferred_scoped_from_descriptor()
}

fn source_uses_type_script(source: &str) -> bool {
    let mut rest = source;
    while let Some(start) = rest.find("<script") {
        rest = &rest[start + "<script".len()..];
        let Some(end) = opening_tag_end(rest) else {
            return false;
        };
        if script_tag_has_ts_lang(&rest[..end]) {
            return true;
        }
        rest = &rest[end + 1..];
    }
    false
}

fn opening_tag_end(source: &str) -> Option<usize> {
    let mut quote = None;
    for (index, byte) in source.bytes().enumerate() {
        match (quote, byte) {
            (Some(expected), current) if expected == current => quote = None,
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(index),
            _ => {}
        }
    }
    None
}

fn script_tag_has_ts_lang(attributes: &str) -> bool {
    let bytes = attributes.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = attributes[cursor..].find("lang") {
        let start = cursor + relative;
        let before_is_name = start > 0 && is_attr_name_byte(bytes[start - 1]);
        let after = start + "lang".len();
        let after_is_name = bytes
            .get(after)
            .is_some_and(|byte| is_attr_name_byte(*byte));
        if before_is_name || after_is_name {
            cursor = after;
            continue;
        }
        let mut value = after;
        while bytes.get(value).is_some_and(u8::is_ascii_whitespace) {
            value += 1;
        }
        if bytes.get(value) != Some(&b'=') {
            cursor = after;
            continue;
        }
        value += 1;
        while bytes.get(value).is_some_and(u8::is_ascii_whitespace) {
            value += 1;
        }
        let quote = bytes
            .get(value)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        value += usize::from(quote.is_some());
        let end = attributes[value..]
            .find(|character: char| {
                quote.map_or_else(
                    || character.is_ascii_whitespace(),
                    |quote| character == quote as char,
                )
            })
            .map_or(attributes.len(), |relative| value + relative);
        return matches!(&attributes[value..end], "ts" | "tsx");
    }
    false
}

fn is_attr_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn format_sfc_errors(errors: Vec<vize_atelier_sfc::SfcError>) -> Option<String> {
    (!errors.is_empty()).then(|| {
        errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    })
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[cfg(test)]
#[path = "artifact_graph/tests.rs"]
mod tests;
