//! `format(format(x)) == format(x)` over the in-repo Vue corpus and an option
//! matrix (#6098).
//!
//! Every file runs under the defaults, every preset, each non-default option
//! value alone and seeded combinations of several, split across threads.
#![expect(clippy::disallowed_macros, reason = "test diagnostics use std strings")]
#![expect(clippy::disallowed_types, reason = "test diagnostics use std strings")]

use std::path::{Path, PathBuf};

use vize_glyph::{FormatOptions, FormatPreset, format_sfc};

/// Each axis lists its non-default values as JSON.
const AXES: &[(&str, &[&str])] = &[
    ("printWidth", &["40", "80", "120"]),
    ("tabWidth", &["4"]),
    ("useTabs", &["true"]),
    ("semi", &["false"]),
    ("singleQuote", &["true"]),
    ("jsxSingleQuote", &["true"]),
    ("trailingComma", &["\"es5\"", "\"none\""]),
    ("bracketSpacing", &["false"]),
    ("bracketSameLine", &["true"]),
    ("arrowParens", &["\"avoid\""]),
    ("endOfLine", &["\"crlf\"", "\"cr\"", "\"auto\""]),
    ("quoteProps", &["\"consistent\"", "\"preserve\""]),
    ("singleAttributePerLine", &["true"]),
    ("vueIndentScriptAndStyle", &["true"]),
    ("sortAttributes", &["false"]),
    ("attributeSortOrder", &["\"as-written\""]),
    ("mergeBindAndNonBindAttrs", &["true"]),
    ("maxAttributesPerLine", &["1", "3"]),
    (
        "attributeGroups",
        &[r#"[["id"], ["v-*"], [":*", "@*"], ["*"]]"#],
    ),
    ("normalizeDirectiveShorthands", &["false"]),
    ("sortBlocks", &["false"]),
    ("normalizeCssQuotes", &["true"]),
    ("normalizeInlineStyles", &["true"]),
    ("normalizeVForParens", &["true"]),
    ("selfCloseVoidElements", &["true"]),
    ("reindentPreprocessorStyles", &["true"]),
];

const PRESETS: &[(&str, FormatPreset)] = &[
    ("preset=vize", FormatPreset::Vize),
    ("preset=prettier", FormatPreset::Prettier),
    ("preset=compact", FormatPreset::Compact),
    ("preset=documentation", FormatPreset::Documentation),
];

/// Seeded combinations of several non-default values.
const COMBINATIONS: u64 = 16;

const CORPUS_ROOTS: &[&str] = &[
    "tests/_fixtures",
    "crates/vize_glyph",
    "playground",
    "examples",
];

struct OptionSet {
    label: String,
    options: FormatOptions,
}

fn with(overrides: &[(&str, &str)]) -> FormatOptions {
    let mut value = serde_json::to_value(FormatOptions::default()).expect("serialize defaults");
    for (key, json) in overrides {
        value[*key] = serde_json::from_str(json).expect("axis value is JSON");
    }
    serde_json::from_value(value).expect("axis values are valid options")
}

/// xorshift64*, so the combinations stay identical across runs.
fn next(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// Defaults, presets, each non-default value alone, then seeded combinations.
fn option_sets() -> Vec<OptionSet> {
    let mut sets = vec![OptionSet {
        label: "defaults".into(),
        options: FormatOptions::default(),
    }];
    for &(label, preset) in PRESETS {
        let mut options = FormatOptions::default();
        options.apply_preset(preset);
        sets.push(OptionSet {
            label: label.into(),
            options,
        });
    }
    for (key, values) in AXES {
        for value in *values {
            sets.push(OptionSet {
                label: format!("{key}={value}"),
                options: with(&[(key, value)]),
            });
        }
    }
    let mut state = 0x6098_u64;
    for _ in 0..COMBINATIONS {
        let mut chosen = Vec::new();
        for (key, values) in AXES {
            let roll = next(&mut state) as usize;
            if roll % 3 == 0 {
                let value = values
                    .get((roll / 3) % values.len())
                    .copied()
                    .expect("every option axis has values");
                chosen.push((*key, value));
            }
        }
        sets.push(OptionSet {
            label: chosen
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(" "),
            options: with(&chosen),
        });
    }
    sets
}

fn collect_vue_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == "node_modules" || name == "target" || name == "dist")
            {
                continue;
            }
            collect_vue_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            files.push(path);
        }
    }
}

fn is_terminator(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn first_difference(left: &str, right: &str) -> String {
    let line = left
        .split_inclusive(is_terminator)
        .zip(right.split_inclusive(is_terminator))
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| left.split_inclusive(is_terminator).count());
    let at = |text: &str| {
        text.split_inclusive(is_terminator)
            .nth(line)
            .unwrap_or("<eof>")
            .to_owned()
    };
    format!("line {}: {:?} then {:?}", line + 1, at(left), at(right))
}

/// Format each file under every set and report every non-idempotent run.
fn check_file(
    root: &Path,
    path: &Path,
    sets: &[OptionSet],
    violations: &mut Vec<String>,
) -> (usize, usize) {
    let Ok(source) = std::fs::read_to_string(path) else {
        return (0, sets.len());
    };
    let relative = path.strip_prefix(root).unwrap_or(path).display();
    let mut runs = 0;
    let mut refusals = 0;
    for set in sets {
        // Sources the formatter refuses are covered by their own tests.
        let Ok(first) = format_sfc(&source, &set.options) else {
            refusals += 1;
            continue;
        };
        runs += 1;
        match format_sfc(&first.code, &set.options) {
            Ok(second) if second.code == first.code => {}
            Ok(second) => violations.push(format!(
                "{relative} [{}]: {}",
                set.label,
                first_difference(&first.code, &second.code)
            )),
            Err(error) => violations.push(format!(
                "{relative} [{}]: formatted output is refused: {error}",
                set.label
            )),
        }
    }
    (runs, refusals)
}

#[test]
fn formatting_is_idempotent_across_the_option_matrix() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in CORPUS_ROOTS {
        collect_vue_files(&root.join(dir), &mut files);
    }
    assert!(files.len() > 300, "corpus shrank to {} files", files.len());

    let sets = option_sets();
    let workers = std::thread::available_parallelism().map_or(1, usize::from);
    let next_file = std::sync::atomic::AtomicUsize::new(0);
    let (runs, refusals, mut violations) = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let (root, sets, files, next_file) = (&root, &sets, &files, &next_file);
                scope.spawn(move || {
                    let mut violations = Vec::new();
                    let mut runs = 0;
                    let mut refusals = 0;
                    while let Some(path) =
                        files.get(next_file.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let (file_runs, file_refusals) =
                            check_file(root, path, sets, &mut violations);
                        runs += file_runs;
                        refusals += file_refusals;
                    }
                    (runs, refusals, violations)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("matrix worker"))
            .fold(
                (0, 0, Vec::new()),
                |(runs, refusals, mut all), (more, refused, found)| {
                    all.extend(found);
                    (runs + more, refusals + refused, all)
                },
            )
    });
    violations.sort();
    let attempts = files.len() * sets.len();
    assert!(attempts >= 30_000, "matrix shrank to {attempts} cases");
    assert!(
        runs * 10 >= attempts * 9,
        "only {runs} of {attempts} cases formatted successfully; {refusals} were refused"
    );
    assert!(
        violations.is_empty(),
        "{} of {runs} runs are not idempotent:\n{}",
        violations.len(),
        violations.join("\n")
    );
}
