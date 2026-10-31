import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: { provider: { type: "string" }, output: { type: "string" }, source: { type: "string" } },
});
assert.ok(values.provider && values.output && values.source);
assert.match(values.source, /^[0-9a-f]{40}$/u);
const provider = resolve(values.provider);
const output = resolve(values.output);
const fixture = fileURLToPath(
  new URL(
    "../../../../tests/_fixtures/differential/typechecker/ambient-template-ref-10980/",
    import.meta.url,
  ),
);
const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const version = (name: string) =>
  JSON.parse(readFileSync(join(provider, "node_modules", name, "package.json"), "utf8")).version;
assert.equal(version("typescript"), "6.0.3");
assert.equal(version("vue-tsc"), "3.3.13");
const original = readFileSync(join(fixture, "Probe.vue.txt"), "utf8");
const witness = "useTemplateRef('target')";
assert.equal(original.split(witness).length, 2);
const repaired = original.replace(witness, "useTemplateRef<HTMLButtonElement>('target')");
const project = join(output, "project");
mkdirSync(project, { recursive: true });
symlinkSync(join(provider, "node_modules"), join(project, "node_modules"), "dir");
copyFileSync(join(fixture, "global.d.ts.txt"), join(project, "global.d.ts"));
const config = JSON.parse(readFileSync(join(fixture, "tsconfig.json.txt"), "utf8"));
// Vue's name-only auto-composable inference must not claim this unrelated declaration.
config.vueCompilerOptions = { composables: { useTemplateRef: [] } };
writeFileSync(join(project, "tsconfig.json"), JSON.stringify(config, null, 2) + "\n");
const vueTsc = join(provider, "node_modules/vue-tsc/bin/vue-tsc.js");
const argv = [vueTsc, "--noEmit", "--pretty", "false", "--project", "tsconfig.json"];
const observations = [];
for (const [phase, source, expected] of [
  [
    "original",
    original,
    "Probe.vue(3,32): error TS2339: Property 'focus' does not exist on type '{}'.\n",
  ],
  ["typed-repair", repaired, ""],
  [
    "restored",
    original,
    "Probe.vue(3,32): error TS2339: Property 'focus' does not exist on type '{}'.\n",
  ],
] as const) {
  const directory = join(output, phase);
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(project, "Probe.vue"), source);
  for (const name of ["Probe.vue", "global.d.ts", "tsconfig.json"])
    copyFileSync(join(project, name), join(directory, name));
  const result = spawnSync(process.execPath, argv, {
    cwd: project,
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  writeFileSync(join(directory, "stdout.raw"), result.stdout ?? Buffer.alloc(0));
  writeFileSync(join(directory, "stderr.raw"), result.stderr ?? Buffer.alloc(0));
  const observation = {
    phase,
    executable: process.execPath,
    argv,
    cwd: project,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  };
  writeFileSync(join(directory, "process.json"), JSON.stringify(observation, null, 2) + "\n");
  observations.push(observation);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, expected === "" ? 0 : 2);
  assert.deepEqual(result.stdout, Buffer.from(expected));
  assert.deepEqual(result.stderr, Buffer.alloc(0));
}
writeFileSync(
  join(output, "receipt.json"),
  JSON.stringify(
    {
      source: values.source,
      versions: {
        typescript: version("typescript"),
        vueTsc: version("vue-tsc"),
        vue: version("vue"),
      },
      fixture: ["Probe.vue.txt", "global.d.ts.txt", "tsconfig.json.txt"].map((name) => ({
        name,
        sha256: hash(join(fixture, name)),
      })),
      providerLockSha256: hash(join(provider, "package-lock.json")),
      vueTscSha256: hash(vueTsc),
      observations,
    },
    null,
    2,
  ) + "\n",
);
