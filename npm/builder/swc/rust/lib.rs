//! SWC host adapter. JSX semantics remain in vize_atelier_jsx.

#[expect(
    clippy::disallowed_methods,
    reason = "SWC plugin_transform generates ABI exports containing ToString calls"
)]
mod entry;
mod maps;
mod options;

use swc_core::{
    common::{
        BytePos, DUMMY_SP, Mark, Spanned,
        comments::{Comments, SingleThreadedComments},
        errors::HANDLER,
        sync::Lrc,
    },
    ecma::{
        ast::{JSXElement, JSXFragment, Program},
        codegen::{Emitter, text_writer::JsWriter},
        parser::{Parser, StringInput, Syntax, TsSyntax},
        transforms::base::{hygiene::hygiene, resolver},
        visit::{Visit, VisitMutWith, VisitWith},
    },
    plugin::proxies::TransformPluginProgramMetadata,
};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_carton::{Allocator, String, cstr};

fn transform(
    program: &mut Program,
    metadata: &TransformPluginProgramMetadata,
) -> Result<Option<Program>, String> {
    let options: options::Options = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .unwrap_or_else(|| "{}".into()),
    )
    .map_err(|error| cstr!("Invalid Vize SWC options: {error}"))?;
    let mut finder = JsxFinder::default();
    program.visit_with(&mut finder);
    if !finder.0 {
        return Ok(None);
    }

    // Printing resolves any renamed bindings from earlier plugins before the
    // generated AST receives fresh resolver contexts.
    program.mutate(hygiene());
    let mut bytes = Vec::new();
    let mut positions = Vec::new();
    #[expect(
        clippy::disallowed_types,
        reason = "SWC Emitter requires its Lrc-owned source mapper at the host boundary"
    )]
    let cm = Lrc::new(metadata.source_map.clone());
    let comments = metadata
        .comments
        .as_ref()
        .map(|value| value as &dyn Comments);
    Emitter {
        cfg: Default::default(),
        cm: cm.clone(),
        comments,
        wr: JsWriter::new(Default::default(), "\n", &mut bytes, Some(&mut positions)),
    }
    .emit_program(program)
    .map_err(|error| cstr!("Cannot print SWC input: {error}"))?;
    let source =
        std::str::from_utf8(&bytes).map_err(|error| cstr!("Invalid SWC source: {error}"))?;
    let mut config = JsxCompileConfig {
        default_mode: options.jsx_mode.into(),
        compat: options.jsx_compat.into(),
        ssr: options.ssr,
        ..Default::default()
    };
    config.vdom.source_map = true;
    let allocator = Allocator::new();
    let output = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
    for diagnostic in &output.diagnostics {
        HANDLER.with(|handler| {
            if diagnostic.is_error() {
                handler
                    .struct_span_err(program.span(), &diagnostic.message)
                    .emit();
            } else {
                handler
                    .struct_span_warn(program.span(), &diagnostic.message)
                    .emit();
            }
        });
    }
    if output.has_errors() {
        return Ok(None);
    }
    if output
        .components
        .iter()
        .any(|component| component.scoped_style().is_some())
    {
        return Err("Scoped JSX CSS requires a bundler CSS consumer; SWC Wasm plugins cannot emit stylesheets".into());
    }
    let code = output.module_code();
    let parsed_comments = SingleThreadedComments::default();
    let end = u32::try_from(code.len())
        .ok()
        .and_then(|len| len.checked_add(1))
        .ok_or_else(|| String::from("Vize output exceeds the SWC source size limit"))?;
    let mut parser = Parser::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        }),
        StringInput::new(&code, BytePos(1), BytePos(end)),
        Some(&parsed_comments),
    );
    let mut generated = parser
        .parse_program()
        .map_err(|error| cstr!("Invalid Vize output: {:?}", error.kind()))?;
    if let Some(error) = parser.take_errors().first() {
        return Err(cstr!("Invalid Vize output: {:?}", error.kind()));
    }
    let mut remaining = JsxFinder::default();
    generated.visit_with(&mut remaining);
    if remaining.0 {
        return Err("Vize did not return a complete JSX-free module".into());
    }
    let map = output
        .source_map()
        .ok_or_else(|| String::from("Vize did not return a source map"))?;
    let mut remapper = maps::Remapper::new(&code, map, positions)?;
    generated.visit_mut_with(&mut remapper);
    // Generated helper imports have no authored origin. Keep the enclosing
    // program's original boundary so SWC's module pass can carry file-leading
    // comments onto its generated directives.
    if let Program::Module(module) = &mut generated {
        module.span = program.span();
    } else if let Program::Script(script) = &mut generated {
        script.span = program.span();
    } else {
        return Err("Unsupported SWC program kind".into());
    }
    // The host resolver has already run. Re-resolve the newly parsed module so
    // SWC's CommonJS and target passes recognize its imports and local bindings.
    generated.mutate(resolver(metadata.unresolved_mark, Mark::new(), true));
    if let Some(host) = comments {
        let mut old = maps::CommentPositions::default();
        program.visit_with(&mut old);
        for pos in old.0 {
            host.take_leading(pos);
            host.take_trailing(pos);
        }
        let (leading, trailing) = parsed_comments.take_all();
        for (is_leading, entries) in [(true, leading), (false, trailing)] {
            for (pos, list) in entries.borrow_mut().drain() {
                let mapped = remapper.position(pos);
                if mapped == DUMMY_SP.lo {
                    continue;
                }
                let list = list
                    .into_iter()
                    .map(|mut comment| {
                        comment.span = remapper.span(comment.span);
                        comment
                    })
                    .collect();
                if is_leading {
                    host.add_leading_comments(mapped, list);
                } else {
                    host.add_trailing_comments(mapped, list);
                }
            }
        }
    }
    Ok(Some(generated))
}

#[derive(Default)]
struct JsxFinder(bool);
impl Visit for JsxFinder {
    fn visit_jsx_element(&mut self, _: &JSXElement) {
        self.0 = true;
    }
    fn visit_jsx_fragment(&mut self, _: &JSXFragment) {
        self.0 = true;
    }
}
