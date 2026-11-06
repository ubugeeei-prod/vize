import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  plainScriptProjectMetadataReference,
  type PlainScriptPacket,
} from "./support/plain-script-project-metadata-reference.ts";
import { setupComponentBracketReference } from "./support/setup-component-bracket-reference.ts";
import {
  moduleShapeFactsReference,
  typescriptErasureReference,
  type ParseModule,
} from "./support/typescript-erasure-reference.ts";

const fixture = new URL(
  "../_fixtures/differential/compiler/plain-script-project-metadata-11885/",
  import.meta.url,
);
const read = (file: string) => readFileSync(new URL(file, fixture), "utf8");
const manifest = JSON.parse(read("manifest.json")) as {
  workspaceRoot: string;
  files: Array<{ file: string; sha256: string }>;
};
const before = JSON.parse(read("before-full-module-packets.json.txt")) as PlainScriptPacket[];
const candidate = JSON.parse(read("candidate-full-module-packets.json.txt")) as PlainScriptPacket[];
const fromRoot = createRequire(import.meta.url);
const fromBabel = createRequire(
  process.env.VIZE10903_PROOF
    ? path.join(process.env.VIZE10903_PROOF, "host/package.json")
    : fromRoot.resolve("@babel/core"),
);
const { parse } = fromBabel("@babel/parser") as { parse: ParseModule };
const setupComponents: Record<string, string> = {
  original: "Child",
  "named-alias": "Child",
  "default-unknown": "Child",
  "constant-controls": "DefaultAlias",
};

function comparePackets(priorPackets: PlainScriptPacket[], nextPackets: PlainScriptPacket[]) {
  assert.equal(priorPackets.length, nextPackets.length);
  const metadataRows = [];
  const unqualifiedRows = [];
  for (const [index, prior] of priorPackets.entries()) {
    const next = nextPackets[index];
    for (const key of [
      "case",
      "filename",
      "source",
      "sourceSha256",
      "options",
      "official",
    ] as const)
      assert.deepEqual(prior[key], next[key]);
    let preserved;
    if (!prior.options.inlineTemplate && prior.filename === "src/App.vue") {
      const component = setupComponents[prior.case];
      assert.equal(typeof component, "string");
      preserved = setupComponentBracketReference(prior.native, component);
    } else if (prior.filename === "src/Child.vue" || prior.case === "constant-controls") {
      preserved = prior.native;
    } else {
      unqualifiedRows.push([prior.case, prior.filename, prior.options]);
      continue;
    }
    const shape = next.native.moduleShape;
    // Historical packets without `imports` derive both HMR sides from candidate
    // code. This preserves the replay but does not validate candidate-native HMR facts.
    const nextNative =
      shape &&
      typeof shape === "object" &&
      !Array.isArray(shape) &&
      !Object.hasOwn(shape, "imports")
        ? moduleShapeFactsReference(next.native, parse)
        : next.native;
    const erased = typescriptErasureReference(preserved, nextNative, parse);
    const expected = plainScriptProjectMetadataReference(prior, erased, manifest.workspaceRoot);
    assert.deepEqual(expected, nextNative);
    if (prior.case === "constant-controls" && prior.filename === "src/App.vue")
      metadataRows.push(prior.options);
  }
  return { metadataRows, unqualifiedRows };
}

test("the replay retains the six original artifact entries byte for byte", () => {
  assert.deepEqual(
    manifest.files.map(({ file }) => file),
    ["before", "candidate"].flatMap((label) =>
      ["full-module-packets.json", "native-identity.json", "performance.json"].map(
        (name) => `${label}-${name}.txt`,
      ),
    ),
  );
  for (const { file, sha256 } of manifest.files)
    assert.equal(createHash("sha256").update(read(file)).digest("hex"), sha256);
  assert.equal(
    manifest.files[0].sha256,
    "e7c0f1c8b456e8339c3c386e960f8a72578789b193c97a3569b3f80f80a23cf0",
  );
  assert.equal(
    manifest.files[3].sha256,
    "9154528bebe16ded24a67288287afb5738aa8d4a8e68473096e1533385d86750",
  );
});

