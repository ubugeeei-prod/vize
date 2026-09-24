//! TS-9 lane for P4-10a: exact route facts and route-typing diagnostics over
//! `tests/_fixtures/davinci-routes/`, every error witness re-verified (TS-36)
//! and zero undeclared fact accesses (TS-35).
//!
//! Each fixture directory is one project; `expected.txt` in it is the exact
//! rendering of its facts and diagnostics. Regenerate after an intended
//! change with `UPDATE_DAVINCI_ROUTES=1 cargo test -p vize_croquis_cf --test davinci_routes`.
#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

#[path = "davinci_routes/render.rs"]
mod render;

use std::path::{Path, PathBuf};

use vize_croquis_cf::providers::vue_router::typing::{RouteTyping, WITNESS_CHECKS, check};
use vize_croquis_cf::providers::{PROJECT_FACTS, ProjectSources};
use vize_davinci::fact::{FactManager, undeclared_accesses};
use vize_davinci::witness::verify;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-routes")
}

fn files(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            // Running `vize lint` inside a fixture leaves a `node_modules/.vize`
            // schema cache behind; it is not part of the project.
            if path.file_name().is_some_and(|name| name != "node_modules") {
                files(&path, root, out);
            }
        } else if path.file_name().is_some_and(|name| name != "expected.txt") {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, std::fs::read_to_string(&path).unwrap()));
        }
    }
}

fn project_of(dir: &Path) -> ProjectSources {
    let mut sources = Vec::new();
    files(dir, dir, &mut sources);
    let mut project = ProjectSources::new();
    for (path, source) in &sources {
        assert!(
            project.add(path, source).is_some(),
            "{path} was not accepted"
        );
    }
    project
}

#[test]
fn every_fixture_renders_exactly_and_every_witness_verifies() {
    let root = fixtures_root();
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    dirs.retain(|path| path.is_dir());
    dirs.sort();
    let names: Vec<_> = dirs
        .iter()
        .map(|dir| dir.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "demo-app",
            "multi-router",
            "open-tree",
            "param-types",
            "shadowing"
        ]
    );

    let update = std::env::var_os("UPDATE_DAVINCI_ROUTES").is_some();
    let mut stale = Vec::new();
    for dir in &dirs {
        let project = project_of(dir);
        let mut manager = FactManager::new(&PROJECT_FACTS);
        let view = manager.prepare::<RouteTyping>(&project).unwrap();
        let diagnostics = check(&project, &view).unwrap();
        for finding in &diagnostics {
            assert_eq!(
                verify(&finding.diagnostic, &view, &WITNESS_CHECKS),
                Ok(()),
                "{}: {} witness must verify",
                dir.display(),
                finding.code
            );
            // Every route-typing finding carries a fact chain: a proof on an
            // error, the "why" on the missing-param warning.
            assert!(
                finding.diagnostic.witness_chain().is_some(),
                "{}",
                finding.code
            );
        }
        let actual = render::fixture(&project, &view, &diagnostics);
        let expected_path = dir.join("expected.txt");
        if update {
            std::fs::write(&expected_path, &actual).unwrap();
        } else if std::fs::read_to_string(&expected_path).unwrap_or_default() != actual {
            stale.push((expected_path.display().to_string(), actual));
        }
    }
    for (path, actual) in &stale {
        eprintln!("--- {path} (actual)\n{actual}");
    }
    assert!(
        stale.is_empty(),
        "stale fixtures; rerun with UPDATE_DAVINCI_ROUTES=1"
    );
    assert_eq!(
        undeclared_accesses(),
        0,
        "TS-35: zero undeclared fact accesses"
    );
}

#[test]
fn a_forged_unknown_route_witness_against_an_open_tree_fails_verification() {
    use vize_croquis_cf::providers::vue_router::{RouteParams, RouteTree};
    use vize_davinci::diagnostic::{Verdict, WitnessChain, WitnessLink};
    use vize_davinci::fact::FactGroup;
    use vize_davinci::witness::{WitnessError, verify_chain};

    let project = project_of(&fixtures_root().join("open-tree"));
    let mut manager = FactManager::new(&PROJECT_FACTS);
    let view = manager.prepare::<RouteTyping>(&project).unwrap();
    let tree = view.get::<RouteTree>().unwrap().get(&0).unwrap();
    assert!(!tree.is_closed());
    let forged = WitnessChain::new(WitnessLink::of::<RouteTree>(&0, tree.span));
    assert_eq!(
        verify_chain(&forged, &view, &WITNESS_CHECKS),
        Err(WitnessError::NotProven {
            link: 0,
            group: RouteTree::ID,
            verdict: Verdict::Unknown
        })
    );
    // The static `order` route's params stay proven inside the open tree.
    let order = view
        .get::<RouteParams>()
        .unwrap()
        .get(&"order".into())
        .unwrap();
    let proven = WitnessChain::new(WitnessLink::of::<RouteParams>(
        &"order".into(),
        order.name_span,
    ));
    assert_eq!(verify_chain(&proven, &view, &WITNESS_CHECKS), Ok(()));
}
