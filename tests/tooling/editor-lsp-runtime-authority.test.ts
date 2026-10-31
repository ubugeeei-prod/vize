import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const runtimeVariables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];

test("editor fixture runtime authority reaches only the child and preserves explicit choices", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "editor-runtime-authority-"));
  try {
    const support = fileURLToPath(new URL("../../tools/support", import.meta.url));
    const launcher = path.join(directory, "runtime-authority.rs");
    fs.writeFileSync(
      launcher,
      `//! \x60\x60\x60cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! vize_tool_support = { path = ${JSON.stringify(support)} }
//! serde_json = "1"
//! \x60\x60\x60
use vize_tool_support::editors::lsp_smoke::LspSession;
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let root = std::path::Path::new(&args[1]);
    let runtime = std::path::Path::new(&args[3]);
    let session = if args[2] == "inherited" {
        LspSession::spawn(root)
    } else {
        LspSession::spawn_with_runtime(root, runtime)
    };
    let mut session = match session {
        Ok(session) => session,
        Err(message) => { eprintln!("{message}"); std::process::exit(1); }
    };
    let child = session.request("fixture/runtimeAuthority", serde_json::json!({})).unwrap();
    let variables = ${JSON.stringify(runtimeVariables)};
    let parent = variables.iter().filter_map(|name| std::env::var(name).ok().map(|value| (name, value))).collect::<std::collections::BTreeMap<_, _>>();
    println!("{}", serde_json::json!({"child":child,"parent":parent}));
    session.shutdown().unwrap();
}
`,
    );
    // Synthetic transport law only: these processes provide no native checker credit.
    fs.writeFileSync(
      path.join(directory, "lsp"),
      `const fs = require("node:fs");
fs.writeFileSync(process.env.EDITOR_RUNTIME_MARKER, "child started");
let bytes = Buffer.alloc(0);
process.stdin.on("data", chunk => {
  bytes = Buffer.concat([bytes, chunk]);
  while (true) {
    const separator = bytes.indexOf("\\r\\n\\r\\n");
    if (separator < 0) return;
    const length = Number(/Content-Length: (\\d+)/.exec(bytes.subarray(0, separator).toString())[1]);
    if (bytes.length < separator + 4 + length) return;
    const request = JSON.parse(bytes.subarray(separator + 4, separator + 4 + length));
    bytes = bytes.subarray(separator + 4 + length);
    if (request.method === "exit") process.exit(0);
    if (request.id === undefined) continue;
    const runtime = Object.fromEntries(${JSON.stringify(runtimeVariables)}.filter(name => process.env[name] !== undefined).map(name => [name, process.env[name]]));
    const body = JSON.stringify({jsonrpc:"2.0",id:request.id,result:request.method === "shutdown" ? null : runtime});
    process.stdout.write("Content-Length: " + Buffer.byteLength(body) + "\\r\\n\\r\\n" + body);
  }
});
`,
    );
    const selected = path.join(directory, "selected provider é 日本");
    fs.writeFileSync(selected, "inert native authority file; never executed\n");
    const marker = path.join(directory, "child-started");
    const run = (mode: string, runtime: string, choices: Record<string, string> = {}) => {
      fs.rmSync(marker, { force: true });
      const environment = { ...process.env };
      for (const variable of runtimeVariables) delete environment[variable];
      const result = spawnSync("rust-script", ["--debug", launcher, directory, mode, runtime], {
        encoding: "utf8",
        timeout: 120_000,
        env: {
          ...environment,
          ...choices,
          VIZE_LSP_BIN: process.execPath,
          EDITOR_RUNTIME_MARKER: marker,
        },
      });
      assert.ifError(result.error);
      assert.equal(result.signal, null);
      return result;
    };
    const selectedResult = run("selected", selected);
    assert.equal(selectedResult.status, 0, selectedResult.stderr);
    assert.deepEqual(JSON.parse(selectedResult.stdout), {
      child: { CORSA_PATH: fs.realpathSync(selected) },
      parent: {},
    });

    const inherited = { CORSA_PATH: "/caller/selected/runtime" };
    const defaultResult = run("inherited", selected, inherited);
    assert.equal(defaultResult.status, 0, defaultResult.stderr);
    assert.deepEqual(JSON.parse(defaultResult.stdout), { child: inherited, parent: inherited });

    for (const variable of runtimeVariables) {
      const choices = { [variable]: `/caller/missing/${variable}` };
      const result = run("selected", path.join(directory, "missing fallback"), choices);
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(JSON.parse(result.stdout), { child: choices, parent: choices });
    }

    const emptyChoices = Object.fromEntries(runtimeVariables.map((name) => [name, ""]));
    const empty = run("selected", selected, emptyChoices);
    assert.equal(empty.status, 0, empty.stderr);
    assert.deepEqual(JSON.parse(empty.stdout), {
      child: { ...emptyChoices, CORSA_PATH: fs.realpathSync(selected) },
      parent: emptyChoices,
    });

    for (const runtime of [path.join(directory, "missing runtime"), directory]) {
      const result = run("selected", runtime);
      assert.equal(result.status, 1);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /fixture native runtime/);
      assert.equal(fs.existsSync(marker), false, "invalid authority must fail before child launch");
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
