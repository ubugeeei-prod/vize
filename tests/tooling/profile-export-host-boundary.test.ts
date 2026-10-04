import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import { profileHostImports } from "./support/davinci-profile-host-imports.ts";
import {
  edges,
  callerChanges,
  calls,
  exportFile,
  exportTests,
  gateChanges,
  hashes,
} from "../../tools/support/levels/profile-export-host-contract.ts";
import {
  prepare,
  validateEdges,
} from "../../tools/support/levels/profile-export-host-preflight.ts";

const read = (file: string) => fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8");
const forbidden = /\bvize_carton::|use vize_carton\b/u;

test("actual JSON hosts retain L0 storage and admit only their exact export symbols", () => {
  for (const [file, declaration] of Object.entries(profileHostImports)) {
    const source = read(file);
    for (const path of [file, file.replaceAll("/", "\\")])
      assert.doesNotMatch(withoutHostRuntimeReferences(source, path), forbidden);
    for (const extra of [
      "use vize_carton::String;",
      "use vize_carton::profiler::Profiler;",
      "use vize_carton::profile_export::*;",
      "use vize_carton::{profile_export::export_report, String};",
      "use vize_carton::profile_export::unreviewed_function;",
    ])
      assert.match(withoutHostRuntimeReferences(source + "\n" + extra, file), forbidden);
    assert.match(
      withoutHostRuntimeReferences(declaration, "davinci/vize_l2/src/lib.rs"),
      forbidden,
    );
  }
});

test("every tracked Rust caller retires the L0 wire API and inherent export method", () => {
  const root = new URL("../../", import.meta.url);
  const files = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of files)
    assert.doesNotMatch(
      read(file),
      /\.export_report\s*\(|vize_l0\s*::\s*profiler\s*::\s*(?:ProfileExport|PROFILE_EXPORT_SCHEMA_VERSION|\{[^}]*\bProfileExport)/u,
      file,
    );
  assert.doesNotMatch(read("davinci/vize_l0/src/profiler.rs"), /ProfileExport|mod export;/u);
  assert.equal(prepare(read).size, 0);
});

