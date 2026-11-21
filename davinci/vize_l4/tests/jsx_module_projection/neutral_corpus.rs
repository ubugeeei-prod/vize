//! Run the checked module and the existing projection on the same admitted
//! Program for each path in a corpus manifest. Refusals are part of the oracle.

use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::{fs, path::Path};
use vize_l0::Allocator;
use vize_l1_to_l2::framework::vue::jsx::{
    ModuleLoweringError, ModuleOptions, VIZE, lower_module, lower_module_def,
};
use vize_l2::component::Exec;
use vize_l4::targets::ts::{project_jsx_module, project_module};

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    syntax: usize,
    lowering: usize,
    projection: usize,
    documents: usize,
    roots: usize,
    generated_bytes: usize,
    links: usize,
}

#[test]
#[ignore = "takes an explicit JSX/TSX corpus manifest"]
fn compare_modules_at_every_outcome() {
    let manifest = std::env::var_os("VIZE_MODULE_IR_MANIFEST").expect("VIZE_MODULE_IR_MANIFEST");
    let mut count = Counts::default();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in fs::read_to_string(manifest).unwrap().split_terminator('\0') {
        let path = root.join(name);
        let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("{name}: {e}"));
        let profile = if name.ends_with(".tsx") || name.ends_with(".tsx.txt") {
            SourceType::tsx()
        } else {
            SourceType::jsx()
        };
        let arena = Allocator::default();
        let observation = Parser::new(arena.as_oxc(), &source, profile).parse_observed();
        count.files += 1;
        let Some(program) = observation.admitted() else {
            count.syntax += 1;
            continue;
        };
        let scoping = SemanticBuilder::new()
            .build(program.program())
            .semantic
            .into_scoping();
        let options = ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        };
        let old = lower_module(&arena, &source, program.program(), Some(&scoping), options);
        let next = lower_module_def(&arena, &program, Some(&scoping), options);
        if let Ok(ref lowered) = next {
            assert_eq!(old.issues(), lowered.issues, "{name}");
        }
        let (roots, module) = match (old.into_components(), next) {
            (Ok(roots), Ok(lowered)) => (roots, lowered.module),
            (Err(expected), Err(ModuleLoweringError::Component(actual))) => {
                assert_eq!(actual, expected, "{name}");
                count.lowering += 1;
                continue;
            }
            (old, new) => panic!(
                "lowering differs at {name}: old={:?}, new={:?}; requested {:?}, resolved {:?}",
                old.err(),
                new.err(),
                program.source_type(),
                program.program().source_type
            ),
        };
        count.roots += roots.len();
        match (
            project_jsx_module(program.program(), &roots),
            project_module(&module),
        ) {
            (Ok(expected), Ok(actual)) => {
                assert_eq!(actual.document(), expected.document(), "{name}");
                assert_eq!(actual.link_facts(), expected.link_facts(), "{name}");
                assert_eq!(actual.directives(), expected.directives(), "{name}");
                count.documents += 1;
                count.generated_bytes += actual.document().as_str().len();
                count.links += actual.document().links().len();
            }
            (Err(expected), Err(actual)) => {
                assert_eq!(actual, expected, "{name}");
                count.projection += 1;
            }
            _ => panic!("projection differs at {name}"),
        }
    }
    assert!(count.documents > 0 && count.roots > 0);
    assert_eq!(
        count.files,
        count.syntax + count.lowering + count.projection + count.documents
    );
    println!("{count:?}");
}
