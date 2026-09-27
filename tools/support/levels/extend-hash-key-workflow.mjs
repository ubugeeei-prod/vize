#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const originalCommand =
  "cargo test -p vize_davinci --test artifact_keys --test key_manifests -- --nocapture";
export const currentCommand =
  "cargo test -p vize_davinci --test artifact_keys --test key_manifests --test hash_domains --test sfc_summary --test global_summary -- --nocapture";
const stepName = "Print and compare the TS-43 golden keys";

// Extend the current workflow; never load or restore a frozen workflow body.
export function extendHashKeyWorkflow(source) {
  const before = parse(source);
  assert.equal(Object.hasOwn(before.on ?? {}, "pull_request"), false);
  const steps = before.jobs?.["key-stability"]?.steps;
  assert.equal(Array.isArray(steps), true);
  const indices = steps.flatMap((step, index) => (step.name === stepName ? [index] : []));
  assert.equal(indices.length, 1, "one exact key-stability step is required");
  const index = indices[0];
  const command = steps[index].run;
  assert.equal(
    command === originalCommand || command === currentCommand,
    true,
    "refuse an unrecognized current key-stability command",
  );
  let after = source;
  if (command === originalCommand) {
    const needle = "run: " + originalCommand;
    assert.equal(source.split(needle).length, 2, "one exact run scalar is required");
    after = source.replace(needle, "run: " + currentCommand);
  }
  const expected = structuredClone(before);
  expected.jobs["key-stability"].steps[index].run = currentCommand;
  assert.deepEqual(parse(after), expected, "only the selected run command may change");
  return after;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv.slice(2);
  assert.equal(mode.length, 1, "use --check or --apply");
  assert.equal(mode[0] === "--check" || mode[0] === "--apply", true);
  const root = fileURLToPath(new URL("../../../", import.meta.url));
  const workflow = path.join(root, ".github/workflows/davinci-incremental.yml");
  const before = fs.readFileSync(workflow, "utf8");
  const after = extendHashKeyWorkflow(before);
  if (mode[0] === "--check") assert.equal(before, after, "replay --apply on current source");
  else if (after !== before) fs.writeFileSync(workflow, after);
  console.log(JSON.stringify({ changed: before !== after, mode: mode[0] }));
}