test("replay rejects malformed actual callers, metric bodies and wire law before returning writes", () => {
  for (const [file, , after] of callerChanges) {
    const parts = after.split(/(?<=;)\s*(?=use )/u);
    const changed = read(file).replace(
      parts[0],
      parts[0].replace("export_report", "unexpected_exporter"),
    );
    assert.throws(() => prepare((path) => (path === file ? changed : read(path))), /unexpected/u);
  }
  for (const [file, , after] of calls) {
    const changed = read(file).replace(/export_report\(/u, "unexpected_report(");
    assert.throws(
      () => prepare((path) => (path === file ? changed : read(path))),
      /unexpected/u,
      after,
    );
    for (const extra of [
      "other.export_report(&options);",
      "use vize_l0::profiler::{ProfileExportBudget};",
    ])
      assert.throws(
        () => prepare((path) => read(path) + (path === file ? "\n" + extra : "")),
        /unexpected|retired/u,
      );
  }
  for (const file of [exportFile, hashes.assembly[0], exportTests, "davinci/vize_l0/src/profiler/snapshot.rs"])
    assert.throws(
      () => prepare((path) => read(path) + (path === file ? "\nfn unexpected() {}" : "")),
      /unexpected/u,
    );
});

test("host dependency and lock ownership reject aliases, targets, malformed and extra declarations", () => {
  for (const [name, file] of edges) {
    for (const extra of [
      "\n[build-dependencies]\nvize_carton.workspace = true\n",
      "\n[target.'cfg(unix)'.dependencies]\nvize_carton.workspace = true\n",
      '\n[dependencies.alias]\npackage = "vize_carton"\nversion = "*"\n',
      '\n[dependencies]\n"vize_carton"."workspace" = false\n',
    ])
      assert.throws(() => prepare((path) => read(path) + (path === file ? extra : "")));
    const lock = read("Cargo.lock"),
      start = lock.indexOf(`name = "${name}"\n`),
      end = lock.indexOf("\n[[package]]", start);
    const changed =
      lock.slice(0, start) +
      lock.slice(start, end).replace(' "vize_carton",\n', "") +
      lock.slice(end);
    assert.throws(
      () => prepare((path) => (path === "Cargo.lock" ? changed : read(path))),
      /lock ownership/u,
    );
  }
});

test("fresh manifest insertion rejects valid quoted or spaced old dependency keys", () => {
  const old = new Map(
    edges.map(([, file, , declaration]) => [file, read(file).replace(declaration + "\n", "")]),
  );
  const oldRead = (file: string) => old.get(file) ?? read(file);
  validateEdges(oldRead, true);
  for (const [, file, kind] of edges)
    for (const bad of [
      "vize_carton.workspace = false",
      '"vize_carton"."workspace" = false',
      "vize_carton . workspace = false",
    ]) {
      const changed = oldRead(file).replace(`[${kind}]\n`, `[${kind}]\n${bad}\n`);
      assert.throws(
        () => validateEdges((path) => (path === file ? changed : oldRead(path)), true),
        /dependency/u,
      );
    }
});

test("wire identity preserves literal and golden indentation bytes", () => {
  for (const [file, before, after] of [
    [hashes.assembly[0], 'tool: "vize"', 'tool: "vi ze"'],
    [exportTests, '"  \\"schema_version\\": 1,\\n"', '"   \\"schema_version\\": 1,\\n"'],
  ]) {
    assert.ok(read(file).includes(before), "actual reviewed wire witness");
    assert.throws(
      () => prepare((path) => (path === file ? read(path).replace(before, after) : read(path))),
      /wire law|assembly/u,
    );
  }
});

test("replay checks the executable gate companions as well as the real Rust callers", () => {
  for (const [file, before, after] of gateChanges) {
    const source = read(file);
    const compact = (text: string) => text.replace(/\s+/gu, "").replace(/,(?=[}\]])/gu, "");
    assert.ok(compact(source).includes(compact(after)));
    const changed =
      file !== "tests/tooling/davinci-stage-dependencies.test.ts"
        ? source.replace(after, before)
        : source.replace(/(hostRuntime:\s*\[[\s\S]*?)"vize_curator",?/u, '$1"unexpected_curator",');
    assert.notEqual(changed, source);
    assert.throws(() => prepare((path) => (path === file ? changed : read(path))), /gate/u);
  }
});

test("gate literal whitespace and blanket host companion replacement reject before writes", () => {
  const gate = "tests/tooling/support/davinci-host-imports.ts";
  const list = "tests/tooling/davinci-stage-dependencies.test.ts";
  for (const [file, changed] of [
    [
      gate,
      read(gate).replace("davinci-profile-host-imports.ts", "davinci-prof ile-host-imports.ts"),
    ],
    [list, read(list).replace(/(hostRuntime:\s*\[[\s\S]*?)"vize_curator"/u, '$1"vize_cur ator"')],
    [
      "tests/tooling/support/davinci-profile-host-imports.ts",
      'export function withoutProfileHostImports(source: string, file: string) { return "host_profile_export"; }',
    ],
  ]) {
    assert.notEqual(changed, read(file));
    assert.throws(
      () => prepare((path) => (path === file ? changed : read(path))),
      /gate|companion/u,
    );
  }
});

test("the SFC executable dependency law rejects literal drift and partial helpers", () => {
  const file = "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs";
  for (const changed of [
    read(file).replace("vize_carton", "vize_car ton"),
    read(file) + "\nfunction assertPreferredDependencies() {}\n",
  ])
    assert.throws(
      () => prepare((path) => (path === file ? changed : read(path))),
      /exact SFC dependency gate/u,
    );
});
