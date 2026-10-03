use vize_atelier_sfc::{
    NativeSsrSfcCompileError, NativeSsrSfcCompileOptions, compile_native_ssr_sfc,
};
use vize_l0::{Allocator, config::VueVersion};
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;
use vize_l3::decision::ssr::SsrUnsupported;
use vize_l4::module::AssemblyError;
use vize_l4::targets::ssr::SsrErrorKind;

fn pack() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/native-sfc-ssr-vue-3.5.35.json")).unwrap()
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
        modules.push(serde_json::json!({"id":fixture["id"],"source":source,"filename":filename,"code":output.code(),"map":map,"links":links,"outcome":"complete_original_sfc_module"}));
    }
    if let Ok(path) = std::env::var("VIZE_NATIVE_SFC_SSR_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({"custody":"once_selected_scriptless_sfc","modules":modules,"refusals":refusals()})).unwrap()).unwrap();
    } else {
        refusals();
    }
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
        ("class", "<template><div class='static'/></template>"),
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
        match id {
            "ordinary-empty" | "setup-empty" | "setup-constant" => assert!(
                matches!(error, NativeSsrSfcCompileError::Lowering(issue) if matches!(issue.kind, NativeSelectedSfcIssueKind::Script(_)))
            ),
            "global-style" | "scoped-style" => assert!(
                matches!(error, NativeSsrSfcCompileError::Lowering(issue) if issue.kind == NativeSelectedSfcIssueKind::Style)
            ),
            "search" | "slot" => {
                assert!(matches!(error, NativeSsrSfcCompileError::Ssr(_)))
            }
            "interpolation" | "for" => {
                assert!(compilation.observation().admitted().is_some());
                assert!(
                    matches!(error, NativeSsrSfcCompileError::Ssr(error) if error.kind == SsrErrorKind::Unsupported(SsrUnsupported::Operation))
                );
            }
            "handler-syntax" => {
                let original = compilation.observation().template().unwrap();
                assert!(original.view().is_err());
                assert!(!original.file().unwrap().rejected_handlers().is_empty());
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
