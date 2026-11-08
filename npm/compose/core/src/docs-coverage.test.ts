import assert from "node:assert/strict";
import { test } from "node:test";

import {
  auditDocumentation,
  auditSourceDirectory,
  auditSourceGraph,
} from "./test-support/api-documentation.ts";

const sourceDirectory = new URL(".", import.meta.url);

void test("every exported declaration and optional option carries complete documentation", () => {
  const audit = auditSourceDirectory(sourceDirectory);

  // Guard the parser itself: if source discovery rots, the suite must fail
  // loudly instead of silently checking nothing.
  assert.ok(
    audit.declarations >= 20,
    `expected >= 20 exported declarations, found ${audit.declarations}`,
  );
  assert.ok(
    audit.optionalMembers >= 15,
    `expected >= 15 optional options, found ${audit.optionalMembers}`,
  );
  assert.deepEqual(audit.problems, []);
});

void test("formatting cannot hide exported declarations or optional options", () => {
  const audit = auditDocumentation(
    "fixture.ts",
    [
      "/** Documented despite the multiline declaration. */",
      "export",
      "interface MultilineOptions {",
      "  hidden?: boolean;",
      "}",
      "",
      "/** Type-alias options are part of the same contract. */",
      "export type AliasOptions = {",
      "  /** @default false */",
      "  documented?: boolean;",
      "  missing?: string;",
      "};",
      "",
      "  export async function undocumented(): Promise<void> {}",
    ].join("\n"),
  );

  assert.equal(audit.declarations, 3);
  assert.equal(audit.optionalMembers, 3);
  assert.deepEqual(audit.problems, [
    'fixture.ts:4 optional option "hidden" must document @default',
    'fixture.ts:11 optional option "missing" must document @default',
    'fixture.ts:14 export "undocumented" has no JSDoc block directly above',
  ]);
});

void test("re-exports are exempt and an overload group is documented once", () => {
  const audit = auditDocumentation(
    "fixture.ts",
    [
      'export * from "./dependency.ts";',
      'export { dependency } from "./dependency.ts";',
      "/** Documented overload group. */",
      "export function parse(value: string): string;",
      "export function parse(value: number): string;",
      "export function parse(value: string | number): string { return String(value); }",
    ].join("\n"),
  );

  assert.equal(audit.declarations, 3);
  assert.equal(audit.optionalMembers, 0);
  assert.deepEqual(audit.problems, []);
});

void test("public source graphs follow relative re-exports once", () => {
  const fixtureRoot = new URL("test-support/fixtures/api-documentation/", import.meta.url);
  const audit = auditSourceGraph(
    [new URL("index.ts", fixtureRoot), new URL("dependency.ts", fixtureRoot)],
    fixtureRoot,
  );

  assert.equal(audit.declarations, 3);
  assert.equal(audit.optionalMembers, 1);
  assert.deepEqual(audit.problems, []);
});

void test("public source graphs cannot escape their declared root", () => {
  const fixtureRoot = new URL("test-support/fixtures/api-documentation/", import.meta.url);

  assert.throws(
    () => auditSourceGraph([new URL("../../api-documentation.ts", fixtureRoot)], fixtureRoot),
    /public source graph escaped its source root/,
  );
});
