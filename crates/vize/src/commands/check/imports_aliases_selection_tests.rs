//! Select an authored alias before probing; missing targets cannot change it.

use super::super::{PathAlias, PathAliasResolver, split_pattern};
use super::{CanonicalPathCache, resolve_import_base, resolve_import_base_with_inputs, write};
use std::path::{Path, PathBuf};
use vize_l0::cstr;

fn resolver(root: &Path, definitions: &[(&str, &[&str])]) -> PathAliasResolver {
    PathAliasResolver {
        aliases: definitions
            .iter()
            .map(|(pattern, targets)| {
                let (prefix, suffix, has_wildcard) = split_pattern(pattern);
                PathAlias {
                    prefix,
                    suffix,
                    has_wildcard,
                    targets: targets.iter().map(|target| (*target).into()).collect(),
                    base_dir: root.to_path_buf(),
                }
            })
            .collect(),
        ..Default::default()
    }
}

fn missing_inputs(base: &Path) -> Vec<PathBuf> {
    let mut inputs = vec![base.to_path_buf()];
    let extensions = [
        ".ts", ".tsx", ".d.ts", ".mts", ".d.mts", ".cts", ".d.cts", ".vue",
    ];
    inputs.extend(
        extensions
            .iter()
            .map(|extension| PathBuf::from(cstr!("{}{extension}", base.display()).as_str())),
    );
    inputs.extend(
        extensions
            .iter()
            .map(|extension| base.join(cstr!("index{extension}").as_str())),
    );
    inputs
}

fn assert_resolution(
    resolver: &PathAliasResolver,
    specifier: &str,
    expected: Option<PathBuf>,
    expected_inputs: Vec<PathBuf>,
) {
    let resolved = resolver.resolve(
        specifier,
        &mut CanonicalPathCache::default(),
        false,
        resolve_import_base,
    );
    let with_inputs = resolver.resolve_with_inputs(
        specifier,
        &mut CanonicalPathCache::default(),
        false,
        resolve_import_base_with_inputs,
    );
    assert_eq!(
        (resolved, with_inputs),
        (expected.clone(), (expected, expected_inputs)),
    );
}

#[test]
fn exact_key_wins_an_earlier_matching_wildcard() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    write(&root, "wildcard.ts", "export const value = 1;");
    let selected = write(&root, "selected.ts", "export const value = 'selected';");
    let resolver = resolver(
        &root,
        &[("@x*", &["wildcard.ts"]), ("@x", &["selected.ts"])],
    );
    assert_resolution(&resolver, "@x", Some(selected.clone()), vec![selected]);
}

#[test]
fn missing_exact_target_does_not_consult_a_matching_wildcard() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    write(&root, "wildcard.ts", "export const value = 1;");
    let resolver = resolver(&root, &[("@x", &["missing"]), ("@x*", &["wildcard.ts"])]);
    assert_resolution(&resolver, "@x", None, missing_inputs(&root.join("missing")));
}

#[test]
fn missing_longest_prefix_does_not_consult_a_broader_pattern() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    write(&root, "broaderx.ts", "export const value = 1;");
    let resolver = resolver(&root, &[("@*", &["broader*.ts"]), ("@x*", &["missing*"])]);
    assert_resolution(&resolver, "@x", None, missing_inputs(&root.join("missing")));
}

#[test]
fn equal_wildcard_prefixes_keep_their_incoming_order_even_when_missing() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let selected = write(&root, "selectedy.ts", "export const value = 'selected';");
    let first = resolver(
        &root,
        &[("@x*", &["missing*"]), ("@x*z", &["selected*.ts"])],
    );
    assert_resolution(
        &first,
        "@xyz",
        None,
        missing_inputs(&root.join("missingyz")),
    );
    let reversed = resolver(
        &root,
        &[("@x*z", &["selected*.ts"]), ("@x*", &["missing*"])],
    );
    assert_resolution(&reversed, "@xyz", Some(selected.clone()), vec![selected]);
}

#[test]
fn selected_target_fallback_keeps_all_missing_inputs() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let selected = write(&root, "selected.ts", "export const value = 'selected';");
    write(&root, "broader.ts", "export const value = 1;");
    let resolver = resolver(
        &root,
        &[
            ("@x", &["missing", "selected.ts"]),
            ("@x*", &["broader.ts"]),
        ],
    );
    let mut inputs = missing_inputs(&root.join("missing"));
    inputs.push(selected.clone());
    assert_resolution(&resolver, "@x", Some(selected), inputs);
}

#[test]
fn base_url_fallback_stays_after_only_the_selected_alias_targets() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let selected = write(&root, "base/@x.ts", "export const value = 'base';");
    write(&root, "broader.ts", "export const value = 1;");
    let mut resolver = resolver(&root, &[("@x", &["missing"]), ("@x*", &["broader.ts"])]);
    resolver.base_url = Some(root.join("base"));
    let mut inputs = missing_inputs(&root.join("missing"));
    inputs.extend([root.join("base/@x"), selected.clone()]);
    assert_resolution(&resolver, "@x", Some(selected), inputs);
}
