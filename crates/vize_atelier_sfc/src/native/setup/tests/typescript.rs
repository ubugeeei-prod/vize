use super::capture_pack;
use vize_l0::{String, cstr};

#[test]
fn three_whole_ts_primitive_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures()
-> Result<(), String> {
    let pack = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_sfc_ts_setup_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    capture_pack(
        pack,
        3,
        "VIZE_NATIVE_SFC_TS_SETUP_CAPTURE",
        "vize.native-sfc.ts-setup-capture",
    )
}

#[test]
fn ts_type_bearing_and_strict_invalid_bodies_keep_actual_owners_and_refuse_output()
-> Result<(), String> {
    use crate::{NativeSfcCompileOptions, compile_native_sfc};
    use vize_l0::Allocator;
    let arena = Allocator::default();
    for body in [
        "const value: number | string = 1;",
        "let value!:number;",
        "declare const value:number;",
        "const value=1 as number;",
        "const value=1 satisfies number;",
        "const value=1!;",
        "type Name=number;const value=1;",
        "interface Name{}const value=1;",
        "enum Name {Value=1}const value=1;",
        "namespace Name {}const value=1;",
        "import type {Name} from 'types';const value=1;",
        "export const value=1;",
        "const eval=1;",
        "let arguments=1;",
        "var public=1;",
        r"const \u0065val=1;",
        "const value=010;",
        r"let value='\8';",
    ] {
        let source = cstr!("<script setup lang=ts>{body}</script><template><p/></template>");
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        require!(
            compilation.result().is_err(),
            "no TS erasure/invalid module: {body}"
        );
        let owner = compilation.observation();
        require!(
            core::ptr::eq(owner.descriptor().source(), source.as_str()),
            "original root"
        );
        let script = owner.scripts().first().ok_or("actual script")?;
        require!(script.block().source() == body, "original TS body");
        let syntax = script.syntax().ok_or("original syntax")?;
        require!(
            syntax.admitted_program().is_some(),
            "genuine original parse"
        );
        require!(
            syntax.diagnostics().count() == 0,
            "unchanged ordinary diagnostics"
        );
    }
    Ok(())
}
