import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("shared mapper types are generated from the canonical standalone helper bodies", () => {
  const checked = spawnSync(
    process.execPath,
    ["npm/cli/scripts/generate-content-mapper-types.ts", "--check"],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    },
  );
  assert.equal(checked.status, 0, `${checked.stdout}${checked.stderr}`);
  const generated = readFileSync(
    path.join(root, "npm/cli/generated/content-mapper-types.d.ts"),
    "utf8",
  );
  assert.doesNotMatch(generated, /declare\s+(global|namespace|function|var|const)/u);
  const manifest = JSON.parse(readFileSync(path.join(root, "npm/cli/package.json"), "utf8")) as {
    files: string[];
  };
  assert.ok(manifest.files.includes("generated/content-mapper-types.d.ts"));
});

test("ESM and CommonJS share exact helper types and observe the consumer Vue augmentation", () => {
  for (const input of [
    "npm/cli/tests/content-mapper-input.test-d.ts",
    "npm/cli/tests/content-mapper-input.test-d.cts",
  ]) {
    const checked = spawnSync(
      process.execPath,
      [
        path.join(root, "npm/cli/node_modules/typescript/bin/tsc"),
        "--ignoreConfig",
        "--noEmit",
        "--strict",
        "--target",
        "ES2022",
        "--module",
        "Node16",
        "--moduleResolution",
        "Node16",
        input,
      ],
      { cwd: root, encoding: "utf8", timeout: 60_000 },
    );
    assert.equal(checked.status, 0, `${input}\n${checked.stdout}${checked.stderr}`);
  }
});

const slotAlias = "$SlotConstructorWithoutArguments";
const publicFiles = ["ts", "mts", "cts"].map((extension) => `npm/cli/virtual-ts.d.${extension}`);
const embedded = "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-resolver.d.ts.txt";
const shared =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-resolver.d.ts.txt";
function generatorFixture() {
  const cwd = mkdtempSync(path.join(tmpdir(), "vize-slot-export-generator-"));
  const files = [
    "npm/cli/scripts/generate-content-mapper-types.ts",
    "npm/cli/package.json",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/prop-keys.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
    "npm/cli/generated/content-mapper-types.d.ts",
    "npm/cli/virtual-ts-globals.d.ts",
    "npm/cli/virtual-ts-patterns.d.ts",
    embedded,
    shared,
    ...publicFiles,
  ];
  for (const file of files) {
    mkdirSync(path.dirname(path.join(cwd, file)), { recursive: true });
    copyFileSync(path.join(root, file), path.join(cwd, file));
  }
  symlinkSync(path.join(root, "node_modules"), path.join(cwd, "node_modules"), "dir");
  symlinkSync(
    path.join(root, "npm/cli/node_modules"),
    path.join(cwd, "npm/cli/node_modules"),
    "dir",
  );
  return {
    cwd,
    run: (check = true) =>
      spawnSync(
        process.execPath,
        ["npm/cli/scripts/generate-content-mapper-types.ts", ...(check ? ["--check"] : [])],
        { cwd, encoding: "utf8", timeout: 30_000 },
      ),
    cleanup: () => rmSync(cwd, { recursive: true, force: true }),
  };
}
function publicAlias(source: string, replacement: string) {
  const start = source.indexOf(`export type ${slotAlias}<`);
  const end = source.indexOf("export type $SlotConstructorWithArguments<", start);
  assert.ok(start >= 0 && end > start, "real adjacent public declarations required");
  return source.slice(0, start) + replacement + "\n" + source.slice(end);
}
const previous = `export type ${slotAlias}<C> = [$SlotConstructorHasArguments<C>] extends [never] ? C : new () => { $slots: never };`;
test("the one zero-call export regenerates from both canonical bodies and is idempotent", (t) => {
  const fixture = generatorFixture();
  t.after(fixture.cleanup);
  for (const file of publicFiles) {
    const target = path.join(fixture.cwd, file);
    writeFileSync(target, publicAlias(readFileSync(target, "utf8"), previous));
  }
  for (const check of [false, false, true]) {
    const result = fixture.run(check);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    for (const file of publicFiles) {
      assert.equal(
        readFileSync(path.join(fixture.cwd, file), "utf8"),
        readFileSync(path.join(root, file), "utf8"),
        file,
      );
    }
  }
});
for (const [name, file, mutate, reason] of [
  [
    "missing embedded alias",
    embedded,
    (source: string) => source.replace(`type ${slotAlias}<`, "type $LostSlotAlias<"),
    /must declare exactly one/u,
  ],
  [
    "duplicate shared alias",
    shared,
    (source: string) => source + `\nexport type ${slotAlias}<C> = C;\n`,
    /must declare exactly one/u,
  ],
  [
    "canonical policy drift",
    embedded,
    (source: string) => {
      const policy =
        "C & ([$SlotConstructorHasArguments<C>] extends [never] ? unknown : new () => { $slots: never })";
      assert.equal(source.split(policy).length - 1, 1, "exact canonical guard required");
      return source.replace(policy, policy.replace("$slots: never", "$slots: unknown"));
    },
    /policies drifted/u,
  ],
  [
    "shared visibility drift",
    shared,
    (source: string) => source.replace(`export type ${slotAlias}<`, `type ${slotAlias}<`),
    /export visibility drifted/u,
  ],
  [
    "missing public alias",
    publicFiles[0]!,
    (source: string) => source.replace(`export type ${slotAlias}<`, "export type $LostSlotAlias<"),
    /must declare exactly one/u,
  ],
  [
    "duplicate public alias",
    publicFiles[1]!,
    (source: string) => source + `\nexport type ${slotAlias}<C> = C;\n`,
    /must declare exactly one/u,
  ],
  [
    "unknown public policy",
    publicFiles[2]!,
    (source: string) => publicAlias(source, `export type ${slotAlias}<C> = C;`),
    /refuse to overwrite drift/u,
  ],
  [
    "public parameter drift",
    publicFiles[0]!,
    (source: string) =>
      source.replace(`export type ${slotAlias}<C>`, `export type ${slotAlias}<C extends unknown>`),
    /must not constrain C/u,
  ],
] as const) {
  test(`zero-call export generation refuses ${name}`, (t) => {
    const fixture = generatorFixture();
    t.after(fixture.cleanup);
    const target = path.join(fixture.cwd, file);
    const before = readFileSync(target, "utf8");
    const changed = mutate(before);
    assert.notEqual(before, changed, "mutation must affect the real input");
    writeFileSync(target, changed);
    const result = fixture.run();
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, reason);
    assert.equal(readFileSync(target, "utf8"), changed, "refusal preserves the unsupported input");
  });
}
test("unknown later SDK input prevents an earlier legacy alias from being overwritten", (t) => {
  const fixture = generatorFixture();
  t.after(fixture.cleanup);
  const first = path.join(fixture.cwd, publicFiles[0]!);
  const legacy = publicAlias(readFileSync(first, "utf8"), previous);
  writeFileSync(first, legacy);
  const second = path.join(fixture.cwd, publicFiles[1]!);
  writeFileSync(
    second,
    publicAlias(readFileSync(second, "utf8"), `export type ${slotAlias}<C> = C;`),
  );
  const result = fixture.run(false);
  assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
  assert.match(`${result.stdout}${result.stderr}`, /refuse to overwrite drift/u);
  assert.equal(readFileSync(first, "utf8"), legacy);
});
