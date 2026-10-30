import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { assertPreservedCallbackPacket } from "../../tools/support/compat/content-mapper/callback-owner-audit.ts";
import { callbackNativeEventGate } from "../../tools/support/compat/content-mapper/callback-owner-packet-parity.ts";
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
  assertPreservedCallbackPacket(beforeMarker, {
    ...after,
    text: `${after.text}type Public = ${previousSlotsMarker};\n`,
  });
  for (const [original, candidate] of [
    [`${before.text}${currentSlotsExport}\n`, `${after.text}${currentSlotsExport}\n`],
    [`${before.text}${previousSlotsExport}\n`, `${after.text}${currentSlotsExport}\n`],
    [
      `${beforeMarker.text}type Existing = ${currentSlotsMarker};\n`,
      `${afterMarker.text}type Existing = ${currentSlotsMarker};\n`,
    ],
  ]) {
    assert.throws(
      () =>
        assertPreservedCallbackPacket({ ...before, text: original }, { ...after, text: candidate }),
      assert.AssertionError,
    );
  }
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

test("callback event scaffolding relocates complete UTF-8 packets and refuses unknown changes", () => {
  for (const newline of ["\n", "\r\n"]) {
    const prefix = `// 日本 🧭${newline}`;
    const root =
      "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }";
    const args =
      `  type __Comp_1_save_fallthrough_args = typeof Comp extends ${root}\n` +
      "    ? $IsAny<__F> extends true ? unknown[] : NonNullable<__F> extends infer __P ? (__P extends { onSave?: (...args: infer __A) => any } ? __A : unknown[]) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[]\n" +
      "    : unknown[];\n";
    const returns = `  type __Comp_1_save_return_native_fallthrough = typeof Comp extends ${root} ? $IsAny<__F> extends true ? any : NonNullable<__F> extends { onSave?: infer __H } ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any : any;\n`;
    const gate = callbackNativeEventGate("Comp");
    const nextArgs = args
      .replace("    ? $IsAny", `    ? ${gate}$IsAny`)
      .replace("    : unknown[];\n", "    : unknown[] : unknown[];\n");
    const nextReturn = returns
      .replace(`extends ${root} ? `, `extends ${root} ? ${gate}`)
      .replace(";\n", " : any;\n");
    const mapped = "const café = '雪🧭';\n";
    const beforeStart = Buffer.byteLength(`${prefix}${args}${returns}const `);
    const afterStart = Buffer.byteLength(`${prefix}${nextArgs}${nextReturn}const `);
    const length = Buffer.byteLength("café");
    const before = {
      text: `${prefix}${args}${returns}${mapped}${previousSlotsExport}\ntype Public = ${previousSlotsMarker};\n`,
      extension: ".tsx",
      mappings: [[beforeStart, length, 0, length, 0, 1048575]],
      semanticLinks: [
        {
          sourceStart: beforeStart,
          sourceLength: length,
          targetStart: beforeStart,
          targetLength: length,
          kind: "rename",
        },
      ],
      diagnosticDirectives: {
        directives: [[0, length, beforeStart, beforeStart + length, 0, 0]],
        unusedExpectDirectiveDiagnostics: [],
      },
      diagnostics: [],
      futureMetadata: { preserved: true },
    };
    const after = {
      ...before,
      text: `${prefix}${nextArgs}${nextReturn}${mapped}${currentSlotsExport}\ntype Public = ${currentSlotsMarker};\n`,
      mappings: [[afterStart, length, 0, length, 0, 1048575]],
      semanticLinks: [
        { ...before.semanticLinks[0], sourceStart: afterStart, targetStart: afterStart },
      ],
      diagnosticDirectives: {
        ...before.diagnosticDirectives,
        directives: [[0, length, afterStart, afterStart + length, 0, 0]],
      },
    };
    assertPreservedCallbackPacket(before, after);
    assert.throws(
      () =>
        assertPreservedCallbackPacket(
          {
            ...before,
            text: before.text.replace(
              `${previousSlotsExport}\ntype Public = ${previousSlotsMarker};\n`,
              "",
            ),
          },
          {
            ...after,
            text: after.text.replace(
              `${currentSlotsExport}\ntype Public = ${currentSlotsMarker};\n`,
              "",
            ),
          },
        ),
      assert.AssertionError,
    );
    for (const unobserved of [
      'import("vize:virtual-ts").IsAny',
      'import("vize:virtual-ts").$IsAny',
    ]) {
      const unprojected = {
        mappings: [],
        semanticLinks: [],
        diagnosticDirectives: { directives: [] },
      };
      assert.throws(
        () =>
          assertPreservedCallbackPacket(
            { ...before, ...unprojected, text: before.text.replaceAll("$IsAny", unobserved) },
            { ...after, ...unprojected, text: after.text.replaceAll("$IsAny", unobserved) },
          ),
        assert.AssertionError,
      );
    }
    for (const mutation of [
      { ...after, text: after.text.replace("__vizeEmitProps", "__otherMarker") },
      { ...after, text: `${after.text}type NewSlotConstructor = any;\n` },
      { ...after, mappings: [[afterStart + 1, length, 0, length, 0, 1048575]] },
      { ...after, mappings: [[afterStart, length, 1, length, 0, 1048575]] },
      { ...after, mappings: [[afterStart, length, 0, length, 0, 10]] },
      {
        ...after,
        diagnosticDirectives: {
          ...after.diagnosticDirectives,
          directives: [[0, length, afterStart, afterStart + length, 1, 0]],
        },
      },
      { ...after, semanticLinks: [{ ...after.semanticLinks[0], kind: "other" }] },
      { ...after, futureMetadata: { preserved: false } },
    ])
      assert.throws(() => assertPreservedCallbackPacket(before, mutation), assert.AssertionError);
    const insertion = Buffer.byteLength(`${prefix}${args.split("    ? ")[0]}    ? `);
    for (const [start, spanLength] of [
      [insertion - 1, 2],
      [insertion, 0],
    ]) {
      assert.throws(
        () =>
          assertPreservedCallbackPacket(
            { ...before, mappings: [[start, spanLength, 0, spanLength, 0, 1048575]] },
            { ...after, mappings: [[start, spanLength, 0, spanLength, 0, 1048575]] },
          ),
        assert.AssertionError,
      );
    }
  }
});

