import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import type { Page } from "playwright";
import { selectLiteral } from "../../../tools/support/release/public_acceptance/vrt_trusted_ui.ts";
import {
  literalVariants,
  literalVrtSources,
  materializeLiteralVrtInputs,
  readLiteralVrtInputs,
} from "../../../tools/support/release/public_acceptance/vrt_literal.ts";

const fixtures = fileURLToPath(new URL("../fixtures/musea/literal-variant-keys/", import.meta.url));
const vector = fileURLToPath(
  new URL("../../_fixtures/differential/musea/literal-variant-keys.json", import.meta.url),
);
function read(source: string) {
  assert.ok(literalVrtSources.includes(source as (typeof literalVrtSources)[number]));
  return readFileSync(
    source.endsWith(".json") ? vector : path.join(fixtures, path.basename(source)),
  );
}

// Authored input/filesystem controls only; never call the installed build or any surface factory.
void test("all exact Controls2/Keys3 inputs materialize once with full byte custody", async (t) => {
  const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-input-"));
  t.after(() => rm(consumer, { recursive: true }));
  const records = materializeLiteralVrtInputs(consumer, read);
  assert.equal(records.length, 3);
  assert.deepEqual(
    records.map((record) => record.sourceFile),
    [...literalVrtSources],
  );
  for (const record of records)
    assert.deepEqual(readFileSync(path.join(consumer, record.file)), read(record.sourceFile));
  const sources = readLiteralVrtInputs(consumer);
  assert.deepEqual(
    sources.map((source) => source.path),
    literalVrtSources.slice(0, 2),
  );
  for (const source of sources) assert.deepEqual(source.bytes, read(source.path));
  assert.deepEqual(
    literalVariants.Controls.map(([name]) => name),
    ["Default", "Custom theme"],
  );
  assert.deepEqual(
    literalVariants.Keys.map(([name]) => name),
    ["__proto__", "constructor", "hasOwnProperty"],
  );
  assert.throws(() => materializeLiteralVrtInputs(consumer, read), { code: "EEXIST" });
});

void test("either altered authored Art is refused by the full frozen source hash", async (t) => {
  for (const title of ["Controls", "Keys"]) {
    const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-art-refusal-"));
    t.after(() => rm(consumer, { recursive: true }));
    materializeLiteralVrtInputs(consumer, read);
    writeFileSync(path.join(consumer, "trusted-vrt-inputs", `${title}.art.vue`), "changed\n");
    assert.throws(() => readLiteralVrtInputs(consumer), /Frozen literal authored bytes changed/);
  }
});

void test("changed vector cannot authorize a foreign source path or fewer reserved names", async (t) => {
  const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-vector-refusal-"));
  t.after(() => rm(consumer, { recursive: true }));
  materializeLiteralVrtInputs(consumer, read);
  const changed = JSON.parse(readFileSync(vector, "utf8"));
  changed.sources[0].path = "../../foreign.art.vue";
  changed.variants.Keys = ["__proto__"];
  writeFileSync(
    path.join(consumer, "trusted-vrt-inputs/literal-variant-keys.json"),
    JSON.stringify(changed),
  );
  assert.throws(() => readLiteralVrtInputs(consumer), /Frozen literal vector changed/);
});

test("literal navigation selects the next Art before mounting its Variants", async () => {
  for (const title of ["Controls", "Keys"] as const) {
    const calls: string[] = [];
    let selected = "previous-Art",
      mounted = false;
    const root = "/authored";
    const gallery = "https://owned.example/gallery/";
    const rows = literalVariants[title].map(([name, button], index) => ({
      name,
      buttons: [button],
      unresolved: 0,
      id:
        title === "Controls"
          ? ["variant-default", "variant-custom-theme"][index]
          : ["variant-proto", "variant-constructor", "variant-hasownproperty"][index],
      aria:
        title === "Controls"
          ? ["variant-default", "variant-custom-theme"][index]
          : ["variant-proto", "variant-constructor", "variant-hasownproperty"][index],
      url: new URL(`preview/${index}.html`, gallery).href,
    }));
    const page = {
      locator(selector: string) {
        return {
          filter({ hasText }: { hasText: string }) {
            assert.equal(selector, ".art-item");
            return {
              async click() {
                assert.equal(hasText, title);
                selected = title;
                calls.push(`Art:${title}`);
              },
            };
          },
          last() {
            return {
              async click() {
                assert.ok(mounted);
              },
            };
          },
          first() {
            return {
              async click() {
                assert.ok(mounted);
              },
              async waitFor() {
                assert.ok(mounted);
              },
            };
          },
        };
      },
      getByRole(role: string, options: { name: string; exact: boolean }) {
        assert.deepEqual([role, options], ["button", { name: "Variants", exact: true }]);
        return {
          async click() {
            assert.equal(selected, title, "old Art previews must never mount");
            mounted = true;
            calls.push("Variants");
          },
        };
      },
      frameLocator(selector: string) {
        assert.equal(selector, ".variant-card iframe");
        return {
          nth(index: number) {
            return {
              getByRole(role: string, options: { name: string; exact: boolean }) {
                assert.deepEqual(
                  [role, options],
                  ["button", { name: literalVariants[title][index][1], exact: true }],
                );
                return {
                  async waitFor(options: { timeout: number }) {
                    assert.equal(options.timeout, 15000);
                    assert.ok(mounted);
                  },
                };
              },
            };
          },
        };
      },
      async evaluate() {
        assert.ok(mounted);
        return rows;
      },
    };
    await assert.rejects(
      page.getByRole("button", { name: "Variants", exact: true }).click(),
      /old Art previews/,
    );
    const build = {
      root,
      manifest: {
        previews: {
          [path.join(root, `src/${title}.art.vue`)]: Object.fromEntries(
            rows.map((row, index) => [row.name, `preview/${index}.html`]),
          ),
        },
      },
    };
    assert.deepEqual(
      await selectLiteral(
        page as unknown as Page,
        title,
        build as Parameters<typeof selectLiteral>[2],
        gallery,
      ),
      rows,
    );
    assert.deepEqual(calls, [`Art:${title}`, "Variants"]);
  }
  const caller = readFileSync(
    new URL(
      "../../../tools/support/release/public_acceptance/vrt_trusted_observation.ts",
      import.meta.url,
    ),
    "utf8",
  );
  assert.equal(caller.match(/await selectLiteral\(page, title, build, galleryUrl\)/g)?.length, 2);
  assert.doesNotMatch(
    caller,
    /getByRole\("button", \{ name: "Variants", exact: true \}\)\.click\(\)/,
  );
});
