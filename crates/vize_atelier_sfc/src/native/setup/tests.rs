use crate::{NativeSfcCompileOptions, compile_native_sfc};
use serde_json::Value;
use vize_l0::{Allocator, String, cstr};

macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition { return Err(cstr!($($message)+)); }
    };
}

mod annotations;
mod constants;
mod refusal;
mod strict;
mod typescript;

fn pack() -> Result<Value, String> {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_sfc_js_setup_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("missing {key}"))
}

#[test]
fn eight_whole_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures()
-> Result<(), String> {
    capture_pack(
        pack()?,
        8,
        "VIZE_NATIVE_SFC_SETUP_CAPTURE",
        "vize.native-sfc.js-setup-capture",
    )
}

fn capture_pack(pack: Value, count: usize, capture_env: &str, schema: &str) -> Result<(), String> {
    let fixtures = pack
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("fixtures")?;
    require!(fixtures.len() == count, "{count} whole original inputs");
    let mut capture = Vec::new();
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
            output.code() == text(fixture, "code")?,
            "{id}: complete recorded code"
        );
        require!(
            plain_output.code() == output.code(),
            "{id}: identical NoLinks bytes"
        );
        require!(plain_output.document().links().is_empty(), "{id}: NoLinks");
        require!(plain_output.source_map().is_none(), "{id}: optional map");
        let map: Value = serde_json::from_str(output.source_map().ok_or("actual map")?)
            .map_err(|error| cstr!("{error}"))?;
        require!(Some(&map) == fixture.get("nativeMap"), "{id}: complete map");
        let observation = mapped.observation();
        let admitted = observation.admitted().ok_or("actual native SFC")?;
        let script = observation.scripts().first().ok_or("script owner")?;
        let original = script.syntax().ok_or("syntax owner")?;
        let file = admitted.file().file();
        require!(file.is_complete(), "{id}: completed original File");
        require!(
            core::ptr::eq(file.artifact().source(), source),
            "{id}: same root source"
        );
        require!(
            core::ptr::eq(original.source().text(), script.block().source()),
            "{id}: actual script span"
        );
        require!(
            script.block().source() == text(fixture, "script")?,
            "{id}: authored script unchanged"
        );
        require!(
            observation.scripts().len() == 1,
            "{id}: exact one original setup owner"
        );
        let block = script.block();
        let copied = output
            .document()
            .links()
            .iter()
            .find(|link| link.authored == block.span())
            .ok_or("copied original script link")?;
        let generated = output
            .code()
            .get(copied.generated.start as usize..copied.generated.end as usize)
            .ok_or("checked generated source")?;
        require!(
            generated == block.source(),
            "{id}: original script copied once without rewriting"
        );
        require!(
            output
                .document()
                .links()
                .iter()
                .filter(|link| link.authored == block.span())
                .count()
                == 1,
            "{id}: exactly one original body link"
        );
        let selected = observation
            .descriptor()
            .admitted()
            .map_err(|_| "descriptor")?
            .setup()
            .ok_or("selected setup")?;
        let setup = vize_l2::lang::js::VueSetup::checked(
            file,
            selected,
            original
                .admitted_program()
                .ok_or("admitted original Program")?,
        )
        .map_err(|error| cstr!("{id}: {error:?}"))?;
        let expected = fixture
            .get("bindings")
            .and_then(Value::as_array)
            .ok_or("binding names")?;
        require!(
            setup.bindings().count() == expected.len(),
            "{id}: all actual root bindings"
        );
        for (binding, name) in setup.bindings().zip(expected) {
            let declaration = binding.declaration().ok_or("actual declaration")?;
            require!(
                Some(declaration.name.as_str()) == name.as_str(),
                "{id}: declaration order"
            );
            require!(core::ptr::eq(binding.file(), file), "{id}: binding owner");
            require!(
                declaration.is_direct_program() && declaration.scope == setup.exposure().scope(),
                "{id}: root setup scope"
            );
            require!(
                declaration.initializer == vize_l2::file::InitializerKind::PrimitiveLiteral,
                "{id}: original primitive fact"
            );
            if let Some(immutable) = fixture.get("immutableBindings").and_then(Value::as_array) {
                require!(
                    (declaration.kind == vize_l2::file::DeclarationKind::Const)
                        == immutable.contains(name),
                    "{id}: genuine immutable declaration class"
                );
            }
        }
        capture.push(serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map,"bindings":expected}));
    }
    if let Some(path) = std::env::var_os(capture_env) {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":schema, "adapter":"vize_atelier_sfc::compile_native_sfc", "fixtures":capture
        })).map_err(|error|cstr!("{error}"))?).map_err(|error|cstr!("{error}"))?;
    }
    Ok(())
}

#[test]
fn successful_compilation_moves_every_original_native_owner_with_the_complete_output()
-> Result<(), String> {
    let arena = Allocator::default();
    let source = "<script setup>/* original */let count=1;</script><template>{{count}}</template>";
    let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
    let script = compilation
        .observation()
        .scripts()
        .first()
        .ok_or("script")?;
    let body = script
        .syntax()
        .and_then(|syntax| syntax.program())
        .ok_or("original Program")?
        .body
        .as_ptr();
    let moved = Box::new(compilation);
    let (owner, result) = (*moved).into_parts();
    require!(result.is_ok(), "whole output moves with original custody");
    require!(
        owner.admitted().is_some(),
        "genuine native admission remains"
    );
    let syntax = owner
        .scripts()
        .first()
        .and_then(|script| script.syntax())
        .ok_or("retained original")?;
    require!(
        syntax.program().ok_or("Program")?.body.as_ptr() == body,
        "same original arena body"
    );
    require!(syntax.comments().count() == 1, "original comment retained");
    require!(
        core::ptr::eq(owner.descriptor().source(), source),
        "same descriptor source"
    );
    require!(
        owner
            .template()
            .and_then(|template| template.component())
            .is_some(),
        "whole original template owner"
    );
    Ok(())
}
