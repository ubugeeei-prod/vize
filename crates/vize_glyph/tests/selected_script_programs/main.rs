//! Read one set of actual Vue script observations through the semantic and formatting APIs.
use std::{fs, path::Path};
use vize_glyph::{FormatError, FormatOptions, ScriptFormatQuery, format_script_with_sort_imports};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{
        Vue,
        vue::{AdmittedDescriptor, DescriptorOptions, ScriptRole, ScriptView},
    },
    embed::{
        EmbedSource,
        syntax::{EmbedSyntax, ProgramOptions, parse_program_once},
    },
    markup::SurfaceParseOptions,
};
use vize_l2::query::vue::{OrdinaryDemand, VueDemand, VueL2IrProducer};

mod profile;

const ALL: VueDemand = VueDemand {
    setup: true,
    ordinary: Some(OrdinaryDemand::Both),
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn direct<'a>(arena: &'a Allocator, script: ScriptView<'_, 'a>) -> EmbedSyntax<'a> {
    parse_program_once(
        arena,
        EmbedSource::authored(script.source(), script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    )
}

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    invalid_utf8: usize,
    descriptor_errors: usize,
    roles: usize,
    comments: usize,
    diagnostics: usize,
    outputs: usize,
    output_bytes: usize,
    formatting_errors: usize,
    files_lowered: usize,
    producer_errors: usize,
}

fn compare(view: AdmittedDescriptor<'_, '_>, arena: &Allocator, counts: &mut Counts, label: &str) {
    view.with_script_programs(arena, |programs| {
        let ordinary = view.ordinary().map(|s| direct(arena, s));
        let setup = view.setup().map(|s| direct(arena, s));
        let settings = FormatOptions::default();
        for (role, original) in [
            (ScriptRole::Ordinary, ordinary.as_ref()),
            (ScriptRole::Setup, setup.as_ref()),
        ] {
            let observed = programs.get(role);
            assert_eq!(observed.is_some(), original.is_some(), "{label}");
            let (Some(a), Some(b)) = (original, observed) else {
                continue;
            };
            counts.roles += 1;
            assert!(core::ptr::eq(a.source().text(), b.source().text()));
            assert!(core::ptr::eq(
                a.source().authored_root(),
                b.source().authored_root()
            ));
            assert_eq!(a.source().span(), b.source().span());
            assert_eq!(a.grammar(), b.grammar());
            assert_eq!(a.source_type(), b.source_type());
            assert_eq!(a.hole(), b.hole());
            assert_eq!(a.program().is_some(), b.program().is_some());
            let comments = |s: &EmbedSyntax<'_>| {
                s.comments()
                    .map(|c| {
                        (
                            c.kind(),
                            c.authored_span(),
                            c.decoded_span(),
                            c.text().map(|s| s.to_owned()),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            let diagnostics = |s: &EmbedSyntax<'_>| {
                s.diagnostics()
                    .map(|d| {
                        (
                            vize_l0::cstr!("{:?}", d.code()),
                            d.severity(),
                            d.message().to_owned(),
                            d.help().map(str::to_owned),
                            d.note().map(str::to_owned),
                            d.labels()
                                .map(|l| {
                                    (
                                        l.primary(),
                                        l.message().map(str::to_owned),
                                        l.decoded_span(),
                                        l.authored_span(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            let expected_comments = comments(a);
            let expected_diagnostics = diagnostics(a);
            counts.comments += expected_comments.len();
            counts.diagnostics += expected_diagnostics.len();
            assert_eq!(expected_comments, comments(b), "{label}");
            assert_eq!(expected_diagnostics, diagnostics(b), "{label}");
            let old = format_script_with_sort_imports(
                a.source().text(),
                &settings,
                arena,
                a.source_type(),
                None,
            );
            let query = ScriptFormatQuery::new(arena, b, &settings, None).unwrap();
            match (old, query.standalone()) {
                (Ok(original), Ok(bytes)) => {
                    assert_eq!(original.as_bytes(), bytes.as_bytes(), "{label}");
                    counts.outputs += 1;
                    counts.output_bytes += bytes.len();
                }
                (Err(FormatError::ScriptParseError(a)), Err(FormatError::ScriptParseError(b)))
                | (
                    Err(FormatError::ScriptFormatError(a)),
                    Err(FormatError::ScriptFormatError(b)),
                ) => {
                    assert_eq!(a.as_bytes(), b.as_bytes(), "{label}");
                    counts.formatting_errors += 1;
                }
                (a, b) => panic!("{label} {a:?} {b:?}"),
            }
            assert!(core::ptr::eq(b, programs.get(role).unwrap()));
        }
        let baseline = VueL2IrProducer::scripts(
            arena,
            view,
            ordinary.as_ref().and_then(EmbedSyntax::admitted_program),
            setup.as_ref().and_then(EmbedSyntax::admitted_program),
            ALL,
        );
        let candidate = VueL2IrProducer::from_script_programs(arena, programs, ALL);
        match (baseline, candidate) {
            (Ok(old), Ok(new)) => {
                let (a, b) = (old.finish().unwrap(), new.finish().unwrap());
                assert_eq!(a.l2ir().issues(), b.l2ir().issues(), "{label}");
                assert_eq!(a.model().contract(), b.model().contract(), "{label}");
                assert_eq!(a.model().options(), b.model().options(), "{label}");
                assert_eq!(a.model().macro_calls(), b.model().macro_calls(), "{label}");
                counts.files_lowered += 1;
            }
            (Err(old), Err(new)) => {
                assert_eq!(old, new, "{label}");
                counts.producer_errors += 1;
            }
            _ => panic!("different File producer admission for {label}"),
        }
    });
}

#[test]
fn consumers_observe_original_roles_after_formatting_or_parser_refusals() {
    let mut counts = Counts::default();
    for source in [
        "<script setup lang='ts'>const p=defineProps<{x:string}>()</script><template>{{ p.x }}</template>",
        "<template>雪</template><script setup>/* α */ const x=1</script><script>/* β */ export default {props:['x']}</script>",
        "<script>const x=(;</script><script setup>const y = 1</script>",
        "<template>{{ x }}</template>",
    ] {
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, source, options());
        compare(owner.admitted().unwrap(), &arena, &mut counts, source);
    }
    assert_eq!(
        (
            counts.roles,
            counts.files_lowered,
            counts.producer_errors,
            counts.formatting_errors
        ),
        (5, 3, 1, 1)
    );
    assert!(counts.comments >= 2 && counts.diagnostics > 0);
}

#[test]
#[ignore = "compares original pinned and tracked Vue files supplied by NUL-delimited manifest"]
fn original_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = fs::read(std::env::var_os("VIZE_SCRIPT_PROGRAMS_MANIFEST").unwrap()).unwrap();
    let mut counts = Counts::default();
    for raw in list.split(|&b| b == 0).filter(|s| !s.is_empty()) {
        let name = core::str::from_utf8(raw).unwrap();
        let bytes = fs::read(root.join(name)).unwrap();
        counts.files += 1;
        let Ok(source) = core::str::from_utf8(&bytes) else {
            counts.invalid_utf8 += 1;
            continue;
        };
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, source, options());
        let Ok(view) = owner.admitted() else {
            counts.descriptor_errors += 1;
            continue;
        };
        compare(view, &arena, &mut counts, name);
    }
    assert!(counts.outputs > 0 && counts.files_lowered > 0);
    eprintln!("{counts:?}");
}
