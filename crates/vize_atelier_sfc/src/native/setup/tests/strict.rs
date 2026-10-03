use crate::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use vize_l0::{Allocator, String, cstr};
use vize_l2::lang::js::SetupIssueKind;

#[test]
fn strict_module_early_errors_refuse_with_the_complete_original_observation() -> Result<(), String>
{
    let arena = Allocator::default();
    let mut rows = Vec::new();
    for kind in ["const", "let", "var"] {
        for declaration in [
            "eval=1",
            "arguments=1",
            "public=1",
            "yield=1",
            "interface=1",
            r"\u0065val=1",
            "value=010",
            "value=08",
            "value=09.5",
            r"value='\1'",
            r"value='\8'",
            r"value='\9'",
            r"value='\00'",
        ] {
            let source =
                cstr!("<script setup>{kind} {declaration};</script><template><p/></template>");
            let compilation =
                compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
            require!(
                matches!(compilation.result(), Err(NativeSfcCompileError::ScriptSetup(issue))
                if issue.kind == SetupIssueKind::UnsupportedSyntax),
                "strict module refusal: {kind} {declaration}"
            );
            let observation = compilation.observation();
            require!(
                core::ptr::eq(observation.descriptor().source(), source.as_str()),
                "original owner"
            );
            require!(
                observation.admitted().is_some(),
                "neutral complete original File"
            );
            rows.push(serde_json::json!({"source": source.as_str(), "accepted":false, "script": observation.scripts().first().ok_or("script owner")?.block().source()}));
            let syntax = observation
                .scripts()
                .first()
                .and_then(|script| script.syntax())
                .ok_or("actual syntax")?;
            require!(
                syntax.admitted_program().is_some(),
                "syntax admission remains separate"
            );
            require!(
                syntax.diagnostics().count() == 0,
                "ordinary diagnostics unchanged"
            );
        }
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SFC_STRICT_SETUP_REFUSAL_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&rows).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}

#[test]
fn strict_safe_literal_spellings_preserve_original_body_and_complete_module() -> Result<(), String>
{
    let arena = Allocator::default();
    let mut rows = Vec::new();
    for literal in [
        "0",
        "0o10",
        "0x10",
        "0b10",
        "0.1",
        "0e1",
        "'雪'",
        r"'\0'",
        r"'\x01'",
        r"'\u0001'",
        r"'\\1'",
    ] {
        let source =
            cstr!("<script setup>const value={literal};</script><template>{{value}}</template>");
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        let output = compilation
            .result()
            .map_err(|error| cstr!("{literal}: {error:?}"))?;
        rows.push(
            serde_json::json!({"source":source.as_str(), "accepted":true,"code":output.code()}),
        );
        require!(
            output
                .code()
                .contains(cstr!("const value={literal};").as_str()),
            "authored spelling unchanged"
        );
        require!(
            compilation.observation().admitted().is_some(),
            "complete original custody"
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SFC_STRICT_SETUP_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&rows).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}
