//! JSX/TSX -> Vue SSR compilation (#1580).
#![expect(
    clippy::disallowed_macros,
    clippy::unwrap_used,
    reason = "tests assert by panicking; insta and fixtures use format!"
)]

#[expect(dead_code, reason = "shared test helpers; each binary uses a subset")]
mod common;

use common::{snapshot_cases, snapshot_lang_cases};
use vize_atelier_jsx::{JsxLang, JsxOutputMode, SsrCompileOptions, compile_to_ssr};
use vize_l0::Allocator;

fn ssr_code(source: &str, lang: JsxLang) -> vize_l0::String {
    let bump = Allocator::new();
    let out = compile_to_ssr(&bump, source, lang, SsrCompileOptions::default());
    assert!(
        !out.has_errors(),
        "unexpected diagnostics: {:?}",
        out.diagnostics
    );
    assert_eq!(out.components.len(), 1, "expected exactly one component");
    out.components.into_iter().next().unwrap().code
}

#[test]
fn ssr_codegen_matrix() {
    let cases = [
        ("element", "const A = () => <div/>;"),
        ("text", "const A = () => <p>hello</p>;"),
        ("interpolation", "const A = () => <div>{msg}</div>;"),
        ("dynamic attr", "const A = () => <div id={id}>{msg}</div>;"),
        (
            "control flow",
            "const A = () => <div>{ok ? <span>yes</span> : <em>no</em>}</div>;",
        ),
        (
            "list",
            "const A = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        ssr_code(source, JsxLang::Jsx)
    }));
}

#[test]
fn ssr_tsx_codegen_snapshot() {
    let cases = [(
        "tsx ssr",
        JsxLang::Tsx,
        "const A = (): JSX.Element => <span>{label}</span>;",
    )];

    insta::assert_snapshot!(snapshot_lang_cases(&cases, ssr_code));
}

#[test]
fn ssr_preserves_component_metadata_and_client_mode() {
    let bump = Allocator::new();
    let out = compile_to_ssr(
        &bump,
        "const Fast = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );

    assert_eq!(out.components[0].component_name.as_deref(), Some("Fast"));
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn runtime_slots_capture_complete_modules_for_the_pinned_ssr_runtime() {
    let cases = [
        ("bare", "const A = () => <B v-slots={slots}/>;"),
        (
            "mixed",
            "const A = () => <B v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></B>;",
        ),
        ("member", "const A = () => <B v-slots={owner.slots}/>;"),
        (
            "conditional",
            "const A = () => <B v-slots={choose ? left : right}/>;",
        ),
        (
            "nested",
            "const A = () => <Outer><B v-slots={slots}/></Outer>;",
        ),
        (
            "dynamic-component",
            "const A = () => <view.Component v-slots={slots}/>;",
        ),
        ("alias", "const A = () => <B vSlots={slots}/>;"),
        ("compiled-ssr", "const A = () => <B v-slots={getSlots()}/>;"),
        (
            "dynamic-mixed",
            "const A = () => <view.Component v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></view.Component>;",
        ),
    ];
    let packets = cases.into_iter().map(|(id, source)| {
        serde_json::json!({ "id": id, "source": source, "code": ssr_code(source, JsxLang::Jsx).as_str() })
    }).collect::<Vec<_>>();
    assert_eq!(packets.len(), 9);
    if let Some(path) = std::env::var_os("VIZE_JSX_SLOTS_SSR_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&packets).expect("whole module packets"),
        )
        .expect("retain fresh JSX SSR modules");
    }
}

#[test]
fn unsupported_roots_and_builtins_keep_the_original_slots_diagnostic() {
    let expected = "v-slots forwards a slots object the compiler cannot see inside, which SSR \
        output cannot express: the server renderer inlines each slot's content. \
        Write the slots inline, e.g. v-slots={{ default: () => <div/> }}, or render \
        this component on the client.";
    for source in [
        "const A = () => <B v-slots={slots}><div v-once/></B>;",
        "const A = () => <Teleport v-slots={slots}/>;",
        "const A = () => <Suspense v-slots={slots}/>;",
    ] {
        let allocator = Allocator::new();
        let output = compile_to_ssr(
            &allocator,
            source,
            JsxLang::Jsx,
            SsrCompileOptions::default(),
        );
        assert_eq!(output.diagnostics.len(), 1, "{source}");
        let diagnostic = output
            .diagnostics
            .first()
            .expect("unsupported runtime object");
        assert!(diagnostic.is_error());
        assert_eq!(diagnostic.message.as_str(), expected);
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some("slots")
        );
    }
}

