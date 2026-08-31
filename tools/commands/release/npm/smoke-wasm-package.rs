#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;

use serde_json::Value;
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const REQUIRED_FILES: &[&str] = &[
    "index.js",
    "index.d.ts",
    "lint-format.d.ts",
    "vize_vitrine.js",
    "vize_vitrine.d.ts",
    "vize_vitrine_bg.wasm",
];

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [package_dir] = args.as_slice() else {
        return Err(
            "Usage: rust-script tools/commands/release/npm/smoke-wasm-package.rs <npm/wasm>"
                .to_string(),
        );
    };
    let package_dir = PathBuf::from(package_dir)
        .canonicalize()
        .map_err(|error| format!("cannot resolve package directory: {error}"))?;
    let package_json = common::read_json(package_dir.join("package.json"))?;
    assert_manifest(&package_json)?;
    assert_files_exist(&package_dir)?;
    assert_entry_point(&package_dir)?;
    println!("@vizejs/wasm package smoke passed");
    Ok(())
}

fn assert_manifest(package_json: &Value) -> Result<(), String> {
    expect_string(package_json, &["name"], "@vizejs/wasm")?;
    expect_string(package_json, &["type"], "module")?;
    expect_string(package_json, &["main"], "./index.js")?;
    expect_string(package_json, &["types"], "./index.d.ts")?;
    expect_string(package_json, &["exports", ".", "import"], "./index.js")?;
    expect_string(package_json, &["exports", ".", "types"], "./index.d.ts")?;
    expect_string(
        package_json,
        &["exports", "./vize_vitrine.js", "import"],
        "./vize_vitrine.js",
    )?;
    expect_string(
        package_json,
        &["exports", "./vize_vitrine.js", "types"],
        "./vize_vitrine.d.ts",
    )?;
    expect_string(
        package_json,
        &["exports", "./vize_vitrine_bg.wasm"],
        "./vize_vitrine_bg.wasm",
    )?;

    let files = package_json
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "package files must be an array".to_string())?;
    for required in REQUIRED_FILES {
        if !files.iter().any(|file| file.as_str() == Some(required)) {
            return Err(format!("package files must include {required}"));
        }
    }
    Ok(())
}

fn assert_files_exist(package_dir: &Path) -> Result<(), String> {
    for file in REQUIRED_FILES {
        let path = package_dir.join(file);
        if !path.is_file() {
            return Err(format!("{file} is missing"));
        }
    }
    Ok(())
}

fn expect_string(package_json: &Value, path: &[&str], expected: &str) -> Result<(), String> {
    let actual = path
        .iter()
        .try_fold(package_json, |value, key| value.get(*key))
        .and_then(Value::as_str);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "package.json {} must be {expected}",
            path.join(".")
        ))
    }
}

