use crate::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use vize_l0::{Allocator, String, cstr};
use vize_l2::lang::js::SetupIssueKind;
use vize_l4::{module::setup::SetupEmitErrorKind, targets::dom::DomErrorKind};

#[test]
fn complete_neutral_syntax_cannot_bypass_whole_setup_eligibility() -> Result<(), String> {
    let arena = Allocator::default();
    for script in [
        "const value=1;",
        "let value=1;42;",
        "42;let value=1;",
        "let value=1;var copy=value;",
        "var value;",
        "'use strict';let value=1;",
        "#!/usr/bin/env node\nlet value=1;",
    ] {
        let source = cstr!("<script setup>{script}</script><template><p/></template>");
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        require!(
            matches!(compilation.result(), Err(NativeSfcCompileError::ScriptSetup(issue)) if issue.kind==SetupIssueKind::UnsupportedSyntax),
            "whole syntax refusal: {script}"
        );
        require!(
            compilation.observation().admitted().is_some(),
            "neutral complete custody is separate: {script}"
        );
        require!(
            compilation
                .observation()
                .scripts()
                .first()
                .and_then(|script| script.syntax())
                .is_some(),
            "original syntax retained"
        );
    }
    Ok(())
}

#[test]
fn generated_operations_and_parameter_names_refuse_before_any_partial_output() -> Result<(), String>
{
    let arena = Allocator::default();
    for name in ["Object", "__value"] {
        let source = cstr!("<script setup>let {name}=1;</script><template><p/></template>");
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        let error = compilation
            .result()
            .err()
            .ok_or("typed generated collision")?;
        let file = compilation
            .observation()
            .file()
            .ok_or("original File")?
            .file();
        let declaration = file
            .bindings()
            .next()
            .and_then(|binding| binding.declaration())
            .ok_or("actual colliding declaration")?;
        require!(declaration.name.as_str() == name, "actual normalized name");
        require!(
            matches!(error, NativeSfcCompileError::SetupEmission(issue)
            if issue.kind==SetupEmitErrorKind::GeneratedBindingCollision && issue.span==declaration.span),
            "typed original collision span"
        );
        require!(
            compilation.observation().admitted().is_some(),
            "collision does not discard original owners"
        );
    }
    Ok(())
}

#[test]
fn prohibited_script_families_and_collisions_keep_their_original_program_observations()
-> Result<(), String> {
    let arena = Allocator::default();
    for source in [
        "<script>let value=1;</script><template><p/></template>",
        "<script>let ordinary=1;</script><script setup>let value=1;</script><template><p/></template>",
        "<script setup lang=ts>let value=1;</script><template><p/></template>",
        "<script setup>/* only comment */</script><template><p/></template>",
        "<script setup>import value from 'x';</script><template><p/></template>",
        "<script setup>export let value=1;</script><template><p/></template>",
        "<script setup>let value=defineProps();</script><template><p/></template>",
        "<script setup>let value=new Number(1);</script><template><p/></template>",
        "<script setup>let value=import('x');</script><template><p/></template>",
        "<script setup>function tag(){}let value=tag`x`;</script><template><p/></template>",
        "<script setup>let value=this;</script><template><p/></template>",
        "<script setup>let {value}={value:1};</script><template><p/></template>",
        "<script setup>let [value]=[1];</script><template><p/></template>",
        "<script setup>let value=1;var value=2;</script><template><p/></template>",
        "<script setup>function hidden(value){let value=1;}let visible=1;</script><template><p/></template>",
        "<script setup>let __props=1;</script><template><p/></template>",
        "<script setup>let __expose=1;</script><template><p/></template>",
        "<script setup>let __returned__=1;</script><template><p/></template>",
        "<script setup>let __proto__=1;</script><template><p/></template>",
        "<script setup>let value={};</script><template><p/></template>",
        "<script setup>let value=()=>1;</script><template><p/></template>",
    ] {
        let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        require!(
            compilation.result().is_err(),
            "prohibited original source: {source}"
        );
        require!(
            core::ptr::eq(compilation.observation().descriptor().source(), source),
            "same original descriptor"
        );
        require!(
            !compilation.observation().scripts().is_empty(),
            "original scripts retained"
        );
        for script in compilation.observation().scripts() {
            let syntax = script.syntax().ok_or("actual original syntax")?;
            require!(syntax.program().is_some(), "original Program retained");
            require!(
                core::ptr::eq(syntax.source().text(), script.block().source()),
                "same physical authored script"
            );
        }
    }
    Ok(())
}

#[test]
fn complete_setup_does_not_grant_unavailable_template_spelling() -> Result<(), String> {
    let arena = Allocator::default();
    for template in ["{{count + 1}}", "{{({value:count})}}"] {
        let source = cstr!("<script setup>let count=1;</script><template>{template}</template>");
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        require!(
            compilation.observation().admitted().is_some(),
            "original whole native custody retained"
        );
        require!(
            matches!(compilation.result(),Err(NativeSfcCompileError::Dom(error))
            if error.kind==DomErrorKind::UncertifiedExpressionSpelling),
            "typed spelling refusal: {template}"
        );
        require!(
            compilation
                .observation()
                .template()
                .and_then(|template| template.component())
                .is_some(),
            "whole original template"
        );
    }
    Ok(())
}