#[test]
fn vapor_server_mode_uses_the_same_checked_ssr_plan() {
    use vize_atelier_jsx::{VaporCompileOptions, compile_to_vapor};
    let source = "const A = () => <B v-slots={slots}/>;";
    let allocator = Allocator::new();
    let output = compile_to_vapor(
        &allocator,
        source,
        JsxLang::Jsx,
        VaporCompileOptions {
            ssr: true,
            ..Default::default()
        },
    );
    assert_eq!(output.diagnostics.len(), 0);
    assert_eq!(output.components.len(), 1);
    let component = output.components.first().expect("one server component");
    assert_eq!(component.mode, JsxOutputMode::Vapor);
    assert_eq!(component.code, ssr_code(source, JsxLang::Jsx));
}

#[test]
fn lowercase_component_keeps_the_original_plain_element_diagnostic() {
    let allocator = Allocator::new();
    let output = compile_to_ssr(
        &allocator,
        "const A = () => <component is={view} v-slots={slots}/>;",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic.is_error(),
                diagnostic.message.as_str(),
                diagnostic.start,
                diagnostic.end
            ))
            .collect::<Vec<_>>(),
        [(
            true,
            "v-slots can only be used on a component; a plain element has no slots.",
            37,
            52
        )]
    );
}

#[test]
fn vapor_server_slots_capture_all_nine_complete_modules() {
    use vize_atelier_jsx::{VaporCompileOptions, compile_to_vapor};
    let cases = [
        ("bare", "const A = () => <B v-slots={slots}/>;"),
        (
            "mixed",
            "const A = () => <B v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></B>;",
        ),
        ("member", "const A = () => <B v-slots={owner.slots}/>;"),
        (
            "conditional",
            "const A = () => <B v-slots={choose ? left : right}/>;",
        ),
        (
            "nested",
            "const A = () => <Outer><B v-slots={slots}/></Outer>;",
        ),
        (
            "dynamic-component",
            "const A = () => <view.Component v-slots={slots}/>;",
        ),
        ("alias", "const A = () => <B vSlots={slots}/>;"),
        ("compiled-ssr", "const A = () => <B v-slots={getSlots()}/>;"),
        (
            "dynamic-mixed",
            "const A = () => <view.Component v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></view.Component>;",
        ),
    ];
    let packets = cases
        .into_iter()
        .map(|(id, source)| {
            let allocator = Allocator::new();
            let output = compile_to_vapor(
                &allocator,
                source,
                JsxLang::Jsx,
                VaporCompileOptions { ssr: true },
            );
            assert!(
                output.diagnostics.is_empty(),
                "{id}: {:?}",
                output.diagnostics
            );
            assert_eq!(output.components.len(), 1, "{id}");
            let component = output.components.first().expect("one Vapor SSR component");
            assert_eq!(component.mode, JsxOutputMode::Vapor, "{id}");
            assert!(
                component.templates.is_empty(),
                "server output has no client templates: {id}"
            );
            assert_eq!(
                component.code,
                ssr_code(source, JsxLang::Jsx),
                "whole server module: {id}"
            );
            serde_json::json!({ "id": id, "source": source, "code": component.code.as_str() })
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 9);
    if let Some(path) = std::env::var_os("VIZE_JSX_VAPOR_SLOTS_SSR_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&packets).expect("whole Vapor SSR module packets"),
        )
        .expect("retain fresh JSX Vapor SSR modules");
    }
}