fn assert_entry_point(package_dir: &Path) -> Result<(), String> {
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", WASM_RUNTIME_SMOKE])
        .env("VIZE_WASM_SMOKE_PACKAGE_DIR", package_dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run node wasm smoke: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "wasm runtime smoke failed\n{}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    Ok(())
}

const WASM_RUNTIME_SMOKE: &str = r#"
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const packageDir = process.env.VIZE_WASM_SMOKE_PACKAGE_DIR;
if (!packageDir) throw new Error("VIZE_WASM_SMOKE_PACKAGE_DIR is required");
const expectedExports = [
  "Compiler",
  "compile",
  "compileCss",
  "compileJsx",
  "compileSfc",
  "compileVapor",
  "default",
  "formatSfc",
  "init",
  "isInitialized",
  "lintSfc",
  "parseSfc",
  "parseTemplate",
];

function assertCompilerOptions(entry) {
  const moduleResult = entry.compile("<div>{{ message }}</div>", {
    mode: "module",
    runtimeModuleName: "@acme/vue-runtime",
    sourceMap: true,
    filename: "src/Component.vue",
  });
  assert.match(moduleResult.preamble, /@acme\/vue-runtime/);
  assert.equal(moduleResult.map?.version, 3);
  assert.deepEqual(moduleResult.map?.sources, ["src/Component.vue"]);

  const functionResult = entry.compile("<div></div>", {
    mode: "function",
    runtimeGlobalName: "AcmeVue",
  });
  assert.match(`${functionResult.preamble}\n${functionResult.code}`, /AcmeVue/);

  const legacyQuirks = entry.compile("<div /><span></span>", { vueParserQuirks: true });
  assert.equal(legacyQuirks.ast.children[0].isSelfClosing, true);
  assert.throws(() => entry.compile("<div /><span></span>", { templateSyntax: "strict" }));
  const parsedQuirks = entry.parseTemplate("<div />", { vueParserQuirks: true });
  assert.equal(parsedQuirks.children[0].isSelfClosing, true);
  assert.throws(() =>
    entry.parseTemplate("<div />", {
      templateSyntax: "standard",
      vueParserQuirks: true,
    }),
  );

  const typedSfc = '<script setup lang="ts">const count: number = 1</script>';
  const preserved = entry.compileSfc(typedSfc, { scriptExt: "preserve" });
  const downcompiled = entry.compileSfc(typedSfc, { scriptExt: "downcompile" });
  assert.match(preserved.script.code, /count:\s*number/);
  assert.doesNotMatch(downcompiled.script.code, /count:\s*number/);

  const sfcSource = "<template><div></div></template>";
  const moduleSfc = entry.compileSfc(sfcSource, {
    runtimeModuleName: "@acme/vue-runtime",
    sourceMap: true,
  });
  assert.match(moduleSfc.script.code, /@acme\/vue-runtime/);
  assert.deepEqual(moduleSfc.template.map?.sources, ["anonymous.vue"]);
  const standaloneSfc = entry.compileSfc(sfcSource, {
    mode: "function",
    runtimeModuleName: "@acme/vue-runtime",
    runtimeGlobalName: "AcmeVue",
  });
  assert.match(standaloneSfc.script.code, /AcmeVue/);
  assert.doesNotMatch(standaloneSfc.script.code, /@acme\/vue-runtime/);
}

const entry = await import(
  `${pathToFileURL(path.join(packageDir, "index.js")).href}?smoke=${Date.now()}`
);
assert.deepEqual(Object.keys(entry).sort(), expectedExports);

assert.equal(entry.default, entry.init);
assert.equal(entry.isInitialized(), false);
assert.throws(() => entry.compile("<div />"), /Call `await init\(\)` first/);
assert.throws(() => entry.lintSfc("<template />"), /Call `await init\(\)` first/);
assert.throws(() => entry.formatSfc("<template />"), /Call `await init\(\)` first/);

await assert.rejects(() => entry.init(new Uint8Array()), /.+/);
assert.equal(entry.isInitialized(), false);

const failedUrl = new URL("https://vize.invalid/vize_vitrine_bg.wasm");
const originalFetch = globalThis.fetch;
globalThis.fetch = async (input, init) => {
  const inputUrl =
    typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
  if (inputUrl === failedUrl.href) {
    throw new Error("intentional WASM fetch failure");
  }
  return originalFetch(input, init);
};
try {
  await assert.rejects(() => entry.init(failedUrl), /intentional WASM fetch failure/);
} finally {
  globalThis.fetch = originalFetch;
}
assert.equal(entry.isInitialized(), false);

const wasm = fs.readFileSync(path.join(packageDir, "vize_vitrine_bg.wasm"));
await entry.init(wasm);
assert.equal(entry.isInitialized(), true);
assertCompilerOptions(entry);

const source = '<template>\n  <div  id="x"></div>\n</template>\n';
const formattedSource = '<template>\n  <div id="x"></div>\n</template>\n';
const lintResult = entry.lintSfc(source, {
  filename: "Smoke.vue",
  locale: "en",
  enabledRules: ["vue/no-multi-spaces"],
});
assert.equal(lintResult.filename, "Smoke.vue");
assert.equal(lintResult.errorCount, 0);
assert.equal(lintResult.warningCount, 1);
assert.equal(lintResult.diagnostics.length, 1);
assert.equal(lintResult.diagnostics[0].rule, "vue/no-multi-spaces");
assert.equal(lintResult.diagnostics[0].severity, "warning");
assert.equal(lintResult.diagnostics[0].location.start.offset, 17);
assert.equal(lintResult.diagnostics[0].help, undefined);

const formatResult = entry.formatSfc(source, { printWidth: 80 });
assert.equal(formatResult.code, formattedSource);
assert.equal(formatResult.changed, true);
"#;
