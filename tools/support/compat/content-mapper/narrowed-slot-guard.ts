/** Complete published mapper and pre-projection native vectors for #10660. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { nativeCapture, observe, type ProcessPacket } from "./native-capture.ts";

const options = new Map<string, string>();
for (let index = 2; index < process.argv.length; index += 2) {
  const key = process.argv.at(index),
    value = process.argv.at(index + 1);
  assert.ok(
    key &&
      value &&
      ["--consumer", "--candidate", "--source", "--output"].includes(key) &&
      !options.has(key),
  );
  options.set(key, value);
}
const required = (key: string) => {
  const value = options.get(key);
  assert.ok(value, key);
  return value;
};
const consumer = fs.realpathSync(required("--consumer")),
  candidate = fs.realpathSync(required("--candidate")),
  sourceSha = required("--source"),
  output = path.resolve(required("--output"));
assert.match(sourceSha, /^[a-f0-9]{40}$/u);
assert.equal(fs.existsSync(output), false);
fs.mkdirSync(output, { recursive: true });
const repository = fileURLToPath(new URL("../../../../", import.meta.url)),
  fixtures = path.join(
    repository,
    "tests/_fixtures/differential/typechecker/narrowed-slot-host-guard-10660",
  );
const read = (file: string) => fs.readFileSync(path.join(fixtures, file), "utf8");
const original = read("src/Status.vue.txt"),
  box = read("src/Box.vue.txt");
const impossible = `This comparison appears to be unintentional because the types '"error" | "success"' and '"pending"' have no overlap.`;
const importError =
  "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.";
const missing = "Property 'missing' does not exist on type 'string'.";
type Case = {
  id: string;
  parent: string;
  child: string;
  parentFile: string;
  childFile: string;
  code?: number;
  message?: string;
  token?: string;
  guarded: boolean;
};
const standard = (id: string, parent = original, child = box): Case => ({
  id,
  parent,
  child,
  parentFile: "Status.vue",
  childFile: "Box.vue",
  guarded: true,
});
const cases: Case[] = [
  standard("reporter"),
  {
    ...standard(
      "implicit",
      original.replace("<template #default>{{ fruit }}</template>", "{{ fruit }}"),
    ),
    guarded: false,
  },
  {
    ...standard(
      "no-loop",
      original.replace(' v-for="fruit in fruits" :key="fruit"', "").replace("{{ fruit }}", "slot"),
    ),
    guarded: false,
  },
  standard(
    "named",
    original.replace("#default", "#body"),
    box.replace("<slot />", '<slot name="body" />'),
  ),
  standard(
    "quoted",
    original.replace(`v-if="status === 'pending'"`, `v-if='status === "pending"'`),
  ),
  standard(
    "multiline-props",
    original.replace(':key="fruit"', ':key="fruit"\n      class="case"\n      :title="fruit"'),
  ),
  {
    ...standard("authored", original.replace("{{ fruit }}", "{{ status === 'pending' }}")),
    code: 2367,
    message: impossible,
    token: "status",
  },
  {
    ...standard("slot-body", original.replace("{{ fruit }}", "{{ fruit.missing }}")),
    code: 2339,
    message: missing,
    token: "missing",
  },
  {
    ...standard(
      "wrong-prop",
      original
        .replace("<template #default>{{ fruit }}</template>", "{{ fruit }}")
        .replace(':key="fruit"', ':key="fruit" :label="\'bad\'"'),
      '<script setup lang="ts">defineProps<{label:number}>()</script>\n' + box,
    ),
    guarded: false,
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    token: "label",
  },
];
const oldRoot = path.join(
  repository,
  "crates/vize/tests/fixtures/content_mapper_nested_vfor_guard",
);
cases.push({
  id: "previous-10216",
  parent: fs.readFileSync(path.join(oldRoot, "src/App.vue"), "utf8"),
  child: fs.readFileSync(path.join(oldRoot, "src/MyCell.vue"), "utf8"),
  parentFile: "App.vue",
  childFile: "MyCell.vue",
  guarded: false,
});
const env = Object.fromEntries(
  Object.entries(process.env).filter(
    ([key]) =>
      !/^(VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/u.test(key) &&
      !["NODE_OPTIONS", "NODE_PATH"].includes(key),
  ),
);
env.PATH = path.dirname(process.execPath) + path.delimiter + (env.PATH ?? "");
const packageRoot = (name: string) => fs.realpathSync(path.join(consumer, "node_modules", name));
for (const [name, version] of [
  ["vize", "0.569.0"],
  ["typescript", "7.1.0-beta"],
  ["vue", "3.5.44"],
])
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(packageRoot(name!), "package.json"), "utf8")).version,
    version,
  );
const vize = packageRoot("vize"),
  baseline = [process.execPath, path.join(vize, "bin/vize")];
const official = fs.realpathSync(
  (
    await import(pathToFileURL(path.join(packageRoot("typescript"), "lib/getExePath.js")).href)
  ).default(),
);
const nativeRoot = path.dirname(
    createRequire(path.join(vize, "package.json")).resolve(
      `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
    ),
  ),
  native = fs.realpathSync(path.join(nativeRoot, "lib/tsc"));
const digest = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
save(path.join(output, "providers.json"), {
  sourceSha,
  notPublicationAcceptance: true,
  node: process.version,
  packages: ["vize", "typescript", "vue"].map((name) =>
    JSON.parse(fs.readFileSync(path.join(packageRoot(name), "package.json"), "utf8")),
  ),
  physical: [official, native, baseline[1]!, candidate, process.execPath].map((file) => ({
    file,
    sha256: digest(file),
  })),
  scope:
    "Unwrapped CLI plus actual native child before diagnostic projection; wrapper lifecycle authentication is not claimed",
});
for (const [name, binary, version] of [
  ["official", official, "Version 7.1.0-beta\n"],
  ["native", native, "Version 7.0.2\n"],
]) {
  const r = observe([binary!, "--version"], consumer, path.join(output, name!), env);
  assert.equal(r.status, 0);
  assert.equal(r.stderr, "");
  assert.equal(r.stdout, version);
}
const observations = [];
for (const fixture of cases)
  for (const newline of ["\n", "\r\n"])
    for (const prefix of ["", "<!-- 日本 🧭 -->"]) {
      const id = `${fixture.id}-${newline === "\n" ? "lf" : "crlf"}-${prefix ? "unicode" : "plain"}`,
        root = path.join(output, "projects", id);
      fs.mkdirSync(root, { recursive: true });
      const parent = fixture.parent
          .replace("<template>", `<template>${prefix}`)
          .replaceAll("\n", newline),
        child = fixture.child.replaceAll("\n", newline);
      const config =
        fixture.id === "previous-10216"
          ? JSON.parse(fs.readFileSync(path.join(oldRoot, "tsconfig.json"), "utf8"))
          : JSON.parse(read("tsconfig.json.txt"));
      config.contentMappers = [{ package: "vize", extensions: [".vue"] }];
      const expected = fixture.code
        ? (() => {
            const lines = parent.split(/\r?\n/u);
            const index = lines.findIndex((line) =>
              fixture.id === "authored"
                ? line.includes("{{ status")
                : fixture.id === "slot-body"
                  ? line.includes("fruit.missing")
                  : line.includes(':label="'),
            );
            assert.ok(index >= 0);
            const line = lines[index]!;
            const column =
              fixture.id === "authored"
                ? line.indexOf("status", line.indexOf("{{")) + 1
                : line.indexOf(fixture.token!) + 1;
            return `src/${fixture.parentFile}(${index + 1},${column}): error TS${fixture.code}: ${fixture.message}\n`;
          })()
        : "";
      const sides = [];
      for (const side of ["baseline", "candidate"]) {
        const project = path.join(root, side);
        fs.mkdirSync(path.join(project, "src"), { recursive: true });
        fs.writeFileSync(path.join(project, "src", fixture.parentFile), parent);
        fs.writeFileSync(path.join(project, "src", fixture.childFile), child);
        save(path.join(project, "tsconfig.json"), config);
        const deps = path.join(project, "node_modules");
        if (side === "baseline") fs.symlinkSync(path.join(consumer, "node_modules"), deps, "dir");
        else {
          fs.mkdirSync(path.join(deps, "vize"), { recursive: true });
          fs.symlinkSync(packageRoot("vue"), path.join(deps, "vue"), "dir");
          save(path.join(deps, "vize/package.json"), {
            name: "vize",
            private: true,
            version: `0.0.0-source-${sourceSha}`,
            typescript: {
              contentMapper: {
                exec: [candidate, "content-mapper"],
                dynamicConfig: true,
                compilerOptions: ["checkJs", "noUnusedLocals"],
              },
            },
          });
        }
        const cli = side === "baseline" ? baseline : [candidate],
          args = [
            ...cli,
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
            "--quiet",
          ];
        const check = observe(
          [...args, "--corsa-path", native],
          project,
          path.join(project, "check"),
          env,
        );
        const captured = nativeCapture(args, native, project, path.join(project, "native"), env);
        const mapper = observe(
          [official, "--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
          project,
          path.join(project, "mapper"),
          env,
        );
        save(path.join(project, "startup-configuration.json"), {
          manifest: JSON.parse(fs.readFileSync(path.join(deps, "vize/package.json"), "utf8")),
          custody: "configuration evidence, not mapper RPC witness",
        });
        fs.rmSync(deps, { recursive: true });
        sides.push({ side, project, check, captured, mapper });
      }
      observations.push({ id, fixture, expected, sides });
    }
save(path.join(output, "inventory.json"), {
  projects: observations.length,
  ids: observations.map((row) => row.id),
  topLevelProcesses: observations.length * 6,
  completeCaptureBeforeAssertions: true,
});
assert.equal(observations.length, 40);
function stable(result: ProcessPacket) {
  assert.equal(result.error, null);
  assert.equal(result.signal, null);
  assert.equal(result.stderr, "");
}
for (const row of observations) {
  const first = row.sides[0]!;
  for (const side of row.sides) {
    stable(side.check);
    stable(side.mapper);
    stable(side.captured.result);
    assert.equal(side.check.status, row.fixture.code ? 1 : 0, row.id);
    assert.equal(side.captured.result.status, side.check.status);
    assert.equal(
      side.captured.result.stdout,
      side.check.stdout,
      "observation must not alter the complete CLI result",
    );
    assert.equal(
      side.check.stdout,
      first.check.stdout,
      `${row.id}: every authored native CLI diagnostic is unchanged`,
    );
    const packet = JSON.parse(side.check.stdout);
    assert.equal(packet.errorCount, row.fixture.code ? 1 : 0);
    assert.equal(packet.warningCount, 0);
    if (side.side === "candidate" || !row.fixture.guarded) {
      assert.equal(side.mapper.status, row.fixture.code ? 2 : 0, row.id);
      assert.equal(side.mapper.stdout, row.expected, `${row.id}: whole mapper stdout`);
    } else {
      assert.equal(side.mapper.status, 2);
      assert.ok(side.mapper.stdout.includes("TS2367"), "physical public negative control");
    }
    const children = side.captured.processes.filter((child) => child.argv.includes("--project"));
    assert.equal(children.length, 1, `${row.id}: exact native project child`);
    const child = children[0]!;
    assert.equal(child.signal, null);
    assert.equal(child.stderr, "");
    assert.equal(child.stdin.length, 0);
    const sourceRoot = child.prefix + ".program";
    const source = fs.readFileSync(
      path.join(sourceRoot, "src", row.fixture.parentFile + ".ts"),
      "utf8",
    );
    const lines = source.split("\n");
    const expected: { line: number; column: number; code: number; message: string }[] = [];
    lines.forEach((line, index) => {
      const imported = line.match(/^import\b.*?from\s*(['"])([^'"]+\.vue\.ts)\1/u);
      if (imported)
        expected.push({
          line: index + 1,
          column: line.indexOf(imported[1]!) + 1,
          code: 5097,
          message: importError,
        });
      if (
        side.side === "baseline" &&
        row.fixture.guarded &&
        line.includes("__VizeSlotsFactory") &&
        /\bstatus\s*===/u.test(line)
      )
        expected.push({
          line: index + 1,
          column: line.indexOf("status") + 1,
          code: 2367,
          message: impossible,
        });
      if (row.fixture.code === 2367 && line.includes("void (status === 'pending');"))
        expected.push({
          line: index + 1,
          column: line.indexOf("status") + 1,
          code: 2367,
          message: impossible,
        });
      if (row.fixture.code === 2339 && line.includes("void (fruit.missing);"))
        expected.push({
          line: index + 1,
          column: line.indexOf("missing") + 1,
          code: 2339,
          message: missing,
        });
      if (
        row.fixture.code === 2322 &&
        line.includes("const __vize_prop_check_") &&
        line.includes("_label:")
      )
        expected.push({
          line: index + 1,
          column: line.indexOf("__vize_prop_check_") + 1,
          code: 2322,
          message: row.fixture.message!,
        });
      if (row.fixture.code === 2322 && line.trimStart().startsWith('"label":'))
        expected.push({
          line: index + 1,
          column: line.indexOf('"label"') + 1,
          code: 2322,
          message: row.fixture.message!,
        });
    });
    assert.equal(
      expected.filter((d) => d.code === 5097).length,
      1,
      "preserve the complete existing generated import-policy diagnostic",
    );
    if (side.side === "baseline" && row.fixture.guarded)
      assert.equal(
        expected.filter((d) => d.code === 2367).length,
        2 + Number(row.fixture.code === 2367),
      );
    const raw = expected
      .map(
        (d) =>
          `src/${row.fixture.parentFile}.ts(${d.line},${d.column}): error TS${d.code}: ${d.message}\n`,
      )
      .join("");
    assert.equal(
      child.stdout,
      raw,
      `${row.id}/${side.side}: ALL raw native diagnostics, including import-policy and authored errors`,
    );
    assert.equal(
      child.code,
      1,
      "the complete raw vector retains the known import-policy finding; never call filtered CLI clean raw acceptance",
    );
  }
}
save(path.join(output, "result.json"), {
  sourceSha,
  projects: 40,
  completeMapperVectors: true,
  completeNativePreProjectionVectors: true,
  originalGeneratedGuardErrorsRemoved: 2,
  retainedGeneratedImportPolicyDiagnostic: 5097,
  authoredErrorsPreserved: [2367, 2339, 2322],
  previous10216Preserved: true,
  scope:
    "Exact source qualification against physical published providers, not publication acceptance",
});
