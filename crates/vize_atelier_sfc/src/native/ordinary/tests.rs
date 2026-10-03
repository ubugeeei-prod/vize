use crate::{NativeSfcCompileOptions, NativeSfcOutput, compile_native_sfc};
use serde_json::Value;
use vize_l0::{Allocator, Span, String, cstr};
use vize_l2::lang::js::VueOrdinaryEmpty;

macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition { return Err(cstr!($($message)+)); }
    };
}

mod refusal;

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("missing {key}"))
}

#[test]
fn three_whole_ordinary_empty_modules_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_sfc_ordinary_empty_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("fixtures")?;
    require!(fixtures.len() == 3, "three whole original inputs");
    let mut rows = Vec::new();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let plain = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        let mapped = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "Ordinary雪🌸.vue",
                source_map: true,
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
        let unlinked = plain
            .result()
            .map_err(|error| cstr!("{id}: plain {error:?}"))?;
        require!(
            unlinked.code() == output.code(),
            "{id}: actual NoLinks/Recorded bytes"
        );
        require!(
            unlinked.document().links().is_empty(),
            "{id}: actual NoLinks"
        );
        require!(unlinked.source_map().is_none(), "{id}: optional map");
        require!(
            output.css().is_none() && output.css_source_map().is_none(),
            "{id}: no styles"
        );
        let map: Value = serde_json::from_str(output.source_map().ok_or("actual map")?)
            .map_err(|error| cstr!("{error}"))?;
        require!(
            map.get("sourcesContent") == Some(&serde_json::json!([source])),
            "{id}: complete original map source"
        );
        require!(
            map.get("names") == Some(&serde_json::json!([])),
            "{id}: zero bindings"
        );
        // Initial hosted capture precedes frozen native fields. Queue admission
        // requires those source-generated fields and unconditional comparisons.
        if let Some(code) = fixture.get("code").and_then(Value::as_str) {
            require!(output.code() == code, "{id}: whole frozen native module");
            require!(
                fixture.get("nativeMap") == Some(&map),
                "{id}: whole frozen native map"
            );
        }
        let owner = mapped.observation();
        let admitted = owner.admitted().ok_or("original admitted SFC")?;
        let file = admitted.file().file();
        require!(file.is_complete(), "{id}: actual complete File");
        require!(
            core::ptr::eq(file.artifact().source(), source),
            "{id}: original root source"
        );
        let descriptor = owner.descriptor().admitted().map_err(|_| "descriptor")?;
        require!(
            descriptor.setup().is_none(),
            "{id}: actual no setup sibling"
        );
        let selected = descriptor.ordinary().ok_or("selected ordinary")?;
        let script = owner.scripts().first().ok_or("retained script")?;
        require!(owner.scripts().len() == 1, "{id}: sole original script");
        require!(
            script.block().source() == text(fixture, "script")?,
            "{id}: original script bytes"
        );
        let syntax = script.syntax().ok_or("original syntax")?;
        require!(
            syntax.diagnostics().count() == 0,
            "{id}: original diagnostics"
        );
        require!(
            core::ptr::eq(syntax.source().text(), selected.block().source()),
            "{id}: retained original block pointer"
        );
        let program = syntax.admitted_program().ok_or("original Program")?;
        let original = program.program();
        let original_pointer = original as *const _ as usize;
        let ordinary = VueOrdinaryEmpty::checked(file, selected, program)
            .map_err(|error| cstr!("{id}: {error:?}"))?;
        require!(
            core::ptr::eq(ordinary.program().program(), original),
            "{id}: exact original Program"
        );
        require!(
            file.bindings().count() == 0
                && file.imports().is_empty()
                && file.references().is_empty(),
            "{id}: zero script or template context reads"
        );
        require!(file.exports().len() == 1, "{id}: actual sole export row");
        let block = ordinary.source();
        let prefix = Span::new(
            ordinary.statement_span().start,
            ordinary.object_span().start,
        );
        require!(
            !output
                .document()
                .links()
                .iter()
                .any(|link| link.authored.start < prefix.end && link.authored.end > prefix.start),
            "{id}: rewritten export/default bytes have no copied link"
        );
        if block.start() != prefix.start {
            require_segment(output, source, Span::new(block.start(), prefix.start), id)?;
        }
        require_segment(output, source, Span::new(prefix.end, block.end()), id)?;
        let comments = syntax.comments().count();
        let (retained, result) = mapped.into_parts();
        require!(result.is_ok(), "{id}: actual output after handoff");
        let moved = Box::new(retained);
        let moved_syntax = moved
            .scripts()
            .first()
            .and_then(|script| script.syntax())
            .ok_or("moved syntax")?;
        require!(
            moved_syntax.program().ok_or("moved Program")? as *const _ as usize == original_pointer,
            "{id}: original AST survives owner move"
        );
        require!(
            moved_syntax.comments().count() == comments,
            "{id}: original comment custody after rewrite"
        );
        rows.push(serde_json::json!({"id":id,"source":source,"code":result.as_ref().map_err(|_| "result")?.code(),"nativeMap":map,"bindings":[]}));
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SFC_ORDINARY_EMPTY_CAPTURE") {
        let capture = serde_json::json!({"schema":"vize.native-sfc.ordinary-empty-capture","adapter":"vize_atelier_sfc::compile_native_sfc","fixtures":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}

fn require_segment(
    output: &NativeSfcOutput,
    source: &str,
    span: Span,
    id: &str,
) -> Result<(), String> {
    let mut links = output
        .document()
        .links()
        .iter()
        .filter(|link| link.name.is_none() && link.authored == span);
    let link = links
        .next()
        .ok_or_else(|| cstr!("{id}: exact original copied segment"))?;
    require!(links.next().is_none(), "{id}: segment copied exactly once");
    require!(
        source.get(span.start as usize..span.end as usize)
            == output
                .code()
                .get(link.generated.start as usize..link.generated.end as usize),
        "{id}: byte-exact original segment"
    );
    Ok(())
}
