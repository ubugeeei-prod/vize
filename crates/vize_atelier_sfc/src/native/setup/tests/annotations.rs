use super::text;
use crate::{NativeSfcCompileOptions, compile_native_sfc};
use serde_json::Value;
use vize_l0::{Allocator, Span, String, cstr};
use vize_l2::lang::js::VueSetup;

#[test]
fn three_source_owned_annotation_modules_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("original fixtures")?;
    require!(fixtures.len() == 3, "three complete original inputs");
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
                filename: "Setup雪🌸.vue",
                source_map: true,
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
        let plain_output = plain
            .result()
            .map_err(|error| cstr!("{id}: plain {error:?}"))?;
        require!(
            plain_output.code() == output.code(),
            "{id}: actual NoLinks/Recorded bytes"
        );
        require!(
            plain_output.document().links().is_empty(),
            "{id}: genuine NoLinks"
        );
        require!(plain_output.source_map().is_none(), "{id}: optional map");
        let map: Value = serde_json::from_str(output.source_map().ok_or("actual map")?)
            .map_err(|error| cstr!("{error}"))?;
        require!(
            map.get("sourcesContent") == Some(&serde_json::json!([source])),
            "{id}: complete original map source"
        );
        let owner = mapped.observation();
        let admitted = owner.admitted().ok_or("actual original native SFC")?;
        let file = admitted.file().file();
        require!(file.is_complete(), "{id}: actual complete File");
        require!(
            core::ptr::eq(file.artifact().source(), source),
            "{id}: original root"
        );
        let script = owner.scripts().first().ok_or("original script")?;
        require!(owner.scripts().len() == 1, "{id}: one original script");
        require!(
            script.block().source() == text(fixture, "script")?,
            "{id}: original annotated body"
        );
        let syntax = script.syntax().ok_or("retained original syntax")?;
        require!(
            syntax.diagnostics().count() == 0,
            "{id}: original diagnostics"
        );
        require!(
            core::ptr::eq(syntax.source().text(), script.block().source()),
            "{id}: original parsed block pointer"
        );
        let selected = owner
            .descriptor()
            .admitted()
            .map_err(|_| "descriptor")?
            .setup()
            .ok_or("true setup")?;
        let program = syntax.admitted_program().ok_or("original Program")?;
        require!(
            program.source_type().is_typescript(),
            "{id}: actual TS profile"
        );
        let setup =
            VueSetup::checked(file, selected, program).map_err(|error| cstr!("{id}: {error:?}"))?;
        let expected = fixture
            .get("bindings")
            .and_then(Value::as_array)
            .ok_or("bindings")?;
        require!(
            setup.bindings().count() == expected.len(),
            "{id}: complete binding set"
        );
        for (binding, name) in setup.bindings().zip(expected) {
            require!(
                core::ptr::eq(binding.file(), file),
                "{id}: same binding owner"
            );
            let declaration = binding.declaration().ok_or("original declaration")?;
            require!(
                Some(declaration.name.as_str()) == name.as_str(),
                "{id}: genuine declaration order"
            );
            require!(
                declaration.is_direct_program() && declaration.scope == setup.exposure().scope(),
                "{id}: actual root scope"
            );
            require!(
                declaration.initializer == vize_l2::file::InitializerKind::PrimitiveLiteral,
                "{id}: actual primitive initializer"
            );
            let immutable = fixture
                .get("immutableBindings")
                .and_then(Value::as_array)
                .ok_or("immutable bindings")?;
            require!(
                (declaration.kind == vize_l2::file::DeclarationKind::Const)
                    == immutable.contains(name),
                "{id}: original immutable class"
            );
        }
        require!(
            Some(setup.type_annotations().count() as u64)
                == fixture.get("annotationCount").and_then(Value::as_u64),
            "{id}: genuine annotation count"
        );
        let block = setup.source();
        let mut cursor = block.start();
        for annotation in setup.type_annotations() {
            require!(
                core::ptr::eq(annotation.file(), file),
                "{id}: same annotation owner"
            );
            let span = annotation.span();
            require!(
                block.contains_block_span(span) && span.start >= cursor,
                "{id}: ordered checked original span"
            );
            if cursor != span.start {
                require_segment(output, source, Span::new(cursor, span.start), id)?;
            }
            require!(
                !output
                    .document()
                    .links()
                    .iter()
                    .any(|link| link.name.is_none()
                        && link.authored.start < span.end
                        && link.authored.end > span.start),
                "{id}: annotation bytes have no runtime link"
            );
            cursor = span.end;
        }
        require_segment(output, source, Span::new(cursor, block.end()), id)?;
        rows.push(serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map,"bindings":expected}));
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SFC_TS_ANNOTATION_SETUP_CAPTURE") {
        let capture = serde_json::json!({"schema":"vize.native-sfc.ts_annotation-setup-capture","adapter":"vize_atelier_sfc::compile_native_sfc","fixtures":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}

fn require_segment(
    output: &crate::NativeSfcOutput,
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
        .ok_or_else(|| cstr!("{id}: copied original runtime segment"))?;
    require!(
        links.next().is_none(),
        "{id}: original runtime segment copied exactly once"
    );
    require!(
        source.get(span.start as usize..span.end as usize)
            == output
                .code()
                .get(link.generated.start as usize..link.generated.end as usize),
        "{id}: byte-exact linked original segment"
    );
    Ok(())
}
