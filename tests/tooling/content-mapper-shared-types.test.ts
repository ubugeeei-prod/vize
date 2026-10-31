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
import { verifyPublicSlotInstance } from "./support/public-slot-instance.ts";

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
    "npm/cli/scripts/slot-payload-exports.ts",
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
    "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-payload.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-payload.d.ts.txt",
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
const oldInstance = `export type $SlotConstructorInstance<C, Z, P> = [$SlotConstructorHasArguments<C>] extends [never]\n  ? Z\n  : P;`;
function previousInstance(source: string) {
  const start = source.indexOf("type $SlotContractIsUsable<");
  const end = source.indexOf("export type $SlotHostSelector<", start);
  assert.ok(start >= 0 && end > start, "actual complete Instance closure required");
  return source.slice(0, start) + oldInstance + "\n" + source.slice(end);
}
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

test("the complete Instance dependency closure regenerates without replacing either existing family", (t) => {
  const fixture = generatorFixture();
  t.after(fixture.cleanup);
  for (const file of publicFiles) {
    const target = path.join(fixture.cwd, file);
    writeFileSync(target, previousInstance(publicAlias(readFileSync(target, "utf8"), previous)));
  }
  for (const check of [false, false, true]) {
    const result = fixture.run(check);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    for (const file of publicFiles)
      assert.equal(
        readFileSync(path.join(fixture.cwd, file), "utf8"),
        readFileSync(path.join(root, file), "utf8"),
      );
  }
});

for (const [name, file, mutate, reason] of [
  [
    "missing canonical dependency",
    embedded,
    (source: string) => source.replace("type $SlotContractHasKeys<", "type $MissingSlotKeys<"),
    /must declare exactly one/u,
  ],
  [
    "duplicate canonical dependency",
    shared,
    (source: string) => source + "\ntype $SlotContractHasKeys<S> = true;\n",
    /must declare exactly one/u,
  ],
  [
    "canonical dependency drift",
    embedded,
    (source: string) =>
      source.replace(
        "[keyof S] extends [never] ? false : true",
        "[keyof S] extends [never] ? true : false",
      ),
    /canonical embedded\/shared policy drifted/u,
  ],
  [
    "private dependency visibility",
    shared,
    (source: string) =>
      source.replace("type $SlotContractHasKeys<", "export type $SlotContractHasKeys<"),
    /export visibility drifted/u,
  ],
  [
    "partial public closure",
    publicFiles[0]!,
    (source: string) =>
      source.replace(
        "type $SlotContractHasKeys<S> = [keyof S] extends [never] ? false : true;",
        "",
      ),
    /incomplete or duplicate Instance closure/u,
  ],
  [
    "duplicate public dependency",
    publicFiles[1]!,
    (source: string) => source + "\ntype $SlotContractHasKeys<S> = true;\n",
    /incomplete or duplicate Instance closure/u,
  ],
  [
    "public dependency declaration kind",
    publicFiles[2]!,
    (source: string) =>
      source.replace(
        "type $SlotContractHasKeys<S> = [keyof S] extends [never] ? false : true;",
        "interface $SlotContractHasKeys<S> {}",
      ),
    /must declare exactly one/u,
  ],
  [
    "public dependency parameters",
    publicFiles[0]!,
    (source: string) =>
      source.replace("type $SlotContractHasKeys<S>", "type $SlotContractHasKeys<S = unknown>"),
    /must not default S/u,
  ],
  [
    "unknown complete Instance policy",
    publicFiles[2]!,
    (source: string) =>
      source.replace(
        "? $SlotContractHasKeys<S> extends true",
        "? $SlotContractHasKeys<S> extends false",
      ),
    /Instance.*drifted/u,
  ],
  [
    "unknown previous Instance policy",
    publicFiles[2]!,
    (source: string) => previousInstance(source).replace("  ? Z\n  : P;", "  ? Z\n  : P | Z;"),
    /unknown previous Instance policy/u,
  ],
] as const) {
  test(`Instance generation refuses ${name} before writing any SDK entry`, (t) => {
    const fixture = generatorFixture();
    t.after(fixture.cleanup);
    const first = path.join(fixture.cwd, publicFiles[0]!);
    writeFileSync(first, previousInstance(readFileSync(first, "utf8")));
    const target = path.join(fixture.cwd, file);
    // The mutation starts from the real complete entry even when it is the
    // first pending legacy output, so its claimed refusal is meaningful.
    const original = readFileSync(path.join(root, file), "utf8");
    const changed = mutate(original);
    assert.notEqual(changed, original);
    writeFileSync(target, changed);
    const pending = readFileSync(first, "utf8");
    const result = fixture.run(false);
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, reason);
    assert.equal(readFileSync(first, "utf8"), pending);
    assert.equal(readFileSync(target, "utf8"), changed);
  });
}

