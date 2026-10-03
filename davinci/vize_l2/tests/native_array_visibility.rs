#[cfg(test)]
mod cases {
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
        cstr,
    };
    use vize_l1::{
        SurfaceParseOptions,
        container::{
            Vue,
            vue::{DescriptorObservation, DescriptorOptions},
        },
        markup::NativeTemplateComponent,
    };
    use vize_l2::file::FileIssueKind;
    use vize_l2::file::NativeFileInterpolationState;
    use vize_l2::lang::js::{
        NativeInterpolationInput, NativeSetupIssueKind, NativeTemplateIssueKind,
        NativeTemplateOwner,
    };

    fn descriptor<'a>(arena: &'a Allocator, source: &'a str) -> DescriptorObservation<'a> {
        Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        )
    }

    #[test]
    fn complete_array_syntax_cannot_expose_any_binding_from_its_whole_setup_unit() {
        for body in [
            "const primitive:number=1;const items:number[]=[];",
            "const items:{id:number}[]=[{id:1}];const primitive:number=1;",
            "const primitive:number=1;const items:number[]=1;",
        ] {
            let arena = Allocator::default();
            let source = cstr!(
                "<template>{{{{ primitive }}}}</template><script setup lang=ts>/* 🌸 */{body}</script>"
            );
            let descriptor = descriptor(&arena, &source);
            let selected =
                NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
                    .unwrap()
                    .unwrap();
            let mut original = NativeTemplateOwner::new(selected).unwrap();
            original.parse_setup_program().unwrap();
            let syntax = original.retained_setup().unwrap();
            assert_eq!(syntax.diagnostics().count(), 0);
            assert_eq!(syntax.comments().count(), 1);
            let program_body = syntax.program().unwrap().body.as_ptr();
            let script_span = syntax.source().span();
            {
                let mut walk = original.begin().unwrap();
                let selected = walk.selected();
                let child = selected.children().next().unwrap();
                let input = NativeInterpolationInput::from_operand(
                    selected
                        .observe_interpolation_expression(child.reborrow())
                        .unwrap(),
                );
                let issue = walk.root_interpolation(child, input).unwrap_err();
                assert!(matches!(issue.kind, NativeTemplateIssueKind::Artifact(_)));
                assert!(walk.complete().is_err());
            }
            let output = Box::new(original.finish());
            assert!(output.view().is_err());
            assert!(matches!(
                output.setup(),
                Err(issue) if issue.kind == NativeSetupIssueKind::IncompleteFile
            ));
            let syntax = output.retained_setup().unwrap();
            assert_eq!(syntax.program().unwrap().body.as_ptr(), program_body);
            assert_eq!(syntax.source().span(), script_span);
            assert!(core::ptr::eq(
                syntax.source().authored_root(),
                source.as_str(),
            ));
            let file = output.file().unwrap();
            assert!(file.issues().is_empty());
            assert_eq!(file.interrupted_programs().count(), 0);
            assert!(!file.is_complete());
            let [issue] = file.template_issues() else {
                panic!("the actual whole-unit visibility refusal");
            };
            assert_eq!(issue.kind, FileIssueKind::UnresolvedReference);
            assert_eq!(issue.span.slice(&source), "primitive");
            let [interpolation] = file.native_interpolations() else {
                panic!("the retained original interpolation owner");
            };
            assert!(matches!(
                interpolation.state(),
                NativeFileInterpolationState::Refused(NativeTemplateIssueKind::Artifact(_))
            ));
            let operand = interpolation.input().operand();
            assert_eq!(operand.raw_content(), " primitive ");
            assert!(core::ptr::eq(
                operand.syntax().source().authored_root(),
                source.as_str(),
            ));
        }
    }
}
