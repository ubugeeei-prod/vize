import { requiredEnv } from "../shared/files.ts";
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";

import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
const { setupComponentBracketReference } = (await import(
  pathToFileURL(
    resolveWorkflowInput("./tests/tooling/support/setup-component-bracket-reference.ts"),
  ).href
)) as typeof import("../../../../tests/tooling/support/setup-component-bracket-reference.ts");
const { templateMapReference, typescriptErasureReference, withoutTemplateSegments } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/typescript-erasure-reference.ts"))
    .href
)) as typeof import("../../../../tests/tooling/support/typescript-erasure-reference.ts");
const { assertSetupLiteralMovesIntoSetup, plainScriptProjectMetadataReference } = (await import(
  pathToFileURL(
    resolveWorkflowInput("./tests/tooling/support/plain-script-project-metadata-reference.ts"),
  ).href
)) as typeof import("../../../../tests/tooling/support/plain-script-project-metadata-reference.ts");
const { parse } = createRequire(process.env.VIZE10903_PROOF + "/host/package.json")(
  "@babel/parser",
);
const priorPackets = JSON.parse(
  readFileSync(
    requiredEnv("BEFORE").replace("performance.json", "full-module-packets.json"),
    "utf8",
  ),
);
const nextPackets = JSON.parse(
  readFileSync(process.env.VIZE10903_PROOF + "/full-module-packets.json", "utf8"),
);
const setupComponents: Record<string, string> = {
  original: "Child",
  "named-alias": "Child",
  "default-unknown": "Child",
  "constant-controls": "DefaultAlias",
};
assert.equal(priorPackets.length, nextPackets.length);
for (let i = 0; i < priorPackets.length; i++) {
  const prior = priorPackets[i],
    next = nextPackets[i];
  for (const key of ["case", "filename", "source", "sourceSha256", "options", "official"])
    assert.deepEqual(prior[key], next[key]);
  const assertProgram =
    prior.case === "constant-controls" && prior.filename === "src/App.vue"
      ? assertSetupLiteralMovesIntoSetup
      : undefined;
  // The render maps back to the template, so a map gains the segments
  // authored there and a macro-only component gains its map. The erasure
  // reference reads the script's segments; the template reference then
  // accepts only what the candidate's map gained inside `<template>`.
  const scriptMapped = withoutTemplateSegments(next.native, prior.source);
  const erased = (preserved: typeof prior.native) =>
    templateMapReference(
      typescriptErasureReference(preserved, scriptMapped, parse, assertProgram),
      next.native,
      prior.source,
    );
  if (!prior.options.inlineTemplate && prior.filename === "src/App.vue") {
    const component = setupComponents[prior.case];
    assert.equal(typeof component, "string", `missing setup component reference for ${prior.case}`);
    assert.deepEqual(
      plainScriptProjectMetadataReference(
        prior,
        erased(setupComponentBracketReference(prior.native, component)),
        process.cwd(),
      ),
      next.native,
    );
  } else if (prior.filename === "src/Child.vue" || prior.case === "constant-controls")
    assert.deepEqual(
      plainScriptProjectMetadataReference(prior, erased(prior.native), process.cwd()),
      next.native,
    );
}
const before = JSON.parse(readFileSync(requiredEnv("BEFORE"), "utf8"));
const after = JSON.parse(readFileSync(process.env.VIZE10903_PROOF + "/performance.json", "utf8"));
const beforeIdentity = JSON.parse(
  readFileSync(requiredEnv("BEFORE").replace("performance.json", "native-identity.json"), "utf8"),
);
const afterIdentity = JSON.parse(
  readFileSync(process.env.VIZE10903_PROOF + "/native-identity.json", "utf8"),
);
for (const key of ["inputSha256", "options", "warmup", "iterations"])
  assert.deepEqual(before[key], after[key]);
assert.notEqual(before.source, after.source);
assert.equal(afterIdentity.source, process.env.SOURCE_SHA);
assert.equal(afterIdentity.receipt.source, process.env.SOURCE_SHA);
assert.equal(afterIdentity.receipt.identity.token, afterIdentity.token);
assert.equal(afterIdentity.receipt.binary.sha256, afterIdentity.imageSha256);
assert.notEqual(beforeIdentity.imageSha256, afterIdentity.imageSha256);
writeFileSync(
  process.env.VIZE10903_PROOF + "/same-work-comparison.json",
  JSON.stringify(
    {
      before,
      after,
      beforeIdentity,
      afterIdentity,
      policy: "exact specified packet advance; performance observation only",
    },
    null,
    2,
  ) + "\n",
);
