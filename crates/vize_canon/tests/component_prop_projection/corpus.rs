//! Enumerate every outcome from the original inputs. For newly projected
//! files, the existing Canon generator supplies the independent tag and props
//! object. A separate pass checks complete TypeScript diagnostic packets.
use super::*;
use std::fs;
use vize_l4::targets::ts::{LinkMapping, LinkRole};

#[test]
#[ignore = "requires a NUL-separated JSX/TSX corpus manifest"]
fn component_props_in_original_modules() {
    let manifest = std::env::var_os("VIZE_MODULE_IR_MANIFEST").expect("VIZE_MODULE_IR_MANIFEST");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = 0;
    let mut syntax = 0;
    let mut lowering = 0;
    let mut refused = 0;
    let mut prior = 0;
    let mut owners = 0;
    let mut documents = Vec::new();
    for name in fs::read_to_string(manifest).unwrap().split_terminator('\0') {
        files += 1;
        let source = fs::read_to_string(root.join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let tsx = name.ends_with(".tsx") || name.ends_with(".tsx.txt");
        let profile = if tsx {
            SourceType::tsx()
        } else {
            SourceType::jsx()
        };
        let arena = Allocator::default();
        let parsed = Parser::new(arena.as_oxc(), &source, profile).parse_observed();
        let Some(observed) = parsed.admitted() else {
            syntax += 1;
            continue;
        };
        let scoping = SemanticBuilder::new()
            .build(observed.program())
            .semantic
            .into_scoping();
        let options = ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        };
        let Ok(lowered) = lower_module_def(&arena, &observed, Some(&scoping), options) else {
            lowering += 1;
            continue;
        };
        let original = project_module(&lowered.module);
        let candidate = project_module_with_component_checks(&lowered.module);
        let document = match (original, candidate) {
            (Ok(old), Ok(new)) => {
                assert_eq!(new.document(), old.document(), "{name}");
                assert_eq!(new.link_facts(), old.link_facts(), "{name}");
                assert_eq!(new.directives(), old.directives(), "{name}");
                prior += 1;
                continue;
            }
            (Err(_), Err(_)) => {
                refused += 1;
                continue;
            }
            (Err(ModuleProjectionError::UnpublishedHelper("ui.component")), Ok(doc)) => doc,
            (old, new) => panic!(
                "{name}: unexpected old/new status {:?} / {:?}",
                old.err(),
                new.err()
            ),
        };
        let lang = if tsx { JsxLang::Tsx } else { JsxLang::Jsx };
        let legacy = generate_jsx_document_virtual_ts(Path::new(name), &source, lang).unwrap();
        assert!(
            legacy.diagnostics.is_empty(),
            "{name}: {:?}",
            legacy.diagnostics
        );
        let text = document.document().as_str();
        let mut component_tags = document
            .document()
            .links()
            .iter()
            .zip(document.link_facts())
            .filter(|(_, fact)| fact.role == LinkRole::ComponentTag);
        let mut in_module = 0;
        while let Some((tag, fact)) = component_tags.next() {
            assert_eq!(fact.mapping, LinkMapping::Verbatim);
            let (object, fact) = component_tags.next().unwrap();
            assert_eq!(fact.mapping, LinkMapping::Anchor);
            let at = tag.authored.start as usize..tag.authored.end as usize;
            let exact = legacy
                .mappings
                .iter()
                .find(|row| {
                    row.src_range == at
                        && legacy
                            .code
                            .get(..row.gen_range.start)
                            .is_some_and(|prefix| prefix.ends_with("__vize_jsx_component__("))
                })
                .unwrap_or_else(|| panic!("missing Canon tag {name} {at:?}"));
            assert_eq!(
                legacy.code.get(exact.gen_range.clone()).unwrap(),
                tag.generated.slice(text),
                "{name}"
            );
            let props = legacy
                .mappings
                .iter()
                .find(|row| row.src_range == at && row.gen_range.start == exact.gen_range.end + 2)
                .unwrap_or_else(|| panic!("missing Canon object {name} {at:?}"));
            assert_eq!(
                legacy.code.get(props.gen_range.clone()).unwrap(),
                object.generated.slice(text),
                "{name}: {:?}",
                tag.authored
            );
            in_module += 1;
        }
        assert!(in_module > 0, "{name}");
        owners += in_module;
        documents.push(Comparison::new(name, source, legacy, document));
    }
    assert_eq!(files, syntax + lowering + refused + prior + documents.len());
    assert!(files > 0 && prior > 0);
    println!(
        "files={files} syntax={syntax} lowering={lowering} refused={refused} prior={prior} added={} owners={owners}",
        documents.len()
    );
    assert!(
        owners > 0 && !documents.is_empty(),
        "no lexical component was compared"
    );
    let diagnostics: usize = compare_checker(documents).iter().map(Vec::len).sum();
    println!("matched diagnostics={diagnostics}");
}