test("public Instance retains complete before/after consumer laws in all three module entries", (t) => {
  t.diagnostic(JSON.stringify(verifyPublicSlotInstance(root, previousInstance)));
});

const payload = "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-payload.d.ts.txt";
const sharedPayload =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-payload.d.ts.txt";
test("the complete previous payload family regenerates in all three module entries", (t) => {
  const fixture = generatorFixture();
  t.after(fixture.cleanup);
  const legacy = `// @ts-ignore The unchecked payload of an untyped host: an error type, never reported.
type $SilentAny = {}["__vizeSilentAny"];
type $NamedSlotPayload<S, N> = { [K in keyof S & N]-?: NonNullable<S[K]> extends (props: infer P, ...args: any[]) => any ? P : never }[keyof S & N] extends infer P ? ([P] extends [never] ? any : P) : any;
export type $SlotPayload<S, N> = $IsAny<S> extends true ? $SilentAny : string extends N ? string extends keyof S ? $NamedSlotPayload<S, N> : any : $NamedSlotPayload<S, N>;`;
  for (const file of publicFiles) {
    const target = path.join(fixture.cwd, file);
    const current = readFileSync(target, "utf8");
    const start = current.indexOf("// @ts-ignore A missing provider keeps");
    const end = current.indexOf("\n\nexport type __EmitProps", start);
    assert.ok(start >= 0 && end > start);
    writeFileSync(target, current.slice(0, start) + legacy + current.slice(end));
  }
  for (const check of [false, false, true]) {
    const result = fixture.run(check);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    for (const file of publicFiles)
      assert.equal(
        readFileSync(path.join(fixture.cwd, file), "utf8"),
        readFileSync(path.join(root, file), "utf8"),
      );
  }
});
for (const [name, file, mutate, reason] of [
  [
    "missing canonical provider",
    payload,
    (text: string) => text.replace("type $RegisteredSlotProvider<", "type $MissingProvider<"),
    /complete canonical payload family/u,
  ],
  [
    "canonical policy drift",
    payload,
    (text: string) => text.replace("? __C : any", "? unknown : any"),
    /canonical policies drifted/u,
  ],
  [
    "canonical visibility drift",
    sharedPayload,
    (text: string) =>
      text.replace("export type $RegisteredSlotProvider<", "type $RegisteredSlotProvider<"),
    /canonical visibility/u,
  ],
  [
    "public policy drift",
    publicFiles[0]!,
    (text: string) =>
      text.replace(
        "    ? $IsAny<C> extends true\n      ? C",
        "    ? $IsAny<C> extends true\n      ? unknown",
      ),
    /unknown payload policy/u,
  ],
  [
    "duplicate public provider",
    publicFiles[1]!,
    (text: string) => text + "\nexport type $RegisteredSlotProvider<G, K> = any;\n",
    /duplicates/u,
  ],
  [
    "missing public provider",
    publicFiles[2]!,
    (text: string) =>
      text.replace("export type $RegisteredSlotProvider<", "export type $MissingProvider<"),
    /payload aliases must remain adjacent/u,
  ],
] as const) {
  test(`slot payload generation refuses ${name} before writing any SDK file`, (t) => {
    const fixture = generatorFixture();
    t.after(fixture.cleanup);
    const target = path.join(fixture.cwd, file);
    const before = readFileSync(target, "utf8");
    const changed = mutate(before);
    assert.notEqual(changed, before, "the actual source must be mutated");
    writeFileSync(target, changed);
    const first = path.join(fixture.cwd, publicFiles[0]!);
    const firstBefore = readFileSync(first, "utf8");
    const result = fixture.run(false);
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, reason);
    assert.equal(readFileSync(first, "utf8"), firstBefore);
    assert.equal(readFileSync(target, "utf8"), changed);
  });
}
