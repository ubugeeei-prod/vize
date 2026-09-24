//! P3-14 — `vize reduce`, pinned end to end.
//!
//! The acceptance: a **seeded crash** repro (the committed
//! `seeded-crash.repro.folio`, a real SFC with a crash seeded on one nested
//! element via `inject-when`) reduces to at most 20% of its size while the
//! oracle holds — asserted as exact bytes and exact reduced artifact, and
//! re-checked independently by `vize repro` on the output. The seed is shown
//! load-bearing (without it the repro does not reproduce), so the reduction
//! preserved a content-dependent oracle, not a vacuous one. The `.vue` checks
//! (remark, folio, diagnostic, budget, script) and every refusal are pinned
//! with exact output too.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/davinci_reduce")
        .join(name)
}

fn vize(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(args)
        .output()
        .expect("vize runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("UTF-8 output")
}

fn temp_out(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("davinci-reduce");
    std::fs::create_dir_all(&dir).expect("temp dir creates");
    dir.join(name)
}

const FAILURE: &str = "template.transform: injected davinci panic in pass `transform`";

#[test]
fn the_seeded_crash_reduces_below_a_fifth_and_still_reproduces() {
    let input = fixture("seeded-crash.repro.folio");
    let out = temp_out("seeded.reduced.repro.folio");
    let output = vize(&[
        "reduce",
        input.to_str().expect("UTF-8 path"),
        "--out",
        out.to_str().expect("UTF-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        "reduce: 3733 -> 117 bytes (3.1% of the input), 62 oracle run(s), 1-minimal\n"
    );
    // The acceptance bound, stated on the artifact bytes.
    let original = std::fs::read_to_string(fixture("seeded-crash.vue")).expect("fixture reads");
    assert_eq!(original.len(), 3733);
    assert!(117 * 5 <= original.len(), "reduced to at most 20%");

    let reduced = std::fs::read_to_string(&out).expect("reduced repro reads");
    let expected = std::fs::read_to_string(fixture("seeded-crash.reduced.repro.folio"))
        .expect("expected repro reads");
    assert_eq!(reduced, expected);

    // Independent re-check: the reduced repro replays to the recorded
    // failure through `vize repro`'s own exact comparison.
    let replay = vize(&["repro", out.to_str().expect("UTF-8 path")]);
    assert_eq!(replay.status.code(), Some(0));
    assert_eq!(
        text(&replay.stdout),
        format!("repro: reproduced: {FAILURE}\n")
    );
}

#[test]
fn the_seed_is_load_bearing() {
    // The same repro with the seeded element's tag renamed does not
    // reproduce: the crash depends on the content the reducer kept.
    let seeded = std::fs::read_to_string(fixture("seeded-crash.repro.folio")).expect("reads");
    let unseeded = seeded.replace("<seed-crash ", "<seed-calm ");
    let path = temp_out("unseeded.repro.folio");
    std::fs::write(&path, unseeded).expect("writes");
    let replay = vize(&["repro", path.to_str().expect("UTF-8 path")]);
    assert_eq!(replay.status.code(), Some(1));
    assert_eq!(
        text(&replay.stderr),
        format!("repro: did not reproduce: the pipeline completed (recorded {FAILURE})\n")
    );
    let reduce = vize(&["reduce", path.to_str().expect("UTF-8 path")]);
    assert_eq!(reduce.status.code(), Some(1));
    assert_eq!(
        text(&reduce.stderr),
        "reduce: the input does not satisfy the oracle; nothing to preserve\n"
    );
}

fn sorted_entries(dir: &Path) -> Vec<std::string::String> {
    let mut names: Vec<std::string::String> = std::fs::read_dir(dir)
        .expect("directory exists")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("UTF-8")
        })
        .collect();
    names.sort();
    names
}

/// Build one file under `--davinci-inject-panic seeded-crash:transform:seed-crash`.
fn seeded_build(test_name: &str, source: &str) -> (PathBuf, Output) {
    let root = temp_out(test_name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).expect("project dir creates");
    std::fs::write(root.join("src/seeded-crash.vue"), source).expect("source writes");
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&root)
        .args([
            "build",
            "src",
            "--output",
            "dist",
            "--davinci-inject-panic",
            "seeded-crash:transform:seed-crash",
        ])
        .output()
        .expect("vize runs");
    (root, output)
}

