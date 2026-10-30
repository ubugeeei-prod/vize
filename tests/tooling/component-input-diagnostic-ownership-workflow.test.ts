import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { assertPreservedCallbackPacket } from "../../tools/support/compat/content-mapper/callback-owner-audit.ts";
import {
  currentSlotsExport,
  currentSlotsMarker,
  previousSlotsExport,
  previousSlotsMarker,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const historicalBefore = "78719553cf550bcffbf1d0fb891fda642f42de3a";

test("callback packets certify only the unmapped public Slots re-export", () => {
  const before = {
    extension: ".tsx",
    text: `const item = 1;\n${previousSlotsExport}\n`,
    mappings: [[6, 4, 0, 4, 0, 1048575]],
    diagnostics: [],
    semanticLinks: [{ sourceStart: 6, sourceLength: 4, targetStart: 6, targetLength: 4 }],
  };
  const after = { ...before, text: before.text.replace(previousSlotsExport, currentSlotsExport) };
  assertPreservedCallbackPacket(before, before);
  assertPreservedCallbackPacket(before, after);
  const beforeMarker = { ...before, text: `${before.text}type Public = ${previousSlotsMarker};\n` };
  const afterMarker = {
    ...after,
    text: `${after.text}type Public = ${currentSlotsMarker};\n`,
  };
  assertPreservedCallbackPacket(beforeMarker, afterMarker);
  for (const marker of [
    "Partial<any>",
    "Partial<$Wrong>",
    `${currentSlotsMarker} & ${currentSlotsMarker}`,
  ])
    assert.throws(
      () =>
        assertPreservedCallbackPacket(beforeMarker, {
          ...afterMarker,
          text: afterMarker.text.replace(currentSlotsMarker, marker),
        }),
      assert.AssertionError,
    );
  for (const mutated of [
    { ...after, text: after.text.replace("const item = 1", "const item = 2") },
    { ...after, text: after.text.replace(currentSlotsExport, "export type Slots = any;") },
    { ...after, text: `${after.text}${currentSlotsExport}\n` },
    { ...after, mappings: [[7, 4, 0, 4, 0, 1048575]] },
    { ...after, diagnostics: [{ start: 0, length: 4, code: 2304 }] },
    {
      ...after,
      semanticLinks: [{ sourceStart: 7, sourceLength: 4, targetStart: 6, targetLength: 4 }],
    },
  ])
    assert.throws(() => assertPreservedCallbackPacket(before, mutated), assert.AssertionError);
  const boundary = Buffer.byteLength(before.text.split(previousSlotsExport)[0]);
  for (const mutation of [
    { mappings: [[boundary - 1, 2, 0, 2, 0, 1048575]] },
    { mappings: [[boundary + 1, 0, 0, 0, 0, 1048575]] },
    {
      semanticLinks: [
        { sourceStart: boundary - 1, sourceLength: 2, targetStart: 6, targetLength: 4 },
      ],
    },
    { diagnosticDirectives: { directives: [[0, 4, boundary - 1, boundary + 1, 0, 0]] } },
  ])
    assert.throws(
      () => assertPreservedCallbackPacket({ ...before, ...mutation }, { ...after, ...mutation }),
      assert.AssertionError,
    );
});

function assertAuthenticatedBaseComparison(source: string): void {
  const start = source.indexOf("  published-sdk-packets:");
  const end = source.indexOf("  unchanged-projection-packets:");
  assert.ok(start >= 0 && end > start);
  const job = source.slice(start, end);
  assert.match(job, /BASE_SHA: \$\{\{ github\.event\.pull_request\.base\.sha \}\}/u);
  assert.match(job, /SOURCE_SHA: \$\{\{ github\.event\.pull_request\.head\.sha \}\}/u);
  assert.match(job, new RegExp(`HISTORICAL_CALLBACK_BEFORE: ${historicalBefore}`, "u"));
  assert.match(job, /\[\[ "\$BASE_SHA" =~ \^\[0-9a-f\]\{40\}\$ \]\]/u);
  assert.match(
    job,
    /\[\[ "\$\(git -C "\$RUNNER_TEMP\/callback-cfa-base" rev-parse HEAD\)" == "\$BASE_SHA" \]\]/u,
  );
  assert.match(job, /--comparison-source-sha "\$BASE_SHA"/u);
  assert.match(job, /--historical-comparison-source-sha "\$HISTORICAL_CALLBACK_BEFORE"/u);
  assert.match(job, /--comparison-role pull-request-base/u);
  assert.doesNotMatch(job, /--comparison-source-sha 78719553cf550bcffbf1d0fb891fda642f42de3a/u);
}

function assertConstructorPacketCustody(source: string): void {
  const start = source.indexOf("  unchanged-projection-packets:");
  assert.ok(start >= 0);
  const job = source
    .slice(start)
    .split(/\r?\n/gu)
    .filter((line) => !line.trimStart().startsWith("#"))
    .join("\n")
    .replace(/\\\r?\n\s*/gu, " ");
  const comparisons = [...job.matchAll(/\bcmp\s+([^;\n]+)/gu)].map((match) => match[1].trim());
  assert.deepEqual(comparisons, ['"$observer" "$baseline/$observer"']);
  assert.match(job, /constructor-projection-parity\.ts/u);
  assert.match(job, /base-legacy\.json/u);
  assert.match(job, /head-legacy\.json/u);
}

test("callback packet custody binds the comparison CLI to the exact PR base", () => {
  const source = fs.readFileSync(
    path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml"),
    "utf8",
  );
  assertAuthenticatedBaseComparison(source);

  for (const tampered of [
    source.replace(
      "BASE_SHA: ${{ github.event.pull_request.base.sha }}",
      `BASE_SHA: ${"0".repeat(40)}`,
    ),
    source.replace(
      '[[ "$(git -C "$RUNNER_TEMP/callback-cfa-base" rev-parse HEAD)" == "$BASE_SHA" ]]',
      "true",
    ),
    source.replace(
      '--comparison-source-sha "$BASE_SHA"',
      `--comparison-source-sha ${historicalBefore}`,
    ),
  ]) {
    assert.throws(() => assertAuthenticatedBaseComparison(tampered), assert.AssertionError);
  }
});

test("constructor packet custody delegates certified changes to complete parity", () => {
  const source = fs.readFileSync(
    path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml"),
    "utf8",
  );
  assertConstructorPacketCustody(source);
  assertConstructorPacketCustody(`${source}\n  # cmp only describes the observer\n`);
  assert.throws(
    () =>
      assertConstructorPacketCustody(
        `${source}\n  cmp "$RUNNER_TEMP/base.json" \\\n    "$RUNNER_TEMP/head-legacy.json"\n`,
      ),
    assert.AssertionError,
  );
  assert.throws(
    () =>
      assertConstructorPacketCustody(
        `${source}\n  if ! cmp "$RUNNER_TEMP/base.json" "$RUNNER_TEMP/head-legacy.json"; then exit 1; fi\n`,
      ),
    assert.AssertionError,
  );
});
