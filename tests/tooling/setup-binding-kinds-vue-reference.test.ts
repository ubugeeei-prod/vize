import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

type ReferenceFixture = {
  id: string;
  hoistStatic?: boolean;
  source: string;
  referenceBindings: Record<string, string>;
};

type ReferencePack = {
  reference: { package: string; version: string; api: string };
  fixtures: ReferenceFixture[];
};

const sortBindingEntries = (left: [string, string], right: [string, string]) =>
  left[0].localeCompare(right[0]);

const stableVueRequire = createRequire(new URL("../../npm/devtools/package.json", import.meta.url));
const compiler = stableVueRequire("@vue/compiler-sfc") as {
  parse(source: string, options: { filename: string }): { descriptor: unknown; errors: unknown[] };
  compileScript(
    descriptor: unknown,
    options: { id: string; hoistStatic: boolean },
  ): { bindings: Record<string, string> };
};
const packageVersion = (stableVueRequire("@vue/compiler-sfc/package.json") as { version: string })
  .version;
const fixtureUrl = new URL(
  "../../crates/vize_atelier_sfc/tests/fixtures/setup_binding_kinds_vue_3_5_35.json",
  import.meta.url,
);
const pack = JSON.parse(readFileSync(fixtureUrl, "utf8")) as ReferencePack;

test("setup binding reference equals pinned Vue compileScript metadata", () => {
  assert.deepEqual(pack.reference, {
    package: "@vue/compiler-sfc",
    version: "3.5.35",
    api: "compileScript",
  });
  assert.equal(packageVersion, pack.reference.version);

  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, { filename: `${fixture.id}.vue` });
    assert.deepEqual(parsed.errors, [], fixture.id);
    const compiled = compiler.compileScript(parsed.descriptor, {
      id: fixture.id,
      hoistStatic: fixture.hoistStatic ?? true,
    });
    const actual = Object.fromEntries(
      Object.entries(compiled.bindings).toSorted(sortBindingEntries),
    );
    const expected = Object.fromEntries(
      Object.entries(fixture.referenceBindings).toSorted(sortBindingEntries),
    );
    assert.deepEqual(actual, expected, fixture.id);
  }
});
