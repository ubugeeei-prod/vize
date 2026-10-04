use vize_atelier_sfc::{
    NativeSetupSsrSfcCompileError as Error, NativeSsrSfcCompileOptions,
    compile_native_setup_ssr_sfc,
};
use vize_l0::{Allocator, config::VueVersion};
use vize_l1::container::vue::ScriptRole;
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind as Lower;
use vize_l2::{
    file::{FileIssueKind, RejectedFileHandler},
    lang::js::{NativeSetupIssueKind, NativeTemplateIssueKind as Template},
    op::Op,
};
use vize_l3::decision::ssr::SsrUnsupported;
use vize_l4::{
    module::{AssemblyError, setup::SetupEmitErrorKind},
    targets::ssr::SsrErrorKind,
};

enum Expected {
    Lower(Lower),
    Template(Template),
    Ssr(SsrUnsupported),
    Handler(FileIssueKind),
    Collision,
}

pub fn capture() -> Vec<serde_json::Value> {
    let mut captures = Vec::new();
    for (name, source, expected) in [
        (
            "ordinary",
            "<script></script><script setup>const value=1</script><template><i/></template>",
            Expected::Lower(Lower::Script(ScriptRole::Ordinary)),
        ),
        (
            "style",
            "<script setup>const value=1</script><template><i/></template><style></style>",
            Expected::Lower(Lower::Style),
        ),
        (
            "custom",
            "<custom>kept</custom><script setup>const value=1</script><template><i/></template>",
            Expected::Lower(Lower::Descriptor),
        ),
        (
            "external",
            "<script setup src='external.js'></script><template><i/></template>",
            Expected::Lower(Lower::Descriptor),
        ),
        (
            "missing-setup",
            "<template><i/></template>",
            Expected::Lower(Lower::MissingSetup),
        ),
        (
            "empty",
            "<script setup>/*original*/</script><template><i/></template>",
            Expected::Template(Template::SetupPolicy(NativeSetupIssueKind::EmptyProgram)),
        ),
        (
            "imports",
            "<script setup>import value from 'original'</script><template><i/></template>",
            Expected::Template(Template::SetupPolicy(
                NativeSetupIssueKind::UnsupportedSyntax,
            )),
        ),
        (
            "function",
            "<script setup>function value(){}</script><template><i/></template>",
            Expected::Template(Template::SetupPolicy(
                NativeSetupIssueKind::UnsupportedSyntax,
            )),
        ),
        (
            "array",
            "<script setup>const value=[1]</script><template><i/></template>",
            Expected::Template(Template::SetupPolicy(
                NativeSetupIssueKind::UnsupportedSyntax,
            )),
        ),
        (
            "generated-collision",
            "<script setup>const Object=1</script><template><i/></template>",
            Expected::Collision,
        ),
        (
            "compound",
            "<script setup>const value=1</script><template>{{value+1}}</template>",
            Expected::Ssr(SsrUnsupported::Expression),
        ),
        (
            "member",
            "<script setup>const value='ab'</script><template>{{value.length}}</template>",
            Expected::Ssr(SsrUnsupported::Expression),
        ),
        (
            "adjacent-text",
            "<script setup>const value=1</script><template>hello {{value}}!</template>",
            Expected::Ssr(SsrUnsupported::RootTextGrouping),
        ),
        (
            "adjacent-interpolation",
            "<script setup>const value=1</script><template>{{value}}{{true}}</template>",
            Expected::Ssr(SsrUnsupported::RootTextGrouping),
        ),
        (
            "separated-interpolations",
            "<script setup>const value=1</script><template>{{value}}<i/>{{value}}</template>",
            Expected::Ssr(SsrUnsupported::Operation),
        ),
        (
            "nested",
            "<script setup>const value=1</script><template><i>{{value}}</i></template>",
            Expected::Template(Template::UnsupportedChild),
        ),
        (
            "for",
            "<script setup>const rows='ab'</script><template><i v-for='item in rows'/></template>",
            Expected::Ssr(SsrUnsupported::Operation),
        ),
        (
            "search",
            "<script setup>const value=1</script><template><search/></template>",
            Expected::Ssr(SsrUnsupported::ElementSemantics),
        ),
        (
            "handler-unresolved",
            "<script setup>const value=1</script><template><button @click='missing'/></template>",
            Expected::Handler(FileIssueKind::UnresolvedReference),
        ),
        (
            "handler-safety",
            "<script setup>const value=1</script><template><button @click='return ('/></template>",
            Expected::Handler(FileIssueKind::UnsupportedSyntax),
        ),
    ] {
        let arena = Allocator::default();
        let compilation =
            compile_native_setup_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
        let error = compilation.result().unwrap_err();
        let observation = compilation.observation();
        assert!(core::ptr::eq(
            observation.original().descriptor().source(),
            source
        ));
        let diagnostics = match expected {
            Expected::Lower(expected) => {
                assert!(
                    matches!(error, Error::Lowering(issue) if issue.kind == expected),
                    "{name}: {error:?}"
                );
                assert!(observation.admitted().is_none());
                assert!(observation.original().template().is_none());
                serde_json::json!([])
            }
            Expected::Template(expected) => {
                assert!(
                    matches!(error, Error::Lowering(issue) if matches!(issue.kind,
                    Lower::Template(issue) if issue.kind == expected)),
                    "{name}: {error:?}"
                );
                assert!(observation.admitted().is_none());
                let original = observation.original().template().unwrap();
                let syntax = original.retained_setup().unwrap();
                assert!(core::ptr::eq(syntax.source().authored_root(), source));
                assert!(syntax.program().is_some());
                assert_eq!(syntax.diagnostics().count(), 0);
                assert!(original.view().is_err());
                assert!(original.rejected_file().is_none());
                let file = original.file().unwrap();
                assert_eq!(file.is_complete(), expected != Template::UnsupportedChild);
                serde_json::json!([])
            }
            Expected::Ssr(expected) => {
                assert!(
                    matches!(error, Error::Ssr(error) if error.kind == SsrErrorKind::Unsupported(expected)),
                    "{name}: {error:?}"
                );
                let admitted = observation.admitted().unwrap();
                let setup = admitted.setup();
                let file = setup.file();
                assert!(file.is_complete());
                if name == "for" {
                    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
                        panic!("{name}: exact original For missing")
                    };
                    let head = file.for_head_for(original).unwrap();
                    assert!(head.accepts(original));
                    assert!(core::ptr::eq(head.file(), file));
                    let input = head.resolution().unwrap().input();
                    assert_eq!(input.operand().raw_value(), "item in rows");
                    assert!(core::ptr::eq(
                        input.operand().syntax().source().authored_root(),
                        source
                    ));
                    assert_eq!(input.operand().syntax().diagnostics().count(), 0);
                    let alias = head.value().unwrap();
                    assert!(alias.template_declaration().is_some());
                    assert!(setup.binding(alias).is_err());
                    assert_ne!(head.scope(), Some(setup.scope()));
                }
                serde_json::json!([])
            }
            Expected::Collision => {
                assert!(
                    matches!(error, Error::Setup(error) if error.kind == SetupEmitErrorKind::GeneratedBindingCollision),
                    "{name}: {error:?}"
                );
                assert!(observation.admitted().unwrap().setup().file().is_complete());
                serde_json::json!([])
            }
            Expected::Handler(expected) => {
                assert!(
                    matches!(error, Error::Lowering(issue) if matches!(issue.kind,
                    Lower::Template(issue) if matches!(issue.kind, Template::Handler { kind, .. } if kind == expected))),
                    "{name}: {error:?}"
                );
                assert!(observation.admitted().is_none());
                let original = observation.original().template().unwrap();
                assert!(original.view().is_err());
                assert!(original.retained_setup().unwrap().program().is_some());
                let file = original.file().unwrap();
                assert!(!file.is_complete());
                match (name, file.rejected_handlers()) {
                    ("handler-unresolved", [RejectedFileHandler::Resolution { input, error }]) => {
                        assert_eq!(
                            error.kind,
                            vize_l2::resolution::ResolutionErrorKind::MissingBinding
                        );
                        assert_eq!(input.operand().raw_value(), "missing");
                        assert!(core::ptr::eq(
                            input.operand().syntax().source().authored_root(),
                            source
                        ));
                        let diagnostics: Vec<_> = input.operand().syntax().diagnostics().collect();
                        assert!(diagnostics.is_empty());
                    }
                    ("handler-safety", [RejectedFileHandler::Syntax(input)]) => {
                        assert_eq!(
                            input.kind,
                            vize_l2::lang::js::HandlerInputErrorKind::IncompleteSyntax
                        );
                        let syntax = input.operand().syntax();
                        assert_eq!(
                            syntax.hole(),
                            Some(vize_l1::embed::syntax::EmbedHole::SafetyAdmission)
                        );
                        assert_eq!(input.operand().raw_value(), "return (");
                        assert!(core::ptr::eq(syntax.source().authored_root(), source));
                        assert!(syntax.body().is_none() && syntax.admitted_body().is_none());
                        let diagnostics: Vec<_> = syntax.diagnostics().collect();
                        assert!(diagnostics.is_empty());
                    }
                    _ => panic!("{name}: exact original rejected handler owner missing"),
                }
                serde_json::json!([])
            }
        };
        captures.push(
            serde_json::json!({"name":name,"source":source,"reason":format!("{error:?}"),
            "diagnostics":diagnostics,"completeModule":false}),
        );
    }
    captures
}

#[test]
fn original_profile_and_runtime_vocabulary_remain_checked() {
    let arena = Allocator::default();
    let source = "<script setup>const value=1</script><template>{{value}}</template>";
    let mut options = NativeSsrSfcCompileOptions::default();
    options.runtime_version = "99.0.0";
    let compilation = compile_native_setup_ssr_sfc(&arena, source, options);
    assert!(compilation.observation().admitted().is_some());
    assert_eq!(
        compilation.result().unwrap_err(),
        Error::Assembly(AssemblyError::UnsupportedRuntimeVersion)
    );
    options = NativeSsrSfcCompileOptions::default();
    options.descriptor.version = VueVersion::V2;
    let compilation = compile_native_setup_ssr_sfc(&arena, source, options);
    assert_eq!(
        compilation.observation().original().descriptor().options(),
        options.descriptor
    );
    assert!(compilation.observation().admitted().is_none());
    assert!(
        matches!(compilation.result(), Err(Error::Lowering(issue)) if issue.kind == Lower::Descriptor)
    );
}
