#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! toml_edit = "0.22"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../rust/common.rs"]
mod common;

use std::{env, path::Path, process::ExitCode};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let mut amends = None;
    let mut files = Vec::new();
    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--amends=") {
            amends = Some(value.to_string());
        } else {
            files.push(arg);
        }
    }
    if files.is_empty() {
        return Err("usage: rust-script tools/commands/fixtures/toml-to-pkl.rs [--amends=Schema.pkl] <fixture.toml> ...".to_string());
    }
    for file in files {
        let pkl = convert(Path::new(&file), amends.as_deref())?;
        let pkl_path = file.strip_suffix(".toml").unwrap_or(&file).to_string() + ".pkl";
        common::write_text(&pkl_path, &pkl)?;
        println!("{file} -> {pkl_path}");
    }
    Ok(())
}

fn convert(toml_path: &Path, amends: Option<&str>) -> Result<String, String> {
    let text = common::read_text(toml_path)?;
    let data = text
        .parse::<DocumentMut>()
        .map_err(|error| format!("cannot parse {}: {error}", toml_path.display()))?;
    let mut out = leading_comment(&text);
    if !out.is_empty() {
        out.push(String::new());
    }
    if let Some(amends) = amends {
        out.push(format!("amends {}", pkl_inline_string(amends)));
        out.push(String::new());
    }
    for (key, value) in data.as_table().iter() {
        if amends.is_some() && value.as_array_of_tables().is_some() {
            out.push(format!("{key} {{"));
            let array = value.as_array_of_tables().unwrap();
            for child in array.iter() {
                out.push("  new {".to_string());
                for (child_key, child_value) in child.iter() {
                    out.push(format!("    {child_key} = "));
                    emit_item(child_value, 2, &mut out)?;
                }
                out.push("  }".to_string());
            }
            out.push("}".to_string());
        } else {
            out.push(format!("{key} = "));
            emit_item(value, 0, &mut out)?;
        }
        out.push(String::new());
    }
    while out.last().map(|line| line.is_empty()).unwrap_or(false) {
        out.pop();
    }
    Ok(format!("{}\n", out.join("\n")))
}

fn leading_comment(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim();
        if let Some(comment) = trimmed.strip_prefix('#') {
            out.push(format!("//{comment}"));
        } else if trimmed.is_empty() {
            if !out.is_empty() {
                break;
            }
        } else {
            break;
        }
    }
    out
}

fn pkl_inline_string(value: &str) -> String {
    if !value.contains('"') && !value.contains('\\') {
        return format!("\"{value}\"");
    }
    let mut fence_len = 1usize;
    while value.contains(&format!("\"{}", "#".repeat(fence_len))) {
        fence_len += 1;
    }
    let fence = "#".repeat(fence_len);
    format!("{fence}\"{value}\"{fence}")
}

fn pkl_multiline(value: &str, indent: usize) -> Vec<String> {
    let pad = " ".repeat(indent);
    let lines = value.split('\n').collect::<Vec<_>>();
    let body = if lines.len() > 1 && lines.last() == Some(&"") {
        &lines[..lines.len() - 1]
    } else {
        &lines[..]
    };
    let mut out = vec!["\"\"\"".to_string()];
    for line in body {
        if line.is_empty() {
            out.push(String::new());
        } else {
            out.push(format!("{pad}{line}"));
        }
    }
    if value.ends_with('\n') {
        out.push(String::new());
    }
    out.push(format!("{pad}\"\"\""));
    out
}

fn emit_item(item: &Item, indent: usize, out: &mut Vec<String>) -> Result<(), String> {
    if let Some(value) = item.as_value() {
        return emit_value(value, indent, out);
    }
    if let Some(array) = item.as_array_of_tables() {
        return emit_array_of_tables(array.iter(), indent, out);
    }
    if let Some(table) = item.as_table() {
        return emit_table(table, indent, out);
    }
    Err(format!("unsupported TOML item: {item:?}"))
}

fn emit_value(value: &Value, indent: usize, out: &mut Vec<String>) -> Result<(), String> {
    let pad = "  ".repeat(indent);
    let last = out.len() - 1;
    if let Some(value) = value.as_bool() {
        out[last].push_str(if value { "true" } else { "false" });
    } else if let Some(value) = value.as_integer() {
        out[last].push_str(&value.to_string());
    } else if let Some(value) = value.as_float() {
        out[last].push_str(&value.to_string());
    } else if let Some(value) = value.as_str() {
        if value.contains('\n') {
            let key_col = out[last].len() - out[last].trim_start_matches(' ').len();
            let rendered = pkl_multiline(value, key_col);
            out[last].push_str(&rendered[0]);
            out.extend(rendered.into_iter().skip(1));
        } else {
            out[last].push_str(&pkl_inline_string(value));
        }
    } else if let Some(array) = value.as_array() {
        emit_array(array, indent, out)?;
    } else {
        return Err(format!("unsupported TOML value: {value:?}"));
    }
    if matches!(value, Value::Array(_)) {
        let _ = pad;
    }
    Ok(())
}

fn emit_array(array: &Array, indent: usize, out: &mut Vec<String>) -> Result<(), String> {
    let pad = "  ".repeat(indent);
    let last = out.len() - 1;
    out[last].push_str("new Listing {");
    for child in array.iter() {
        out.push(format!("{pad}  "));
        emit_value(child, indent + 1, out)?;
    }
    out.push(format!("{pad}}}"));
    Ok(())
}

fn emit_array_of_tables<'a>(
    array: impl Iterator<Item = &'a Table>,
    indent: usize,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let pad = "  ".repeat(indent);
    let last = out.len() - 1;
    out[last].push_str("new Listing {");
    for child in array {
        out.push(format!("{pad}  new {{"));
        for (key, value) in child.iter() {
            out.push(format!("{pad}    {key} = "));
            emit_item(value, indent + 2, out)?;
        }
        out.push(format!("{pad}  }}"));
    }
    out.push(format!("{pad}}}"));
    Ok(())
}

fn emit_table(table: &Table, indent: usize, out: &mut Vec<String>) -> Result<(), String> {
    let pad = "  ".repeat(indent);
    let last = out.len() - 1;
    out[last].push_str("new {");
    for (key, value) in table.iter() {
        out.push(format!("{pad}  {key} = "));
        emit_item(value, indent + 1, out)?;
    }
    out.push(format!("{pad}}}"));
    Ok(())
}
