#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! toml = "0.9"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../rust/common.rs"]
mod common;

use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
use toml::Value;

const DIMENSIONS: &[&str] = &[
    "element_kind",
    "directive",
    "modifier_class",
    "binding_source",
    "block_combination",
];
const USAGE: &str = "Usage: rust-script tools/commands/davinci/matrix-gen.rs [--write | --check] [--out-dir <dir>]\n\nGenerates construct-matrix fixture stubs from davinci-road/plan/taxonomy.toml.\nDefault is a dry run that prints the would-be fixture count.";

#[derive(Clone, Debug)]
struct Args {
    write: bool,
    check: bool,
    help: bool,
    out_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct ElementKind {
    id: String,
    representative: String,
}

#[derive(Clone, Debug)]
struct Directive {
    id: String,
    usage: String,
    needs_prior_branch: bool,
}

#[derive(Clone, Debug)]
struct Taxonomy {
    status: String,
    element_kind: Vec<ElementKind>,
    directive: Vec<Directive>,
    dimensions: BTreeMap<String, usize>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err((code, error)) => {
            eprintln!("{error}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<u8, (u8, String)> {
    let args = parse_args(env::args().skip(1).collect())?;
    if args.help {
        println!("{USAGE}");
        return Ok(0);
    }
    let root = repo_root().map_err(|error| (2, error))?;
    let taxonomy = load_taxonomy(&root).map_err(|error| (2, error))?;
    let stubs = generate_stubs(&taxonomy);
    let out_dir = args
        .out_dir
        .unwrap_or_else(|| root.join("tests/fixtures/davinci-matrix"));
    describe(&taxonomy);
    let plane = format!(
        "{} element kinds x {} directives",
        taxonomy.element_kind.len(),
        taxonomy.directive.len()
    );
    if args.check {
        let mut missing = Vec::new();
        let mut stale = Vec::new();
        for (name, content) in &stubs {
            let target = out_dir.join(name);
            if !target.exists() {
                missing.push(name.clone());
            } else if common::read_text(&target).map_err(|error| (2, error))? != *content {
                stale.push(name.clone());
            }
        }
        let extra = if out_dir.exists() {
            fs::read_dir(&out_dir)
                .map_err(|error| (2, format!("cannot read {}: {error}", out_dir.display())))?
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    (name.ends_with(".vue") && !stubs.contains_key(&name)).then_some(name)
                })
                .collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
        };
        if missing.len() + stale.len() + extra.len() > 0 {
            for name in missing {
                println!("missing: {name}");
            }
            for name in stale {
                println!("stale: {name}");
            }
            for name in extra {
                println!("extra: {name}");
            }
            println!(
                "matrix-gen: {} is out of date - rerun with --write and commit",
                relative(&root, &out_dir)
            );
            return Ok(1);
        }
        println!(
            "matrix-gen: {} fixture stubs up to date in {}",
            stubs.len(),
            relative(&root, &out_dir)
        );
        return Ok(0);
    }
    if args.write {
        fs::create_dir_all(&out_dir)
            .map_err(|error| (2, format!("cannot create {}: {error}", out_dir.display())))?;
        for (name, content) in &stubs {
            common::write_text(out_dir.join(name), content).map_err(|error| (2, error))?;
        }
        let extra = fs::read_dir(&out_dir)
            .map_err(|error| (2, format!("cannot read {}: {error}", out_dir.display())))?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                (name.ends_with(".vue") && !stubs.contains_key(&name)).then_some(name)
            })
            .collect::<BTreeSet<_>>();
        println!(
            "matrix-gen: wrote {} fixture stubs ({plane}) to {}",
            stubs.len(),
            relative(&root, &out_dir)
        );
        for name in extra {
            eprintln!("warning: {name} is not part of the generated set (would fail --check)");
        }
        return Ok(0);
    }
    println!(
        "matrix-gen (skeleton): {plane} -> {} fixture stubs",
        stubs.len()
    );
    println!(
        "dry run - nothing written; target {} (--write to emit, --check to verify)",
        relative(&root, &out_dir)
    );
    Ok(0)
}

