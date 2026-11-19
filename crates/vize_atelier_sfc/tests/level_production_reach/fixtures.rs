//! The committed fixture corpus of the production-reach sweep.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.vue` file under `tests/_fixtures`, sorted, skipping the
/// `_git` submodule corpus (swept only through the env root) and
/// `node_modules`.
pub fn collect_committed_fixtures(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            if child.file_name().is_some_and(|name| {
                name == "node_modules" || name == "_git" || name == "_git-worktrees"
            }) {
                continue;
            }
            collect_committed_fixtures(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

/// Witnesses the Real Project Matrix reduced from production-reach DOM
/// divergences (`canonical-corpus-divergence-promotion.mjs`), compiled on the
/// recorded shape under their corpus-relative name. A generated fixture PR
/// stays red until the product difference is fixed.
#[test]
fn promoted_real_project_reach_fixtures_agree() {
    use vize_atelier_sfc::dom_differential::diff::production_divergence;
    use vize_atelier_sfc::dom_differential::shapes::Shape;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/real-project-reach-reductions");
    let mut directories: Vec<PathBuf> = fs::read_dir(&root)
        .expect("promoted fixture root exists")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    directories.sort();
    let mut divergences = Vec::new();
    for directory in &directories {
        let fingerprint = directory.file_name().and_then(|name| name.to_str());
        let fingerprint = fingerprint.expect("UTF-8 fixture fingerprint");
        assert!(
            fingerprint.len() == 12 && fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid promoted fixture fingerprint: {fingerprint}"
        );
        let source = fs::read_to_string(directory.join("witness.vue.txt")).expect("witness reads");
        let provenance: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(directory.join("provenance.json")).expect("provenance reads"),
        )
        .expect("provenance is JSON");
        assert_eq!(provenance["fingerprint"].as_str(), Some(fingerprint));
        assert_eq!(provenance["observer"].as_str(), Some("reach"));
        let name = provenance["name"].as_str().expect("corpus-relative name");
        let shape = match provenance["shape"].as_str() {
            Some("dom_inline") => Shape::DomInline,
            Some("dom_module") => Shape::DomModule,
            other => panic!("{fingerprint}: unknown promoted shape {other:?}"),
        };
        let options = SfcParseOptions {
            filename: name.into(),
            ..Default::default()
        };
        let descriptor = parse_sfc(&source, options).expect("promoted witness parses");
        assert!(
            descriptor.template.is_some(),
            "{fingerprint}: witness has no template"
        );
        if let Some(found) = production_divergence(&descriptor, name, shape) {
            divergences.push(format!("{fingerprint} {name} [{}]: {found}", shape.id()));
        }
    }
    assert!(divergences.is_empty(), "{}", divergences.join("\n\n"));
}