test("all 32 identities replay while only the four specified packets gain project metadata", () => {
  assert.equal(before.length, 32);
  assert.deepEqual(comparePackets(before, candidate), {
    metadataRows: [false, true].flatMap((inlineTemplate) =>
      [false, true].map((sourceMap) => ({
        filename: "src/App.vue",
        inlineTemplate,
        sourceMap,
        isProd: inlineTemplate,
      })),
    ),
    // These six rows were outside the original comparator. Their code and
    // module-shape differences receive no acceptance from this metadata fix.
    unqualifiedRows: ["original", "named-alias", "default-unknown"].flatMap((name) =>
      [false, true].map((sourceMap) => [
        name,
        "src/App.vue",
        { filename: "src/App.vue", inlineTemplate: true, sourceMap, isProd: true },
      ]),
    ),
  });
});

test("the old whole-packet comparison rejects the recorded project metadata addition", () => {
  for (const [index, prior] of before.entries()) {
    if (prior.case !== "constant-controls" || prior.filename !== "src/App.vue") continue;
    const next = candidate[index];
    const preserved = prior.options.inlineTemplate
      ? prior.native
      : setupComponentBracketReference(prior.native, "DefaultAlias");
    assert.throws(
      () =>
        assert.deepEqual(typescriptErasureReference(preserved, next.native, parse), next.native),
      { code: "ERR_ASSERTION" },
    );
  }
});

test("metadata never conceals packet fields or arbitrary project metadata drift", () => {
  for (const [index, prior] of before.entries()) {
    if (prior.case !== "constant-controls" || prior.filename !== "src/App.vue") continue;
    for (const patch of [
      { dependencies: [] },
      { dependencies: [`${manifest.workspaceRoot}/tsconfig.json`] },
      {
        dependencies: [
          `${manifest.workspaceRoot}/extra.json`,
          `${manifest.workspaceRoot}/tsconfig.json`,
        ],
      },
      { typeProject: { root: `${manifest.workspaceRoot}/other.json`, revision: "1" } },
      { typeProject: { root: `${manifest.workspaceRoot}/tsconfig.json`, revision: "2" } },
      {
        typeProject: {
          root: `${manifest.workspaceRoot}/tsconfig.json`,
          revision: "1",
          extra: true,
        },
      },
      { errors: [{ message: "unexpected diagnostic" }] },
      { warnings: [{ message: "unexpected warning" }] },
      { code: candidate[index].native.code + '\nthrow new Error("changed module");\n' },
      { moduleShape: null },
      { templateHash: "changed" },
      { map: "{}" },
      { extraMetadata: true },
    ]) {
      const changed = structuredClone(candidate);
      Object.assign(changed[index].native, patch);
      assert.throws(() => comparePackets(before, changed), { code: "ERR_ASSERTION" });
    }
  }
  const childIndex = candidate.findIndex((row) => row.filename === "src/Child.vue");
  const changed = structuredClone(candidate);
  changed[childIndex].native.typeProject = { root: "unrelated.json", revision: "1" };
  assert.throws(() => comparePackets(before, changed), { code: "ERR_ASSERTION" });
});

test("the metadata reference requires the preserved source, options and absent fields", () => {
  for (const row of before)
    if (row.case !== "constant-controls" || row.filename !== "src/App.vue")
      assert.deepEqual(
        plainScriptProjectMetadataReference(row, row.native, manifest.workspaceRoot),
        row.native,
      );
  const row = before.find(
    (row) => row.case === "constant-controls" && row.filename === "src/App.vue",
  )!;
  for (const changed of [
    { ...row, source: row.source + "\n" },
    { ...row, sourceSha256: "changed" },
    { ...row, options: { ...row.options, isProd: !row.options.isProd } },
    { ...row, options: { ...row.options, filename: "src/Other.vue" } },
    { ...row, options: { ...row.options, extraOption: true } },
  ])
    assert.throws(
      () => plainScriptProjectMetadataReference(changed, row.native, manifest.workspaceRoot),
      { code: "ERR_ASSERTION" },
    );
  for (const field of ["dependencies", "typeProject"])
    assert.throws(
      () =>
        plainScriptProjectMetadataReference(
          row,
          { ...row.native, [field]: undefined },
          manifest.workspaceRoot,
        ),
      { code: "ERR_ASSERTION" },
    );
  assert.throws(() => plainScriptProjectMetadataReference(row, row.native, "relative/workspace"), {
    code: "ERR_ASSERTION",
  });
  assert.equal(
    createHash("sha256").update(read("before-full-module-packets.json.txt")).digest("hex"),
    manifest.files[0].sha256,
  );
});
