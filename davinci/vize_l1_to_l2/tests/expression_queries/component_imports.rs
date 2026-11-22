//! Compare authored component references with the actual Croquis component identity resolver.
use super::{
    component_projection::{draw, original, product, retained_product},
    input, observe,
};
use std::{fs, hint::black_box, path::Path, time::Instant};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::NativeSfcObservation;
use vize_l2::query::ComponentImportIssue;

fn compare_declarations(owner: &NativeSfcObservation<'_>) {
    use oxc_ast::ast::{ImportDeclarationSpecifier as Specifier, Statement};
    use oxc_span::SourceType;
    let descriptor =
        vize_atelier_sfc::parse_sfc(owner.descriptor().source(), Default::default()).unwrap();
    let script = descriptor.script_setup.unwrap();
    let arena = Allocator::new();
    let ty = if script.lang.as_deref() == Some("ts") {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };
    let parsed = oxc_parser::Parser::new(&arena, &script.content, ty).parse();
    let mut originals = std::collections::BTreeMap::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let (local, export) = match specifier {
                Specifier::ImportSpecifier(s) if !s.import_kind.is_type() => {
                    (&s.local, s.imported.name().as_str())
                }
                Specifier::ImportDefaultSpecifier(s) => (&s.local, "default"),
                _ => continue,
            };
            let start = script.loc.start as u32;
            originals.insert(
                (start + local.span.start, start + local.span.end),
                (local.name.as_str(), import.source.value.as_str(), export),
            );
        }
    }
    for site in owner.setup_component_imports().unwrap().sites() {
        let declaration = site.binding().declaration().unwrap();
        assert_eq!(
            originals
                .get(&(declaration.span.start, declaration.span.end))
                .copied(),
            Some((declaration.name.as_str(), site.module(), site.export()))
        );
    }
}

#[test]
fn nested_named_default_and_kebab_tags_borrow_their_import_declarations() {
    let script =
        "import MyCard from './card.vue'; import { Footer as PageFoot } from './components';";
    let source = format!(
        "<script setup lang='ts'>{script}</script>\r\n<template>😀<MyCard id='x'><page-foot/></MyCard><MyCard /></template>"
    );
    let arena = Allocator::new();
    let owner = observe(&arena, &source);
    assert!(owner.admitted().is_none());
    let actual = product(&owner).unwrap();
    compare_declarations(&owner);
    let (croquis, start, _) = draw(&source);
    assert_eq!(actual, original(&source, &croquis, start));
    assert_eq!(
        actual
            .iter()
            .map(|s| (s.tag.as_str(), s.module.as_str(), s.export.as_str()))
            .collect::<Vec<_>>(),
        [
            ("MyCard", "./card.vue", "default"),
            ("page-foot", "./components", "Footer"),
            ("MyCard", "./card.vue", "default")
        ]
    );
    let mut stored = Vec::new();
    let imports = owner.setup_component_imports().unwrap();
    assert!(std::ptr::eq(imports.source(), source.as_str()));
    for site in imports.sites() {
        stored.push((
            site.node().index(),
            site.element_span().slice(&source),
            site.binding().declaration().unwrap().span,
        ));
        let name = site.name_span();
        for offset in name.start..name.end {
            assert_eq!(imports.at_offset(offset).unwrap().node(), site.node());
        }
        assert!(imports.at_offset(name.start - 1).is_none());
        assert!(imports.at_offset(name.end).is_none());
    }
    assert!(imports.at_offset(u32::MAX).is_none());
    assert_eq!(
        stored.iter().map(|r| r.1).collect::<Vec<_>>(),
        [
            "<MyCard id='x'><page-foot/></MyCard>",
            "<page-foot/>",
            "<MyCard />"
        ]
    );
    assert!(stored.windows(2).all(|r| r[0].0 < r[1].0));
    assert_eq!(stored[0].2, stored[2].2);
    assert_eq!(stored[1].2.start as usize, source.find("PageFoot").unwrap());
}

#[test]
fn unbound_local_type_only_namespace_and_ambiguous_targets_refuse_the_product() {
    for (script, template) in [
        ("import Card from './card';", "<Card/><Global/>"),
        ("import type { Card } from './card';", "<Card/>"),
        ("import * as Card from './card';", "<Card/>"),
        (r"import Card from './\ud800';", "<Card/>"),
        (r"import { '\ud800' as Card } from './card';", "<Card/>"),
        ("import * as Card from './card';", "<Card.Widget/>"),
        ("import Card from './card';", "<component :is='Card'/>"),
        ("const Card = 1;", "<Card/>"),
        (
            "import CardPiece from './a';import cardPiece from './b';",
            "<card-piece/>",
        ),
        ("import Transition from './user';", "<Transition/>"),
        (
            "import Card from './card';",
            "<div v-if='unknown'><Card/></div>",
        ),
    ] {
        let source =
            format!("<script setup lang='ts'>{script}</script><template>{template}</template>");
        let arena = Allocator::new();
        assert!(product(&observe(&arena, &source)).is_err(), "{source}");
    }
}

