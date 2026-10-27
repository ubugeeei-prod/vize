import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type ObjectValue = { [key: string]: Json };
interface FileInput {
  source: string;
  bytes: number;
  sha256: string;
}
interface Custody {
  files: Record<string, FileInput>;
}
interface Frozen {
  members: Record<string, string>;
  memberSha256: Record<string, string>;
  rows: Array<{
    statuses: number[];
    reports: { doctor: ObjectValue; lint: ObjectValue[]; inspectorAgent: ObjectValue };
    graph: Json;
  }>;
}
interface Native {
  getNativeBuildToken(): string | null;
  buildInspectorGraph(files: Array<{ path: string; source: string }>): string;
}
function sha(bytes: string | Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}
function object(value: Json | undefined): ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
  return value;
}
function array(value: Json | undefined): Json[] {
  assert(Array.isArray(value));
  return value;
}
const [cliArg, nativeArg, outputArg, role, sourceSha, expectedToken] = process.argv.slice(2);
assert(cliArg && nativeArg && outputArg && (role === "before" || role === "candidate"));
assert(sourceSha && /^[a-f0-9]{40}$/.test(sourceSha));
assert(expectedToken && /^[a-f0-9]{32}$/.test(expectedToken));
const cli = resolve(cliArg);
const output = resolve(outputArg);
mkdirSync(output, { recursive: true });
const inputDir = dirname(new URL(import.meta.url).pathname);
const originalBytes = readFileSync(join(inputDir, "original.json.txt"));
const beforeBytes = readFileSync(join(inputDir, "public-before.json.txt"));
assert.equal(
  sha(originalBytes),
  "db7bdd1947ad38a891d84f32859b35511613eada2edac865902dea84f9b9a40a",
);
assert.equal(sha(beforeBytes), "6894a8bf4507dc8b4463f4f3fa861204d98526d0d0e4ec7d4fdbba92785d140e");
const original = JSON.parse(originalBytes.toString()) as Custody;
const frozen = JSON.parse(beforeBytes.toString()) as Frozen;
assert.equal(Object.keys(original.files).length, 7);
assert.equal(Object.keys(frozen.members).length, 15);
for (const [name, bytes] of Object.entries(frozen.members)) {
  assert.equal(sha(bytes), frozen.memberSha256[name], name);
}
const publicBefore = frozen.rows[0];
assert(publicBefore);
assert.deepEqual(publicBefore.statuses, [1, 1, 1, 0]);
const loader = createRequire(import.meta.url);
const native = loader(resolve(nativeArg)) as Native;
const images = Object.keys(loader.cache).filter((path) => path.endsWith(".node"));
assert.equal(images.length, 1);
const image = images[0];
assert(image);
const imageBytes = readFileSync(image);
const imageIdentity = { path: image, bytes: imageBytes.length, sha256: sha(imageBytes) };
assert.equal(native.getNativeBuildToken(), expectedToken);
const sourceFiles = Object.entries(original.files).map(([path, file]) => {
  assert.equal(Buffer.byteLength(file.source), file.bytes, path);
  assert.equal(sha(file.source), file.sha256, path);
  return { path, source: file.source };
});
assert.deepEqual(JSON.parse(native.buildInspectorGraph(sourceFiles)) as Json, publicBefore.graph);
const cleanDoctor = structuredClone(publicBefore.reports.doctor);
cleanDoctor.findings = [];
const summary = object(cleanDoctor.summary);
summary.overallScore = 100;
summary.counts = { errors: 0, warnings: 0, notices: 0 };
summary.hasBlockingErrors = false;
object(object(summary.categories).correctness).score = 100;
object(object(summary.categories).correctness).penalty = 0;
object(object(summary.categories).correctness).findings = 0;
const cleanLint = publicBefore.reports.lint.map((file) => ({
  ...file,
  messages: [],
  errorCount: 0,
  warningCount: 0,
}));
const commands = [
  ["doctor", "--format", "json", "."],
  ["lint", "-f", "plain", "--no-config", "--cross-file", "src/**/*.vue"],
  ["lint", "--format", "json", "--no-config", "--cross-file", "src/**/*.vue"],
  ["inspector", "--format", "agent", "src/**/*.vue"],
];
function observe(
  id: string,
  files: Record<string, FileInput>,
): { statuses: number[]; doctor: ObjectValue; lint: ObjectValue[]; inspector: ObjectValue } {
  const cwd = join(output, id, "project");
  mkdirSync(cwd, { recursive: true });
  for (const [path, file] of Object.entries(files)) {
    const target = join(cwd, path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, file.source);
  }
  const rows = commands.map((args, index) => {
    const result = spawnSync(cli, args, {
      cwd,
      env: { ...process.env, NO_COLOR: "1" },
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    });
    writeFileSync(join(output, id, `command-${index}.stdout`), result.stdout ?? "");
    writeFileSync(join(output, id, `command-${index}.stderr`), result.stderr ?? "");
    writeFileSync(
      join(output, id, `command-${index}.process.json`),
      JSON.stringify(
        {
          cli,
          cliSha256: sha(readFileSync(cli)),
          args,
          cwd,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(result.error, undefined, id);
    assert.equal(result.signal, null, id);
    assert.notEqual(result.status, null, id);
    return { status: result.status, stdout: result.stdout };
  });
  const doctorRow = rows[0],
    lintRow = rows[2],
    inspectorRow = rows[3];
  assert(doctorRow && lintRow && inspectorRow);
  return {
    statuses: rows.map((row) => {
      assert(row.status !== null);
      return row.status;
    }),
    doctor: JSON.parse(doctorRow.stdout) as ObjectValue,
    lint: JSON.parse(lintRow.stdout) as ObjectValue[],
    inspector: JSON.parse(inspectorRow.stdout) as ObjectValue,
  };
}
const actualOriginal = observe("original", original.files);
assert.deepEqual(actualOriginal.statuses, role === "before" ? [1, 1, 1, 0] : [0, 0, 0, 0]);
assert.deepEqual(
  actualOriginal.doctor,
  role === "before" ? publicBefore.reports.doctor : cleanDoctor,
);
assert.deepEqual(actualOriginal.lint, role === "before" ? publicBefore.reports.lint : cleanLint);
assert.deepEqual(actualOriginal.inspector, publicBefore.reports.inspectorAgent);
const cases = [
  {
    id: "named-alias",
    script:
      "import { defineAsyncComponent as dac } from 'vue'; const LazyNoticeLink = dac(() => import('../components/NoticeLink.vue'));",
    errors: 0,
  },
  {
    id: "namespace",
    script:
      "import * as Vue from 'vue'; const LazyNoticeLink = Vue.defineAsyncComponent(() => import('../components/NoticeLink.vue'));",
    errors: 0,
  },
  {
    id: "loader-alias",
    script:
      "import { defineAsyncComponent as dac } from 'vue'; const loader = () => import('../components/NoticeLink.vue'); const load = loader; const LazyNoticeLink = dac(load);",
    errors: 0,
  },
  {
    id: "options",
    script:
      "import { defineAsyncComponent as dac } from 'vue'; const LazyNoticeLink = dac({loader: () => import('../components/NoticeLink.vue')});",
    errors: 0,
  },
  {
    id: "fake-factory",
    script:
      "const defineAsyncComponent = (loader: unknown) => ({}); const LazyNoticeLink = defineAsyncComponent(() => import('../components/NoticeLink.vue'));",
    errors: 1,
  },
  {
    id: "fake-import",
    script:
      "import { defineAsyncComponent } from 'fake-vue'; const LazyNoticeLink = defineAsyncComponent(() => import('../components/NoticeLink.vue'));",
    errors: 1,
  },
  { id: "opaque-binding", script: "const LazyNoticeLink = opaque();", errors: 1 },
  {
    id: "nonliteral",
    script:
      "import { defineAsyncComponent as dac } from 'vue'; const path = '../components/NoticeLink.vue'; const LazyNoticeLink = dac(() => import(path));",
    errors: 1,
  },
  {
    id: "uncalled-nested",
    script:
      "import { defineAsyncComponent as dac } from 'vue'; const LazyNoticeLink = dac(() => { const nested = () => import('../components/NoticeLink.vue'); return {}; });",
    errors: 1,
  },
];
const packets: Json[] = [];
for (const test of cases) {
  const page = `<script setup lang="ts">\nimport { defineAsyncComponent as actual } from 'vue';\nconst NoticeButton = actual(() => import('../components/NoticeButton.vue'));\n${test.script}\n</script>\n<template><NoticeButton/><LazyNoticeLink/></template>\n`;
  const files = {
    ...original.files,
    "src/pages/index.vue": { source: page, bytes: Buffer.byteLength(page), sha256: sha(page) },
  };
  const result = observe(test.id, files);
  // Before is historical observation, never a claim that opaque factories render.
  if (role === "candidate") {
    assert.deepEqual(result.statuses, test.errors === 0 ? [0, 0, 0, 0] : [1, 1, 1, 0], test.id);
    assert.deepEqual(
      result.doctor,
      test.errors === 0 ? cleanDoctor : publicBefore.reports.doctor,
      test.id,
    );
    assert.deepEqual(
      result.lint,
      test.errors === 0 ? cleanLint : publicBefore.reports.lint,
      test.id,
    );
  }
  assert.equal(array(result.inspector.semanticFiles).length, 5, test.id);
  packets.push({
    id: test.id,
    expectedCandidateErrors: test.errors,
    statuses: result.statuses,
    wholeDoctor: result.doctor,
    wholeLint: result.lint,
    wholeInspector: result.inspector,
  });
}

// A literal package spelling is not a resolved Vue provider. Both configured
// type and runtime aliases explicitly route this factory to an ordinary fake.
// This precision control must remain red until caller-owned authority refuses it.
const overridePage =
  '<script setup lang="ts">\nimport NoticeButton from "../components/NoticeButton.vue";\nimport { defineAsyncComponent } from "vue";\nconst LazyNoticeLink = defineAsyncComponent(() => import("../components/NoticeLink.vue"));\n</script>\n<template><NoticeButton/><LazyNoticeLink/></template>\n';
const overrideSources = {
  "src/pages/index.vue": overridePage,
  "src/fake-vue.ts": "export function defineAsyncComponent(_loader: unknown) { return {}; }\n",
  "tsconfig.json":
    JSON.stringify({ compilerOptions: { baseUrl: ".", paths: { vue: ["src/fake-vue.ts"] } } }) +
    "\n",
  "vite.config.mjs":
    'export default { resolve: { alias: { vue: new URL("./src/fake-vue.ts", import.meta.url).pathname } } };\n',
};
const overrideFiles = { ...original.files };
for (const [path, source] of Object.entries(overrideSources)) {
  overrideFiles[path] = { source, bytes: Buffer.byteLength(source), sha256: sha(source) };
}
const overrideResult = observe("custom-vue-provider", overrideFiles);
if (role === "candidate") {
  assert.deepEqual(
    overrideResult.statuses,
    [1, 1, 1, 0],
    "custom Vue authority must remain unknown",
  );
  assert.deepEqual(overrideResult.doctor, publicBefore.reports.doctor, "custom Vue provider");
  assert.deepEqual(overrideResult.lint, publicBefore.reports.lint, "custom Vue provider");
}
packets.push({
  id: "custom-vue-provider",
  expectedCandidateErrors: 1,
  statuses: overrideResult.statuses,
  wholeDoctor: overrideResult.doctor,
  wholeLint: overrideResult.lint,
  wholeInspector: overrideResult.inspector,
});

writeFileSync(
  join(output, "receipt.json"),
  JSON.stringify(
    {
      schema: "vize.asyncComponentAncestry.v1",
      role,
      sourceSha,
      claim: role === "candidate" ? "source correctness" : "immutable source before",
      publicBeforePacketSha256: sha(beforeBytes),
      originalCustodySha256: sha(originalBytes),
      native: {
        path: resolve(nativeArg),
        buildToken: native.getNativeBuildToken(),
        image: imageIdentity,
      },
      original: actualOriginal,
      controls: packets,
    },
    null,
    2,
  ) + "\n",
);
console.log(
  JSON.stringify({
    role,
    sourceSha,
    originalStatuses: actualOriginal.statuses,
    controls: packets.length,
  }),
);