fn parse_args(argv: Vec<String>) -> Result<Args, (u8, String)> {
    let mut args = Args {
        write: false,
        check: false,
        help: false,
        out_dir: None,
    };
    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--write" => args.write = true,
            "--check" => args.check = true,
            "--out-dir" => {
                index += 1;
                let value = argv
                    .get(index)
                    .ok_or_else(|| (2, "--out-dir requires a directory argument".to_string()))?;
                args.out_dir = Some(PathBuf::from(value));
            }
            "--help" | "-h" => args.help = true,
            arg => return Err((2, format!("unknown argument {arg}"))),
        }
        index += 1;
    }
    if args.write && args.check {
        return Err((2, "--write and --check are mutually exclusive".to_string()));
    }
    Ok(args)
}

fn load_taxonomy(root: &Path) -> Result<Taxonomy, String> {
    let taxonomy_path = root.join("davinci-road/plan/taxonomy.toml");
    let value: Value = toml::from_str(&common::read_text(&taxonomy_path)?)
        .map_err(|error| format!("malformed taxonomy {}: {error}", taxonomy_path.display()))?;
    let id_re = Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    let mut dimensions = BTreeMap::new();
    for dimension in DIMENSIONS {
        let entries = value
            .get(*dimension)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!(
                    "taxonomy {}: missing [[{dimension}]] entries",
                    taxonomy_path.display()
                )
            })?;
        if entries.is_empty() {
            return Err(format!(
                "taxonomy {}: missing [[{dimension}]] entries",
                taxonomy_path.display()
            ));
        }
        let mut ids = BTreeSet::new();
        for entry in entries {
            let id = entry
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| id_re.is_match(id))
                .ok_or_else(|| {
                    format!("taxonomy [[{dimension}]]: entry with missing or non-kebab-case id")
                })?;
            if !ids.insert(id.to_string()) {
                return Err(format!("taxonomy [[{dimension}]]: duplicate id {id}"));
            }
        }
        dimensions.insert((*dimension).to_string(), entries.len());
    }
    let element_kind = value
        .get("element_kind")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|entry| {
            let id = entry.get("id").and_then(Value::as_str).unwrap().to_string();
            let representative = entry
                .get("representative")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("taxonomy [[element_kind]] {id}: representative tag is required")
                })?
                .to_string();
            Ok(ElementKind { id, representative })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let directive = value
        .get("directive")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|entry| {
            let id = entry.get("id").and_then(Value::as_str).unwrap().to_string();
            let usage = entry
                .get("usage")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("taxonomy [[directive]] {id}: usage attribute text is required")
                })?
                .to_string();
            Ok(Directive {
                id,
                usage,
                needs_prior_branch: entry
                    .get("needs_prior_branch")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let status = value
        .get("taxonomy")
        .and_then(Value::as_table)
        .and_then(|table| table.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    Ok(Taxonomy {
        status,
        element_kind,
        directive,
        dimensions,
    })
}

fn generate_stubs(taxonomy: &Taxonomy) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for kind in &taxonomy.element_kind {
        for directive in &taxonomy.directive {
            let name = format!("{}--{}.vue", kind.id, directive.id);
            let mut lines = vec![
                "<!--".to_string(),
                "  generated-by: tools/commands/davinci/matrix-gen.rs (P0-12 skeleton) \u{2014} do not edit"
                    .to_string(),
                format!("  construct: element_kind={} directive={}", kind.id, directive.id),
                "-->".to_string(),
                "<template>".to_string(),
            ];
            if directive.needs_prior_branch {
                lines.push("  <div v-if=\"first\"></div>".to_string());
            }
            lines.push(format!(
                "  <{} {}></{}>",
                kind.representative, directive.usage, kind.representative
            ));
            lines.push("</template>".to_string());
            files.insert(name, format!("{}\n", lines.join("\n")));
        }
    }
    files
}

fn describe(taxonomy: &Taxonomy) {
    let counts = DIMENSIONS
        .iter()
        .map(|dimension| format!("{dimension}={}", taxonomy.dimensions[*dimension]))
        .collect::<Vec<_>>()
        .join(" ");
    println!("taxonomy: {counts} ({})", taxonomy.status);
}

fn relative(root: &Path, target: &Path) -> String {
    target
        .strip_prefix(root)
        .ok()
        .map(common::normalize_path)
        .unwrap_or_else(|| target.display().to_string())
}

fn repo_root() -> Result<PathBuf, String> {
    common::repo_root().or_else(|_| {
        Path::new(file!())
            .ancestors()
            .find(|candidate| {
                candidate.join("Cargo.toml").is_file()
                    && candidate.join("pnpm-workspace.yaml").is_file()
            })
            .map(Path::to_path_buf)
            .ok_or_else(|| "cannot resolve Vize repository root from script path".to_string())
    })
}