#[test]
fn detached_equal_text_program_and_foreign_template_are_refused() {
    use vize_l2::query::Model;
    let source = "<script setup>import Card from './c'</script><template><Card/></template>";
    let a = Allocator::new();
    let b = Allocator::new();
    let one = observe(&a, source);
    let two = observe(&b, source);
    let file = one.rejected_file().unwrap().file().unwrap();
    let descriptor = one.descriptor().admitted().unwrap();
    let script = descriptor.setup().unwrap();
    let program = two.scripts()[0]
        .syntax()
        .unwrap()
        .admitted_program()
        .unwrap();
    assert_eq!(
        Model::new(file).visit_setup_component_imports(
            script,
            &program,
            descriptor.template().unwrap(),
            |_| unreachable!()
        ),
        Err(ComponentImportIssue::OriginalProgram)
    );
    let copied = source.to_owned();
    let three = observe(&b, &copied);
    let template = three.descriptor().admitted().unwrap().template().unwrap();
    let own = one.scripts()[0]
        .syntax()
        .unwrap()
        .admitted_program()
        .unwrap();
    assert_eq!(
        Model::new(file).visit_setup_component_imports(script, &own, template, |_| unreachable!()),
        Err(ComponentImportIssue::OriginalProgram)
    );
}

#[test]
#[ignore = "original Vue files and the independent Croquis resolver"]
fn original_vue_files() {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut invalid, mut refused, mut empty, mut resolved, mut uses) =
        (0, 0, 0, 0, 0, 0);
    let mut parser_diagnostics = 0;
    for name in manifest.split_terminator('\0') {
        files += 1;
        let bytes = fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let arena = Allocator::new();
        let owner = observe(&arena, source);
        let Ok(rows) = product(&owner) else {
            refused += 1;
            continue;
        };
        let (croquis, start, diagnostics) = draw(source);
        parser_diagnostics += diagnostics;
        assert_eq!(rows, original(source, &croquis, start), "{name}");
        if rows.is_empty() {
            empty += 1;
        } else {
            compare_declarations(&owner);
            resolved += 1;
            uses += rows.len();
        }
    }
    assert_eq!(files, invalid + refused + empty + resolved);
    assert!(resolved > 0 && uses >= resolved);
    eprintln!(
        "component_imports files={files} invalid={invalid} refused={refused} empty={empty} resolved={resolved} uses={uses} oracle_diagnostics={parser_diagnostics}"
    );
}

#[test]
#[ignore = "paired complete component-source requests and retained lookups"]
fn paired_queries() {
    let mut regressions = Vec::new();
    for (size, kebab) in [1, 8, 32, 128]
        .into_iter()
        .flat_map(|size| [(size, false), (size, true)])
    {
        let source = input::component_imports(size, kebab);
        let arena = Allocator::new();
        let owner = observe(&arena, &source);
        let (croquis, start, _) = draw(&source);
        let imports = owner.setup_component_imports().unwrap();
        assert_eq!(product(&owner).unwrap(), original(&source, &croquis, start));
        for cold in [false, true] {
            let run = |candidate| {
                if candidate && cold {
                    let a = Allocator::new();
                    product(&observe(&a, black_box(&source))).unwrap()
                } else if candidate {
                    retained_product(black_box(&imports))
                } else if cold {
                    let (c, s, _) = draw(black_box(&source));
                    original(&source, &c, s)
                } else {
                    original(black_box(&source), black_box(&croquis), start)
                }
            };
            let time = |candidate| {
                let s = Instant::now();
                for _ in 0..8 {
                    drop(black_box(run(candidate)));
                }
                s.elapsed().as_nanos() / 8
            };
            let mut pairs = Vec::new();
            for round in 0..35 {
                let first = round % 2 == 0;
                let a = time(first);
                let b = time(!first);
                if round >= 4 {
                    pairs.push(if first { (b, a) } else { (a, b) });
                }
            }
            eprintln!("component_import_time size={size} kebab={kebab} cold={cold} ns={pairs:?}");
            let (mut left, mut right): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
            left.sort_unstable();
            right.sort_unstable();
            eprintln!(
                "component_import_medians size={size} kebab={kebab} cold={cold} ns={}/{}",
                left[15], right[15]
            );
            if right[15] > left[15] {
                regressions.push((size, kebab, cold, left[15], right[15]));
            }
        }
    }
    assert!(regressions.is_empty(), "{regressions:?}");
}
