#[path = "native_scriptless_ssr/class_successor.rs"]
mod class_successor;

use vize_atelier_sfc::{
    NativeSsrSfcCompileError, NativeSsrSfcCompileOptions, compile_native_ssr_sfc,
};
use vize_l0::{Allocator, config::VueVersion};
use vize_l1::embed::syntax::{EmbedHole, NativeForRefusal};
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;
use vize_l2::{
    file::{FileIssueKind, RejectedFileFor, RejectedFileHandler},
    lang::js::{HandlerInputErrorKind, NativeTemplateIssueKind},
    op::{BindingOp, OnOp, Op, Region},
    resolution::ResolutionErrorKind,
};
use vize_l3::decision::ssr::SsrUnsupported;
use vize_l4::module::AssemblyError;
use vize_l4::targets::ssr::SsrErrorKind;

fn pack() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/native-sfc-ssr-vue-3.5.35.json")).unwrap()
}

fn original_on<'o, 'a>(region: &'o Region<'a>) -> Option<&'o OnOp<'a>> {
    for op in &region.ops {
        if let Op::Element(element) = op {
            for binding in &element.bindings {
                if let BindingOp::On(on) = binding {
                    return Some(on);
                }
            }
            if let Some(on) = original_on(&element.children) {
                return Some(on);
            }
        }
    }
    None
}

#[test]
fn original_whole_sfc_modules_keep_selected_custody_and_complete_maps() {
    let pack = pack();
    let mut modules = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let filename = fixture["filename"].as_str().unwrap();
        let compilation = compile_native_ssr_sfc(
            &arena,
            source,
            NativeSsrSfcCompileOptions {
                source_map: true,
                filename,
                ..NativeSsrSfcCompileOptions::default()
            },
        );
        let moved = core::hint::black_box(compilation);
        let observation = moved.observation();
        assert!(
            observation.issues().is_empty(),
            "{}: {:?}",
            fixture["id"],
            observation.issues()
        );
        assert!(core::ptr::eq(observation.descriptor().source(), source));
        let admitted = observation.admitted().unwrap();
        assert!(core::ptr::eq(admitted.observation(), observation));
        let view = admitted.into_template_view();
        assert!(core::ptr::eq(view.owner(), observation.template().unwrap()));
        let file = view.file().unwrap();
        assert!(file.is_complete());
        assert!(file.units().is_empty());
        assert!(core::ptr::eq(file.artifact().source(), source));
        let handler = match fixture["id"].as_str() {
            Some("handler-root" | "handler-nested") => {
                let on = original_on(file.artifact().root()).unwrap();
                let handler = file.handler_for(on).unwrap();
                assert!(handler.accepts_on(on));
                assert!(core::ptr::eq(handler.file(), file));
                let resolution = handler.resolution().unwrap();
                let input = resolution.input();
                let syntax = input.operand().syntax();
                assert!(core::ptr::eq(syntax.source().authored_root(), source));
                assert_eq!(
                    syntax.source().span().slice(source),
                    input.operand().raw_value()
                );
                assert_eq!(syntax.diagnostics().count(), 0);
                serde_json::json!({"raw":input.operand().raw_value(),"decoded":syntax.source().text(),"span":{"start":syntax.source().span().start,"end":syntax.source().span().end}})
            }
            _ => {
                assert!(original_on(file.artifact().root()).is_none());
                serde_json::Value::Null
            }
        };
        let provenance = file.artifact().provenance().to_vec();
        let output = moved
            .result()
            .unwrap_or_else(|error| panic!("{}: {error:?}", fixture["id"]));
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["sources"], serde_json::json!([filename]));
        let links: Vec<_> = output.document().links().iter().map(|link| {
            assert!(source.get(link.authored.start as usize..link.authored.end as usize).is_some());
            assert!(output.code().get(link.generated.start as usize..link.generated.end as usize).is_some());
            serde_json::json!({"authored":{"start":link.authored.start,"end":link.authored.end},"generated":{"start":link.generated.start,"end":link.generated.end},"name":link.name.as_deref(),"segment":link.segment})
        }).collect();
        let plain = compile_native_ssr_sfc(
            &arena,
            source,
            NativeSsrSfcCompileOptions {
                filename,
                ..NativeSsrSfcCompileOptions::default()
            },
        );
        let plain = plain.result().unwrap();
        assert_eq!(plain.code(), output.code());
        assert!(plain.source_map().is_none());
        assert!(plain.document().links().is_empty());
        assert_eq!(file.artifact().provenance(), provenance);
        modules.push(serde_json::json!({"id":fixture["id"],"source":source,"filename":filename,"code":output.code(),"map":map,"mapText":output.source_map().unwrap(),"links":links,"handler":handler,"outcome":"complete_original_sfc_module"}));
    }
    let capture_path = std::env::var("VIZE_NATIVE_SFC_SSR_CAPTURE").ok();
    if let Some(path) = &capture_path {
        std::fs::write(
            format!("{path}.positive.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"custody":"once_selected_scriptless_sfc","suiteCompletion":"positive_modules_only","modules":&modules})).unwrap(),
        ).unwrap();
    }
    let capture = serde_json::json!({"custody":"once_selected_scriptless_sfc","modules":modules,"refusals":refusals(),"classSuccessor":class_successor::capture()});
    class_successor::unchanged(&capture);
    if let Some(path) = capture_path {
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
    let frozen: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/native-scriptless-ssr-output-v2.json"
    ))
    .unwrap();
    assert_eq!(frozen["version"], 2);
    assert_eq!(frozen["state"], "reviewed", "whole successor is unreviewed");
    assert_eq!(capture, frozen["capture"]);
}

