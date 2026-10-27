/** Public-release semantics; these runs do not qualify cold latency or broad accuracy. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

type Diagnostic = { file: string; line: number; column: number; code: number; message: string };
const sha = (p: string) => createHash("sha256").update(fs.readFileSync(p)).digest("hex");
const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    binary: { type: "string" },
    out: { type: "string" },
    release: { type: "string", default: "0.602.0" },
    archive: { type: "string" },
  },
});
assert.ok(values.provider && values.binary && values.out && values.archive);
const provider = path.resolve(values.provider),
  binary = path.resolve(values.binary);
const out = path.resolve(values.out);
assert.ok(!fs.existsSync(out), "retain previous evidence");
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(provider, "package.json"));
const nativeRequire = createRequire(require.resolve("typescript-native/package.json"));
const backend = path.join(
  path.dirname(
    nativeRequire.resolve(
      `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
    ),
  ),
  "lib",
  "tsc",
);
const binarySha256 = sha(binary),
  backendSha256 = sha(backend);
const version = spawnSync(binary, ["--version"], { encoding: "utf8", timeout: 30_000 });
assert.equal(version.status, 0);
assert.equal(version.stdout.trim(), "vize " + values.release);
const rows: unknown[] = [];
const write = (root: string, name: string, text: string) => {
  const p = path.join(root, name);
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, text);
};
const project = (name: string) => {
  const p = path.join(out, name);
  fs.mkdirSync(p);
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(p, "node_modules"), "dir");
  return p;
};
function run(root: string, stage: string, expected: Diagnostic[]) {
  const evidence = path.join(out, "packets", stage);
  fs.mkdirSync(evidence, { recursive: true });
  const runtime = path.join(process.env.RUNNER_TEMP ?? out, "public-semantic-" + randomUUID());
  fs.mkdirSync(path.join(runtime, "tmp"), { recursive: true });
  const args = [
    "check",
    "--no-config",
    "--format",
    "json",
    "--tsconfig",
    "tsconfig.json",
    "--corsa-path",
    backend,
  ];
  const result = spawnSync(binary, args, {
    cwd: root,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 64 * 1024 * 1024,
    env: {
      ...process.env,
      TMPDIR: path.join(runtime, "tmp"),
      XDG_CACHE_HOME: path.join(runtime, "cache"),
      NO_COLOR: "1",
    },
  });
  fs.writeFileSync(path.join(evidence, "stdout.json"), result.stdout ?? "");
  fs.writeFileSync(path.join(evidence, "stderr.txt"), result.stderr ?? "");
  const command = {
    binary,
    args,
    cwd: root,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    runtime,
    binarySha256,
    backendSha256,
  };
  fs.writeFileSync(path.join(evidence, "process.json"), JSON.stringify(command, null, 2));
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  const packet = JSON.parse(result.stdout);
  assert.equal(packet.warningCount, 0);
  const actual: Diagnostic[] = packet.files.flatMap(
    (row: { file: string; diagnostics: string[] }) =>
      row.diagnostics.map((text) => {
        const m = /^error:(\d+):(\d+) \[TS(\d+)\] (.*)$/.exec(text);
        assert.ok(m, "retain and reject an unclassified diagnostic");
        let file = row.file;
        if (path.isAbsolute(file)) file = path.relative(root, file);
        return {
          file: file.replaceAll("\\", "/"),
          line: +m[1],
          column: +m[2],
          code: +m[3],
          message: m[4].replace(/ \(source: .*\)$/, ""),
        };
      }),
  );
  assert.equal(packet.errorCount, actual.length, "duplicate multiplicity is mandatory");
  assert.equal(actual.length, expected.length);
  const order = (a: Diagnostic, b: Diagnostic) =>
    JSON.stringify(a).localeCompare(JSON.stringify(b));
  assert.deepEqual(actual.sort(order), [...expected].sort(order));
  assert.equal(result.status, expected.length ? 1 : 0);
  assert.equal(sha(binary), binarySha256);
  assert.equal(sha(backend), backendSha256);
  rows.push({ stage, ...command, actual, expected, passed: true, fileCount: packet.fileCount });
}
const fixture = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../tests/_fixtures/differential/typechecker/dependency-probe-history-10936",
);
const cases = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
const resolver = project("resolver");
for (const entry of cases.cases) {
  for (const input of entry.inputs)
    write(resolver, input.file, fs.readFileSync(path.join(fixture, input.source), "utf8"));
  run(
    resolver,
    "resolver-" + entry.id,
    entry.diagnostics.map(
      ({ severity: _severity, ...row }: Diagnostic & { severity: number }) => row,
    ),
  );
}
// Deliberate errors in both dependency bodies prove that they are checked,
// in addition to supplying the types used by the entry SFC.
const dependencyExpected: Diagnostic[] = [];
for (const [index, name] of ["src/model.ts", "second/part.ts"].entries()) {
  const p = path.join(resolver, name),
    source = fs.readFileSync(p, "utf8");
  const token = "dependencyCoverage" + index;
  const addition = `const ${token}: number = "invalid";\n`;
  const prefix = source.endsWith("\n") ? source : source + "\n";
  fs.writeFileSync(p, prefix + addition);
  dependencyExpected.push({
    file: name,
    line: prefix.split("\n").length,
    column: 7,
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
  });
}
run(resolver, "resolver-dependency-bodies-invalid", dependencyExpected);
for (const input of cases.cases[0].inputs)
  write(resolver, input.file, fs.readFileSync(path.join(fixture, input.source), "utf8"));
run(resolver, "resolver-dependency-bodies-repaired", []);
const ambient = project("ambient");
write(ambient, "tsconfig.json", '{"extends":"./.nuxt/tsconfig.json"}\n');
write(
  ambient,
  ".nuxt/tsconfig.json",
  JSON.stringify({
    compilerOptions: {
      strict: true,
      noEmit: true,
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      skipLibCheck: true,
    },
    include: ["../app/**/*.ts", "./nuxt.d.ts"],
  }),
);
write(ambient, ".nuxt/nuxt.d.ts", '/// <reference path="types/import-meta.d.ts" />\nexport {};\n');
write(
  ambient,
  ".nuxt/types/import-meta.d.ts",
  'import type { AmbientPayload } from "../../shared/model";\nexport {};\ndeclare global { interface ImportMeta { payload: AmbientPayload; } }\n',
);
write(ambient, "shared/model.ts", "export interface AmbientPayload { label: string }\n");
write(
  ambient,
  "app/auth.ts",
  "const label: string = import.meta.payload.label;\nexport { label };\n",
);
run(ambient, "ambient-valid", []);
write(ambient, "shared/model.ts", "export interface AmbientPayload { label: number }\n");
run(ambient, "ambient-import-type-mutated", [
  {
    file: "app/auth.ts",
    line: 1,
    column: 7,
    code: 2322,
    message: "Type 'number' is not assignable to type 'string'.",
  },
]);
write(ambient, "shared/model.ts", "export interface AmbientPayload { label: string }\n");
run(ambient, "ambient-repaired", []);
const receipt = {
  schema: "vize.public-resolver-acceptance",
  version: 1,
  release: values.release,
  binarySha256,
  backendSha256,
  archiveSha256: sha(path.resolve(values.archive)),
  providerLockSha256: sha(path.join(provider, "package-lock.json")),
  scope:
    "public standalone CLI; precedence/alias/body/hidden-ambient mutation and repair; untimed; no cold or broad accuracy qualification",
  rawMessages:
    "stdout packets retained; only CLI provenance suffix is excluded from message comparison",
  rows,
  passed: true,
};
fs.writeFileSync(path.join(out, "acceptance.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(
  JSON.stringify({ passed: true, release: values.release, stages: rows.length, binarySha256 }),
);
