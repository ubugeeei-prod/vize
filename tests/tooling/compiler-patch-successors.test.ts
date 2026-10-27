import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  assertCompilerPatchSuccessor,
  compilerPatchSuccessor,
  verifyCompilerPatchPair,
  type CompilerPatchCase,
} from "../_helpers/compiler-patch-successors.ts";
import type { CompilerOutput } from "../_helpers/realworld-build-errors.ts";

const cases: CompilerPatchCase[] = [
  "create-vue-clean",
  "create-vue-broken",
  "vue-router-clean",
  "vue-router-broken",
];

test("published compiler successors retain all original raw packets and bound each complete delta", () => {
  const originalPins = {
    "create-vue-clean": [
      "fccdc563078a16e31591a5414510d1b894353e52b1544478cfaad5d9702ac286",
      "1216a548943fe8a09ab349a913c627d4115f93935b75ec89922415511475eff7",
    ],
    "create-vue-broken": [
      "bf6a1940c6dbfd8faea47d747ab4163212b5570d1196aee72b1ee8c839bf7629",
      "417b26aaa9b8a10ad1939061a2e61913ec58dd37f575dd2a432e58e624d7d9df",
    ],
    "vue-router-clean": [
      "84e9feaa14017c1c0e27a6b8955bd4a591b3b61e8708f3ead335aaa0c7c96013",
      "2e99ba339b7fb47f5788a990245b0292ede267d6bdfcb6c80cbe4c76da9cdebc",
    ],
    "vue-router-broken": [
      "0914134eb2d13c322402dcf94b608ac904f9ff2ed021c69cd414d5eed84f5654",
      "eed24c03d97029fca5d3d13f81ac416099d3876e7f36572d113acc0768872679",
    ],
  };
  for (const id of cases) {
    const packet = compilerPatchSuccessor(id);
    assertCompilerPatchSuccessor(id, packet.currentRaw, {
      packetSha256: originalPins[id][0],
      codeSha256: originalPins[id][1],
      previousPacket: packet.previous,
    });
    assert.deepEqual(packet.current.errors, packet.previous.errors);
    assert.deepEqual(packet.review.changedFacets, id === "create-vue-clean" ? [] : ["code"]);
  }
});

test("full current output and original authority reject unrelated and missing packet facets", () => {
  for (const id of cases) {
    const packet = compilerPatchSuccessor(id);
    const mutations: ((value: CompilerOutput) => void)[] = [
      (value) => {
        value.code += "\n// unrelated";
      },
      (value) => {
        value.errors = ["unexpected error"];
      },
      (value) => {
        value.warnings = ["unexpected warning"];
      },
      (value) => {
        value.css = "foreign stylesheet";
      },
      (value) => {
        value.filename = "Other.vue";
      },
      (value) => {
        value.script_lang = "js";
      },
      (value) => {
        value.macro_artifacts = ["foreign macro"];
      },
    ];
    for (const mutate of mutations) {
      const changed = structuredClone(packet.current);
      mutate(changed);
      assert.throws(() => verifyCompilerPatchPair(id, packet.previous, changed));
      assert.throws(() =>
        assertCompilerPatchSuccessor(id, JSON.stringify(changed, null, 2), {
          previousPacket: packet.previous,
        }),
      );
    }
    for (const field of Object.keys(packet.current) as (keyof CompilerOutput)[]) {
      const missing: Partial<CompilerOutput> = structuredClone(packet.current);
      delete missing[field];
      assert.throws(() => verifyCompilerPatchPair(id, packet.previous, missing as CompilerOutput));
      assert.throws(() =>
        assertCompilerPatchSuccessor(id, JSON.stringify(missing, null, 2), {
          previousPacket: packet.previous,
        }),
      );
    }
    const extra = { ...packet.current, unknownFacet: true };
    assert.throws(() => verifyCompilerPatchPair(id, packet.previous, extra));
    assert.throws(() =>
      assertCompilerPatchSuccessor(id, packet.currentRaw, { packetSha256: "0".repeat(64) }),
    );
    assert.throws(() =>
      assertCompilerPatchSuccessor(id, packet.currentRaw, { codeSha256: "0".repeat(64) }),
    );
    assert.throws(() =>
      assertCompilerPatchSuccessor(id, packet.currentRaw + "\n", {
        previousPacket: packet.previous,
      }),
    );
  }
});

test("before and after provider identities are public packages and source inputs stay pinned", () => {
  const capture = JSON.parse(
    fs.readFileSync(
      new URL("../_fixtures/compiler-patch-successors-10923/capture.json", import.meta.url),
      "utf8",
    ),
  ) as {
    producers: {
      side: string;
      native: { name: string; version: string; loadedImages: { bytes: number; sha256: string }[] };
      cli: { name: string; version: string; nativeVersion: string };
    }[];
    upstream: { revision: string }[];
    files: { path: string; sha256: string; bytes: number }[];
  };
  const directory = new URL("../_fixtures/compiler-patch-successors-10923/", import.meta.url);
  assert.equal(capture.files.length, 16);
  assert.deepEqual(
    capture.files.map((file) => file.path).sort(),
    fs
      .readdirSync(directory)
      .filter((name) => name !== "capture.json" && name !== "README.md")
      .sort(),
  );
  for (const file of capture.files) {
    const bytes = fs.readFileSync(new URL(file.path, directory));
    assert.equal(bytes.length, file.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256, file.path);
  }
  assert.deepEqual(
    capture.producers.map((row) => [
      row.side,
      row.native.name,
      row.native.version,
      row.cli.name,
      row.cli.version,
      row.cli.nativeVersion,
    ]),
    [
      ["previous", "@vizejs/native", "0.443.0", "vize", "0.443.0", "0.443.0"],
      ["current", "@vizejs/native", "0.600.0", "vize", "0.600.0", "0.600.0"],
    ],
  );
  for (const provider of capture.producers) {
    assert.equal(provider.native.loadedImages.length, 1);
    for (const image of provider.native.loadedImages) {
      assert.ok(image.bytes > 0);
      assert.match(image.sha256, /^[a-f0-9]{64}$/);
    }
  }
  assert.deepEqual(
    capture.upstream.map((row) => row.revision),
    ["3ee62adffdcdfa4a37b2ed4e9c30636655d5fcd1", "feed382f2fbfe38b3892ea780f5aea3d5459986a"],
  );
});