fn refusals() -> Vec<serde_json::Value> {
    let mut captures = Vec::new();
    for (id, source) in [
        (
            "ordinary-empty",
            "<script></script><template><div/></template>",
        ),
        (
            "setup-empty",
            "<template><div/></template><script setup></script>",
        ),
        (
            "setup-constant",
            "<script setup>const value=1;</script><template><div/></template>",
        ),
        ("global-style", "<template><div/></template><style></style>"),
        (
            "scoped-style",
            "<style scoped>div{color:red}</style><template><div/></template>",
        ),
        ("custom", "<template><div/></template><docs>kept</docs>"),
        ("external", "<template src='./other.vue'></template>"),
        ("missing", "<!--retained original-->"),
        ("search", "<template><search/></template>"),
        ("component", "<template><Widget/></template>"),
        ("slot", "<template><slot/></template>"),
        ("binding", "<template><div :id='value'/></template>"),
        ("interpolation", "<template>{{ 'retained' }}</template>"),
        (
            "for",
            "<template><div v-for='value in [1,2]'>x</div></template>",
        ),
        (
            "nested-whitespace",
            "<template><div>\n a \n</div></template>",
        ),
        (
            "handler-global",
            "<template><div><button @click='globalThis.__vize_ssr_event_probe += 1;'>go</button></div></template>",
        ),
        (
            "handler-syntax",
            "<template><button @click='return ('/></template>",
        ),
    ] {
        let arena = Allocator::default();
        let compilation =
            compile_native_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
        let error = compilation.result().unwrap_err();
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        match error {
            NativeSsrSfcCompileError::Lowering(issue) => match issue.kind {
                NativeSelectedSfcIssueKind::Descriptor
                | NativeSelectedSfcIssueKind::Script(_)
                | NativeSelectedSfcIssueKind::Style => {
                    assert!(compilation.observation().template().is_none());
                    assert!(compilation.observation().rejected_creation().is_none());
                }
                NativeSelectedSfcIssueKind::Template(_) => {
                    let original = compilation.observation().template().unwrap();
                    assert!(original.view().is_err());
                    assert!(original.rejected_file().is_none());
                    let file = original.file().unwrap();
                    assert!(!file.is_complete());
                    assert!(core::ptr::eq(file.artifact().source(), source));
                }
                _ => panic!("{id}: unexpected original lowering boundary: {error:?}"),
            },
            NativeSsrSfcCompileError::Ssr(_) => {
                let admitted = compilation.observation().admitted().unwrap();
                assert!(admitted.into_template_view().file().unwrap().is_complete());
            }
            _ => panic!("{id}: unexpected target boundary: {error:?}"),
        }
        match id {
            "ordinary-empty" | "setup-empty" | "setup-constant" => assert!(
                matches!(error, NativeSsrSfcCompileError::Lowering(issue) if matches!(issue.kind, NativeSelectedSfcIssueKind::Script(_)))
            ),
            "global-style" | "scoped-style" => assert!(
                matches!(error, NativeSsrSfcCompileError::Lowering(issue) if issue.kind == NativeSelectedSfcIssueKind::Style)
            ),
            "search" => {
                assert!(compilation.observation().admitted().is_some());
                assert!(
                    matches!(error, NativeSsrSfcCompileError::Ssr(_)),
                    "{id}: {error:?}"
                );
            }
            "slot" => {
                assert!(compilation.observation().admitted().is_none());
                assert!(
                    matches!(error, NativeSsrSfcCompileError::Lowering(issue) if matches!(issue.kind, NativeSelectedSfcIssueKind::Template(_))),
                    "{id}: {error:?}"
                );
            }
            "interpolation" => {
                assert!(
                    compilation.observation().admitted().is_some(),
                    "{id}: {error:?}"
                );
                assert!(
                    matches!(error, NativeSsrSfcCompileError::Ssr(error) if error.kind == SsrErrorKind::Unsupported(SsrUnsupported::Operation))
                );
            }
            "for" => {
                assert!(compilation.observation().admitted().is_none());
                assert!(
                    matches!(error, NativeSsrSfcCompileError::Lowering(issue) if matches!(issue.kind, NativeSelectedSfcIssueKind::Template(issue) if matches!(issue.kind, NativeTemplateIssueKind::For { kind: FileIssueKind::UnsupportedSyntax, .. }))),
                    "{id}: {error:?}"
                );
                let original = compilation.observation().template().unwrap();
                assert!(original.view().is_err());
                let [RejectedFileFor::Syntax(input)] =
                    original.file().unwrap().rejected_for_heads()
                else {
                    panic!(
                        "{id}: original rejected For syntax missing: {:?}",
                        original.file().unwrap().rejected_for_heads()
                    );
                };
                assert_eq!(input.kind, NativeForRefusal::CollectionShape);
                assert_eq!(input.operand().raw_value(), "value in [1,2]");
                assert_eq!(input.operand().value_span().slice(source), "value in [1,2]");
                let collection = input.operand().syntax().collection().unwrap().unwrap();
                assert!(core::ptr::eq(collection.source().authored_root(), source));
            }
            "handler-global" => {
                let original = compilation.observation().template().unwrap();
                let [RejectedFileHandler::Resolution { input, error }] =
                    original.file().unwrap().rejected_handlers()
                else {
                    panic!("{id}: original rejected handler resolution missing");
                };
                assert_eq!(error.kind, ResolutionErrorKind::MissingBinding);
                assert_eq!(
                    input.operand().value_span().slice(source),
                    input.operand().raw_value()
                );
                assert!(core::ptr::eq(
                    input.operand().syntax().source().authored_root(),
                    source
                ));
            }
            "handler-syntax" => {
                let original = compilation.observation().template().unwrap();
                let [RejectedFileHandler::Syntax(input)] =
                    original.file().unwrap().rejected_handlers()
                else {
                    panic!("{id}: original rejected handler syntax missing");
                };
                assert_eq!(input.kind, HandlerInputErrorKind::IncompleteSyntax);
                assert_eq!(input.operand().value_span().slice(source), "return (");
                assert!(core::ptr::eq(
                    input.operand().syntax().source().authored_root(),
                    source
                ));
                let syntax = input.operand().syntax();
                assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
                assert!(syntax.admitted_body().is_none());
                assert!(syntax.body().is_none());
                let diagnostic_rows: Vec<_> = syntax.diagnostics().collect();
                assert!(diagnostic_rows.is_empty());
            }
            _ => {}
        }
        captures.push(serde_json::json!({"id":id,"source":source,"outcome":"whole_sfc_refusal","reason":format!("{error:?}")}));
    }
    captures
}

#[test]
fn checked_runtime_and_original_profile_refuse_without_output() {
    let arena = Allocator::default();
    let source = "<template><div/></template>";
    let mut options = NativeSsrSfcCompileOptions::default();
    options.runtime_version = "99.0.0";
    let compilation = compile_native_ssr_sfc(&arena, source, options);
    assert!(compilation.observation().admitted().is_some());
    assert_eq!(
        compilation.result().unwrap_err(),
        NativeSsrSfcCompileError::Assembly(AssemblyError::UnsupportedRuntimeVersion)
    );
    options = NativeSsrSfcCompileOptions::default();
    options.descriptor.version = VueVersion::V2;
    let compilation = compile_native_ssr_sfc(&arena, source, options);
    assert_eq!(
        compilation.observation().descriptor().options(),
        options.descriptor
    );
    assert!(compilation.observation().admitted().is_none());
    assert!(
        matches!(compilation.result(), Err(NativeSsrSfcCompileError::Lowering(issue)) if issue.kind == NativeSelectedSfcIssueKind::Descriptor)
    );
}