test("callback certificate is covered by exact-head materialization and producer templates", () => {
  const file = "tools/support/compat/content-mapper/callback-owner-packet-parity.ts";
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/component-input-diagnostic-ownership.yml"),
    "utf8",
  );
  assert.ok(workflow.includes('"tools/support/compat/content-mapper/callback-owner*.ts"'));
  assert.ok(path.basename(file).startsWith("callback-owner"));
  assert.ok(workflow.includes('ref: "${{ github.event.pull_request.head.sha }}"'));
  assert.ok(workflow.includes("node tools/support/compat/content-mapper/callback-owner-audit.ts"));
  const audit = fs.readFileSync(
    path.join(root, "tools/support/compat/content-mapper/callback-owner-audit.ts"),
    "utf8",
  );
  assert.ok(audit.includes('from "./callback-owner-packet-parity.ts"'));
  assert.ok(audit.includes("packetCertificateFiles"));
  assert.ok(audit.includes('"./callback-owner-packet-parity.ts"'));
  const producer = fs.readFileSync(
    path.join(
      root,
      "crates/vize_canon/src/virtual_ts/scope/component_events/native_fallthrough.rs",
    ),
    "utf8",
  );
  const references = fs.readFileSync(
    path.join(root, "crates/vize_canon/src/virtual_ts/helpers/shared_types.rs"),
    "utf8",
  );
  assert.ok(references.includes('IS_ANY => ("$IsAny", "IsAny")'));
  const emitted = producer
    .split("\n")
    .filter((line) => line.trim().startsWith('"') && line.includes("__vizeEmitProps"));
  assert.equal(emitted.length, 2);
  for (const line of emitted) {
    const literal = JSON.parse(line.trim().replace(/,$/u, "")) as string;
    const rendered = literal
      .replaceAll("{component_ref}", "Comp")
      .replaceAll("{{", "{")
      .replaceAll("}}", "}");
    assert.ok(rendered.includes(callbackNativeEventGate("Comp")));
  }
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