#[test]
fn the_seeded_crash_is_what_a_seeded_build_writes() {
    // The committed fixture is exactly the ICE policy's own reproducer: a
    // build seeded on `<seed-crash>` fails the file and writes it.
    let source = std::fs::read_to_string(fixture("seeded-crash.vue")).expect("reads");
    let (root, output) = seeded_build("seeded-build", &source);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        sorted_entries(&root.join("dist")),
        ["seeded-crash.repro.folio"]
    );
    let written =
        std::fs::read_to_string(root.join("dist/seeded-crash.repro.folio")).expect("repro reads");
    let committed = std::fs::read_to_string(fixture("seeded-crash.repro.folio")).expect("reads");
    assert_eq!(written, committed);

    // The same build over the same file without the seeded element compiles
    // it: the trigger is the content, not the file name.
    let calm = source.replace("<seed-crash ", "<seed-calm ");
    let (root, output) = seeded_build("calm-build", &calm);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(sorted_entries(&root.join("dist")), ["seeded-crash.js"]);
}

fn reduce_vue(name: &str, checks: &[&str]) -> Output {
    let input = fixture(name);
    let mut args = vec!["reduce", input.to_str().expect("UTF-8 path")];
    args.extend_from_slice(checks);
    vize(&args)
}

#[test]
fn each_vue_oracle_reduces_to_its_exact_witness() {
    let cases: [(&str, &[&str], &str, &str); 5] = [
        (
            "seeded-crash.vue",
            &["--remark", "s2.hoist-static missed static-props"],
            "<template><section><Swiper><SwiperSlide :key=\"slide.id\"></SwiperSlide></Swiper></section></template>",
            "reduce: 3733 -> 100 bytes (2.6% of the input), 49 oracle run(s), 1-minimal\n",
        ),
        (
            "seeded-crash.vue",
            &["--folio-contains", "ui.for"],
            "<template><section><Swiper><SwiperSlide v-for=\"slide in slides\"></SwiperSlide></Swiper></section></template>",
            "reduce: 3733 -> 108 bytes (2.8% of the input), 49 oracle run(s), 1-minimal\n",
        ),
        (
            "seeded-crash.vue",
            &["--script", "grep -q toggleItalic"],
            "<template><section><article><div><button @click=\"editor?.chain().focus().toggleItalic().run()\"></button></div></article></section></template>",
            "reduce: 3733 -> 141 bytes (3.7% of the input), 62 oracle run(s), 1-minimal\n",
        ),
        (
            "model-scope.vue",
            &["--diagnostic", "v-for or v-slot scope"],
            "<template><form><input v-for=\"item in items\" v-model=\"item\"></form></template>",
            "reduce: 229 -> 78 bytes (34.0% of the input), 42 oracle run(s), 1-minimal\n",
        ),
        (
            "model-scope.vue",
            &[
                "--diagnostic",
                "not supported on plain elements",
                "--walks-over",
                "1",
            ],
            "<template><form><input v-model:value=\"query\"></form></template>",
            "reduce: 229 -> 63 bytes (27.5% of the input), 32 oracle run(s), 1-minimal\n",
        ),
    ];
    for (name, checks, reduced, summary) in cases {
        let output = reduce_vue(name, checks);
        assert_eq!(output.status.code(), Some(0), "{checks:?}");
        assert_eq!(text(&output.stdout), reduced, "{checks:?}");
        assert_eq!(text(&output.stderr), summary, "{checks:?}");
    }
}

#[test]
fn refusals_are_exact() {
    let cases: [(&[&str], i32, &str); 4] = [
        (
            &[],
            2,
            "reduce: a .vue input needs at least one check (--crash, --remark, --folio-contains, \
             --diagnostic, --walks-over, --script)\n",
        ),
        (
            &["--remark", "hoist-static missed static-props"],
            2,
            "reduce: --remark origin `hoist-static` is not `stage.pass`\n",
        ),
        (
            &["--remark", "s2.hoist-static lost static-props"],
            2,
            "reduce: --remark kind `lost` is not applied, missed, or analysis\n",
        ),
        (
            &["--crash"],
            1,
            "reduce: the input does not satisfy the oracle; nothing to preserve\n",
        ),
    ];
    for (checks, code, stderr) in cases {
        let output = reduce_vue("model-scope.vue", checks);
        assert_eq!(output.status.code(), Some(code), "{checks:?}");
        assert_eq!(text(&output.stdout), "", "{checks:?}");
        assert_eq!(text(&output.stderr), stderr, "{checks:?}");
    }
}
