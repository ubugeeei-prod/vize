import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { directory, hash, root, rule } from "./support/patina-attribute-ignore-contract.ts";

const bytes = fs.readFileSync(path.join(directory, "runtime-controls.original.json"));
assert.equal(hash(bytes), "96711d8ae2e3d73365ed768b3acff4334bf0d241cbf13764f4a95102a076a5fa");
const packet = JSON.parse(bytes.toString()) as {
  producer: { bytes: number; sha256: string; revision: string };
  rows: { id: string; input: unknown; outcome: unknown }[];
};
const original = fs.readFileSync(path.join(directory, "vite-runtime-6852.mjs.fixture"));
assert.equal(original.length, packet.producer.bytes);
assert.equal(hash(original), "3efc312a8430dac0e331dc823fd3fd134f59c694404b3283da0eb851b6663de0");
assert.equal(hash(original), packet.producer.sha256);
const legacy = (await import("data:text/javascript;base64," + original.toString("base64"))) as {
  projectViteConfigSource(this: void, value: unknown): Promise<unknown>;
};
async function outcome(project: (value: unknown) => Promise<unknown>, value: unknown) {
  try {
    return { kind: "returned", value: await project(structuredClone(value)) };
  } catch (error) {
    assert(error instanceof Error);
    return { kind: "threw", error: { name: error.name, message: error.message } };
  }
}

test("extracted runtime keeps sixteen whole original enum, union and scoped outcomes", async () => {
  assert.equal(packet.rows.length, 16);
  for (const row of packet.rows) {
    assert.deepEqual(await outcome(legacy.projectViteConfigSource, row.input), row.outcome, row.id);
    assert.deepEqual(await outcome(projectViteConfigSource, row.input), row.outcome, row.id);
  }
});

test("current runtime refuses invalid attribute objects and projects an authored Vite+ object policy", async () => {
  for (const policy of [
    { ignore: [""] },
    { ignore: ["a:exit"] },
    { ignore: ["x", "x"] },
    { ignore: [], unknown: true },
  ]) {
    assert.deepEqual(
      await outcome(projectViteConfigSource, {
        vue: { linter: { ruleOptions: { [rule]: policy } } },
      }),
      {
        kind: "threw",
        error: {
          name: "TypeError",
          message: `vue.linter.ruleOptions.${rule}: must match one documented native option type`,
        },
      },
    );
  }
  const policy = { style: "never", ignore: ["active"] };
  const input = {
    vue: {
      lint: {
        jsPlugins: ["oxlint-plugin-vize"],
        plugins: ["vue"],
        settings: {},
        rules: { [`vize/${rule}`]: ["warn", policy] },
      },
    },
  };
  const before = structuredClone(input);
  assert.deepEqual(await outcome(projectViteConfigSource, input), {
    kind: "returned",
    value: {
      typeChecker: { jsxTypecheck: true },
      linter: { rules: { [rule]: "warn" }, ruleOptions: { [rule]: policy } },
    },
  });
  assert.deepEqual(input, before);
});

test("Git keeps byte-qualified native runtime sources LF even with autocrlf enabled", () => {
  const files = [
    "crates/vize_carton/src/config/loader/vite-runtime.mjs",
    "crates/vize_carton/src/config/loader/vite-runtime-validation.js",
  ];
  const actual = execFileSync(
    "git",
    ["-c", "core.autocrlf=true", "check-attr", "-z", "text", "eol", "--", ...files],
    { cwd: root, encoding: "utf8" },
  );
  assert.deepEqual(
    actual.split("\0"),
    files.flatMap((file) => [file, "text", "set", file, "eol", "lf"]).concat(""),
  );
});
