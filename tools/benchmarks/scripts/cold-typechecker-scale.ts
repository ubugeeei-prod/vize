/** Fresh-process CLI / content-mapper / vue-tsc scaling with authored template refs. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { availableParallelism, cpus, totalmem } from "node:os";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { DEFAULT_SHAPE, generateWorkspace } from "./check-workspace.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";

const hash = (text: string | Buffer) => createHash("sha256").update(text).digest("hex");
const packages = 130;
export const coldScaleSupportScriptFiles = packages * 14 + 3;
const dependencyFiles = packages * 14 + 10;
const model = `export type DeepReadonly<T> = T extends object ? { readonly [K in keyof T]: DeepReadonly<T[K]> } : T;
export type Entity<K extends string> = { id: string & { readonly kind: K }; label: string; children: readonly Entity<K>[] };
export type Payload<T> = T extends Entity<infer K> ? { kind: K; selected: DeepReadonly<T> } : never;
`;

export function prepareColdScale(root: string, provider: string, files: number, invalid: boolean) {
  assert.ok(Number.isSafeInteger(files) && files >= 2000, "at least 2000 authored source files");
  assert.ok(!existsSync(root), "refuse to replace an existing fixture directory");
  const require = createRequire(join(provider, "package.json"));
  const app = generateWorkspace(
    root,
    {
      ...DEFAULT_SHAPE,
      packages,
      appComponents: files - dependencyFiles - 4,
    },
    {
      vue: dirname(require.resolve("vue/package.json")),
      typescript: dirname(require.resolve("typescript/package.json")),
    },
  );
  symlinkSync(join(provider, "node_modules", "vize"), join(root, "node_modules", "vize"), "dir");
  writeFileSync(join(app, "src", "model.ts"), model);
  const components = join(app, "src", "components");
  for (const name of readdirSync(components)) {
    const path = join(components, name);
    let code = readFileSync(path, "utf8");
    const child = code.includes("import Child from");
    const extra = `import { useTemplateRef, onMounted } from "vue";
import type { DeepReadonly, Payload, Entity } from "../model";
const primary = useTemplateRef("primary");
${child ? 'const child = useTemplateRef("child");' : ""}
const graph: DeepReadonly<{ title: string; labels: readonly string[] }> = { title: props.title, labels: labels.value };
type Item = Payload<Entity<"${name.replace(".vue", "")}">>;
const select = (item: Item): string => item.selected.label;
function reset() { open.value = false; }
defineExpose({ reset, labels, graph, select });
onMounted(() => { primary.value?.focus(); ${child ? "child.value?.reset();" : ""} });
`;
    code = code
      .replace("</script>", extra + "</script>")
      .replace('<Child title="child" />', '<Child ref="child" title="child" />')
      .replace("</section>", '<button ref="primary" @click="reset">Reset</button>\n  </section>');
    writeFileSync(path, code);
  }
  writeFileSync(
    join(app, "src", "Control.vue"),
    `<script setup lang="ts">
import { onMounted, useTemplateRef } from "vue";
import Child from "./components/Component1.vue";
const count: number = ${invalid ? '"intentional"' : "1"};
const primary = useTemplateRef("primary");
onMounted(() => { primary.value?.${invalid ? "definitelyMissingMethod" : "focus"}(); });
</script>
<template><button ref="primary">{{ count }}</button><Child :title="${invalid ? "42" : "String(count)"}" /></template>
`,
  );
  const configPath = join(app, "tsconfig.json");
  const config = JSON.parse(readFileSync(configPath, "utf8"));
  config.compilerOptions.incremental = false;
  config.contentMappers = [{ package: "vize", extensions: [".vue"] }];
  writeFileSync(configPath, JSON.stringify(config, null, 2) + "\n");
  const entries: { path: string; bytes: number; sha256: string }[] = [];
  function census(dir: string) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === "node_modules") continue;
      const path = join(dir, entry.name);
      if (entry.isDirectory()) census(path);
      else if (/\.(?:ts|vue)$/.test(path)) {
        const bytes = readFileSync(path);
        entries.push({
          path: path.slice(root.length + 1),
          bytes: bytes.length,
          sha256: hash(bytes),
        });
      }
    }
  }
  census(root);
  entries.sort((a, b) => a.path.localeCompare(b.path));
  assert.equal(entries.length, files, "complete authored dependency-graph census");
  return {
    app,
    files,
    sourceBytes: entries.reduce((n, x) => n + x.bytes, 0),
    sfcFiles: entries.filter((x) => x.path.endsWith(".vue")).length,
    corpusSha256: hash(JSON.stringify(entries)),
    entries,
  };
}

type Diagnostic = { file: string; line: number; column: number; code: number; message: string };

export function verifyColdDiagnostics(
  stdout: string,
  tool: string,
  files: number,
  invalid: boolean,
) {
  let actual: Diagnostic[];
  if (tool === "cli" || tool === "profile") {
    const packet = JSON.parse(stdout) as {
      fileCount: number;
      errorCount: number;
      warningCount: number;
      files: { file: string; diagnostics: string[] }[];
    };
    assert.equal(packet.fileCount, files, "every authored dependency must be checked");
    assert.equal(packet.warningCount, 0);
    actual = packet.files.flatMap((file) =>
      file.diagnostics.map((text) => {
        const match = /^error:(\d+):(\d+) \[TS(\d+)\] (.*)$/.exec(text);
        assert.ok(match, "unclassified diagnostic: " + text);
        return {
          file: file.file.replaceAll("\\", "/"),
          line: Number(match[1]),
          column: Number(match[2]),
          code: Number(match[3]),
          message: match[4].replace(/ \(source: .*\)$/, ""),
        };
      }),
    );
    assert.equal(packet.errorCount, actual.length);
  } else {
    actual = [...stdout.matchAll(/^(.+)\((\d+),(\d+)\): error TS(\d+): (.*)$/gm)].map((match) => ({
      file: match[1].replaceAll("\\", "/"),
      line: Number(match[2]),
      column: Number(match[3]),
      code: Number(match[4]),
      message: match[5],
    }));
  }
  const expected: Diagnostic[] = invalid
    ? [
        {
          file: "src/Control.vue",
          line: 4,
          column: 7,
          code: 2322,
          message: "Type 'string' is not assignable to type 'number'.",
        },
        {
          file: "src/Control.vue",
          line: 6,
          column: 34,
          code: 2339,
          message: "Property 'definitelyMissingMethod' does not exist on type 'HTMLButtonElement'.",
        },
        {
          file: "src/Control.vue",
          line: 8,
          column: 61,
          code: 2322,
          message: "Type 'number' is not assignable to type 'string'.",
        },
      ]
    : [];
  actual.sort((a, b) => a.line - b.line || a.column - b.column);
  assert.deepEqual(
    actual,
    expected,
    "complete authored diagnostics including exact UTF16 coordinates",
  );
  return actual;
}

export function captureColdScale(options: {
  provider: string;
  out: string;
  files: number;
  tool: string;
  invalid: boolean;
  binary?: string;
}) {
  const { provider, out, files, tool, invalid } = options;
  mkdirSync(join(out, "tmp"), { recursive: true });
  const corpus = prepareColdScale(join(out, "project"), provider, files, invalid);
  const { command, backend } = coldScaleCommand({ ...options, app: corpus.app });
  const { executable, args } = command;
  const executableSha256 = hash(readFileSync(executable));
  const backendSha256 = existsSync(backend) ? hash(readFileSync(backend)) : null;
  const started = performance.now();
  const result = spawnSync(
    "/usr/bin/time",
    [process.platform === "darwin" ? "-l" : "-v", executable, ...args],
    {
      cwd: corpus.app,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      timeout: 600_000,
      env: {
        ...process.env,
        ...command.env,
      },
    },
  );
  const wallMs = performance.now() - started;
  assert.equal(
    hash(readFileSync(executable)),
    executableSha256,
    "measured executable was replaced",
  );
  writeFileSync(join(out, "stdout.txt"), result.stdout ?? "");
  writeFileSync(join(out, "stderr.txt"), result.stderr ?? "");
  let correctness: { passed: boolean; diagnostics?: Diagnostic[]; error?: string };
  try {
    correctness = {
      passed: true,
      diagnostics: verifyColdDiagnostics(result.stdout ?? "", tool, files, invalid),
    };
  } catch (error) {
    correctness = { passed: false, error: String(error) };
  }
  const report = {
    schema: "vize.cold-scale",
    version: 2,
    tool,
    invalid,
    platform: process.platform,
    arch: process.arch,
    files,
    sfcFiles: corpus.sfcFiles,
    sourceBytes: corpus.sourceBytes,
    corpusSha256: corpus.corpusSha256,
    wallMs,
    targetColdWallMs: 500,
    meetsFixtureLatencyThreshold: corpus.sfcFiles >= 2500 && wallMs <= 500 && correctness.passed,
    // Fresh project paths do not prove the native cache namespace was empty.
    // An untimed backend observer must qualify that separately first.
    meetsColdTarget: false,
    hardware: {
      cpu: cpus()[0]?.model,
      availableCpus: availableParallelism(),
      memoryBytes: totalmem(),
    },
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    executable,
    executableSha256,
    backendSha256,
    args,
    cold: {
      newProcess: true,
      uniqueProject: true,
      sourceConfigIncremental: false,
      effectiveBackendIncremental: tool === "cli" || tool === "profile",
      cacheQualified: false,
      vizeCache: "XDG/TMP isolated; native cache namespace requires separate untimed audit",
      osPageCache: "not flushed",
    },
    peakRss: "recorded in stderr.txt; process-tree simultaneous RSS is not claimed",
    expectedErrors: invalid ? 3 : 0,
    correctness,
  };
  writeFileSync(join(out, "corpus.json"), JSON.stringify(corpus, null, 2) + "\n");
  writeFileSync(join(out, "measurement.json"), JSON.stringify(report, null, 2) + "\n");
  console.log(JSON.stringify(report));
  assert.ok(result.status !== null && [0, 1, 2].includes(result.status), result.stderr);
  assert.ok(correctness.passed, correctness.error);
  assert.ok(
    invalid ? result.status !== 0 : result.status === 0,
    "exit status must agree with complete diagnostics",
  );
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      files: { type: "string", default: "2500" },
      tool: { type: "string", default: "cli" },
      invalid: { type: "boolean", default: false },
      binary: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out);
  assert.ok(["cli", "mapper", "vue-tsc", "profile"].includes(values.tool!));
  captureColdScale({
    provider: resolve(values.provider),
    out: resolve(values.out),
    files: Number(values.files),
    tool: values.tool!,
    invalid: values.invalid!,
    binary: values.binary,
  });
}
