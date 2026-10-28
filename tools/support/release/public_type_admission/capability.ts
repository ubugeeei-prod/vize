import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { installedRuntime } from "./installed-runtime.ts";
import { project, verifyOriginals } from "./fixtures.ts";
import { checkProject } from "./process.ts";
import { hash, parse, record, string, write, type Expected } from "./model.ts";

/** A known ineligible publication proves host mechanics; it cannot qualify #10950. */
const [workspace, version, outputArg, planFile] = process.argv.slice(2);
assert.ok(workspace && version && outputArg && planFile);
assert.equal(version, "0.603.0");
const output = path.resolve(outputArg);
fs.mkdirSync(output, { recursive: true });
const planBytes = fs.readFileSync(planFile);
const plan = record(parse(planBytes));
assert.equal(plan.schema, "vize.typeAdmission.sourceCapabilityEligibility.v1");
assert.equal(plan.version, version);
assert.equal(plan.scope, "source-capability-only");
assert.notEqual(plan.coreReleaseEligible, true);
const candidate = string(plan.sourceSha);
assert.match(candidate, /^[0-9a-f]{40}$/u);
assert.equal(record(plan.tag).name, "v" + version);
assert.equal(record(record(plan.tag).commit).sha, candidate);
const publication = record(plan.publishedRelease);
assert.equal(publication.tag_name, "v" + version);
assert.equal(publication.draft, false);
assert.ok(string(publication.published_at).length > 0);
write(path.join(output, "capability-plan.txt"), plan);
const runtime = await installedRuntime({ workspace, version, output, candidate, delivery: plan });
const capsule = path.dirname(fileURLToPath(import.meta.url));
const originals = verifyOriginals(capsule);
const valid =
  '<script setup lang="ts">\nconst count: number = 1;\n</script>\n<template>{{ count.toFixed() }}</template>\n';
const invalid = valid.replace("count.toFixed()", "count.missingCapability()");
const fixture = project(runtime, "host-capability", { "Probe.vue": valid }, []);
const expected: Expected[] = [{ file: "Probe.vue", line: 4, column: 20, code: 2339 }];
const cppLock = hash(fs.readFileSync(path.join(runtime.workspace, "package-lock.json")));
const hostLock = hash(
  fs.readFileSync(path.join(runtime.mapperHost.workspace, "package-lock.json")),
);
const image = hash(fs.readFileSync(runtime.native.path));
const results: unknown[] = [];
for (const [stage, source, errors] of [
  ["valid", valid, []],
  ["invalid", invalid, expected],
  ["repaired", valid, []],
] as const) {
  fs.writeFileSync(path.join(fixture.directory, "Probe.vue"), source);
  const capture = path.join(output, stage);
  write(path.join(capture, "authored-inputs.txt"), {
    files: { "Probe.vue": source },
    expected: errors,
  });
  results.push(...checkProject(runtime, fixture.directory, stage, [...errors], capture));
}
assert.equal(results.length, 6);
assert.equal(hash(fs.readFileSync(path.join(runtime.workspace, "package-lock.json"))), cppLock);
assert.equal(
  hash(fs.readFileSync(path.join(runtime.mapperHost.workspace, "package-lock.json"))),
  hostLock,
);
assert.equal(hash(fs.readFileSync(runtime.native.path)), image);
write(path.join(output, "host-capability.txt"), {
  schema: "vize.template-ref.host-capability.v1",
  scope: "known public 0.603 only; hosted route and physical provider mechanics",
  version,
  candidate,
  originals,
  cppSdk: "7.0.2",
  hostedSdk: runtime.mapperHost,
  actualNative: runtime.native,
  results,
  rawStreamsRetained: true,
  pullRequest10950Qualified: false,
  firstContainingReleaseQualified: false,
  originIssue10980Qualified: false,
  passed: true,
});
