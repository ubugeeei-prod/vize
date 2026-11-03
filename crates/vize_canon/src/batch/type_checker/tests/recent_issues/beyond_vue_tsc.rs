//! Diagnostics Vize reports where vue-tsc 3.3.11 stays silent (#11655).
//!
//! Each `tests/_fixtures/type-checking-beyond-vue-tsc/<case>` holds authored
//! code that is wrong (`src/`), the corrected files (`fixed/src/`), and the
//! exact diagnostics for both (`expected.txt`, `fixed/expected.txt`). The docs
//! page quotes these files byte for byte, and
//! `tests/tooling/docs-type-checking-beyond-vue-tsc.test.ts` checks that it
//! does, so the page cannot drift from what the checker reports.
//!
//! Set `VIZE_BLESS_BEYOND_VUE_TSC=1` to rewrite the expectations.

use std::path::{Path, PathBuf};

use vize_carton::{String, append, cstr};

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/type-checking-beyond-vue-tsc")
}

/// `src/*.vue.txt` under `dir`, as (`src/X.vue`, contents).
fn sources(dir: &Path) -> Vec<(String, String)> {
    let mut files: Vec<_> = std::fs::read_dir(dir.join("src"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.strip_suffix(".txt")?;
            let source = std::fs::read_to_string(&path).unwrap();
            Some((cstr!("src/{name}"), String::from(source.as_str())))
        })
        .collect();
    files.sort();
    files
}

/// (file, line, column, code, severity, message).
type Row<'a> = (&'a str, u32, u32, Option<u32>, &'a str, &'a str);

/// The `vize check` text format: one header per file, then
/// `  error:LINE:COLUMN [TSCODE] message` rows in source order.
fn render(snapshot: &[(String, Option<u32>, String)]) -> String {
    let mut rows: Vec<Row<'_>> = snapshot
        .iter()
        .map(|(file, code, row)| {
            // `snapshot_project_diagnostics` rows read `LINE:COLUMN:SEVERITY message`.
            let (position, message) = row.split_once(' ').unwrap();
            let mut position = position.splitn(3, ':');
            let (line, column, severity) = (
                position.next().unwrap(),
                position.next().unwrap(),
                position.next().unwrap(),
            );
            (
                file.as_str(),
                line.parse().unwrap(),
                column.parse().unwrap(),
                *code,
                severity,
                message,
            )
        })
        .collect();
    rows.sort();
    let mut out = String::default();
    let mut current = None;
    for (file, line, column, code, severity, message) in rows {
        if current != Some(file) {
            append!(out, "{file}\n");
            current = Some(file);
        }
        let code = code.map_or_else(String::default, |code| cstr!(" [TS{code}]"));
        let message = message.replace('\n', "\n    ");
        append!(out, "  {severity}:{line}:{column}{code} {message}\n");
    }
    out
}

fn assert_case(name: &str, label: &str, files: &[(String, String)], expected: &Path) {
    let files: Vec<_> = files
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect();
    let Some(snapshot) =
        snapshot_project_diagnostics(&create_project_case(&cstr!("bvt-{name}-{label}"), &files))
    else {
        return;
    };
    let actual = render(&snapshot);
    if std::env::var_os("VIZE_BLESS_BEYOND_VUE_TSC").is_some() {
        std::fs::write(expected, actual.as_str()).unwrap();
        return;
    }
    let want = std::fs::read_to_string(expected).unwrap_or_default();
    assert_eq!(
        actual.as_str(),
        want,
        "{name} ({label}): {}",
        expected.display()
    );
}

#[test]
fn beyond_vue_tsc_cases_pin_authored_and_fixed_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let mut cases: Vec<_> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    cases.sort();
    assert!(cases.len() >= 3, "{cases:?}");
    for case in cases {
        let name = case.file_name().unwrap().to_str().unwrap();
        let authored = sources(&case);
        assert_case(name, "authored", &authored, &case.join("expected.txt"));
        assert!(
            !std::fs::read_to_string(case.join("expected.txt"))
                .unwrap_or_default()
                .is_empty(),
            "{name}: the authored code must report a diagnostic"
        );

        let mut fixed = authored;
        for (path, source) in sources(&case.join("fixed")) {
            let slot = fixed.iter_mut().find(|(existing, _)| *existing == path);
            slot.unwrap().1 = source;
        }
        assert_case(name, "fixed", &fixed, &case.join("fixed/expected.txt"));
    }
}
