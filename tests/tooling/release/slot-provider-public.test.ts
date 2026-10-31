import assert from "node:assert/strict";
import { test } from "node:test";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, matchesGlob, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  loadFixtures,
  installedManifest,
  type Stage,
} from "../../../tools/support/release/slot_provider_public/fixtures.ts";
import { fixturePins } from "../../../tools/support/release/slot_provider_public/fixture-pins.ts";
import {
  publicVersion,
  requiredSource,
  assertPublishedManifest,
  validateCapsule,
  validateNativeReceipt,
  assertLockedPackage,
} from "../../../tools/support/release/slot_provider_public/authority.ts";
import { sha256 } from "../../../tools/support/release/computed_event_custody.ts";
import {
  diagnostics,
  assertCompilerVectors,
} from "../../../tools/support/release/slot_provider_public/oracle.ts";
import {
  assertTypeIdentity,
  type slotProviderTypes,
} from "../../../tools/support/release/slot_provider_public/types.ts";
import { nativeJournal } from "../../../tools/support/release/slot_provider_public/custody.ts";
import { hook } from "../../../tools/support/release/editor_vite_stub/custody.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const fixtures = loadFixtures(root);
test("workflow contract trigger covers real local imports, hook URLs and every pinned input", () => {
  const workflow = readFileSync(
    join(root, ".github/workflows/published-slot-provider.yml"),
    "utf8",
  );
  const block = workflow.split("    paths:\n")[1].split("  workflow_dispatch:")[0];
  const patterns = [...block.matchAll(/^\s*-\s+(.+)$/gmu)].map((match) => match[1]);
  const covered = (name: string) => patterns.some((pattern) => matchesGlob(name, pattern));
  const pending = [
    "tests/tooling/release/slot-provider-public.test.ts",
    "tests/tooling/release/slot-provider-cli-files.test.ts",
  ];
  const seen = new Set<string>();
  for (let index = 0; index < pending.length; index++) {
    const name = pending[index];
    if (seen.has(name)) continue;
    seen.add(name);
    assert.ok(covered(name), "missing PR trigger for local dependency: " + name);
    const text = readFileSync(join(root, name), "utf8");
    for (const match of text.matchAll(/(?:from\s+|import\(\s*|new URL\(\s*)["'](\.[^"']+)["']/gu)) {
      const target = resolve(root, dirname(name), match[1]);
      if (target.includes("/node_modules/") || !/\.(?:ts|cts|mts|js|cjs|mjs)$/u.test(target))
        continue;
      if (existsSync(target)) pending.push(relative(root, target).replaceAll("\\", "/"));
    }
  }
  for (const name of Object.keys(fixturePins))
    assert.ok(covered(name), "missing PR trigger for frozen input: " + name);
  assert.ok(
    seen.has("tools/support/release/public_install/native-custody.cjs"),
    "the hook is a real runtime URL dependency",
  );
});
const stage = (population: string, id: string) => {
  const row = fixtures.stages.find((row) => row.population === population && row.id === id);
  assert.ok(row);
  return row;
};
test("frozen original10 stays first and separate from 7+15+16 owner stages", () => {
  assert.deepEqual(
    ["original10", "imported7", "registered15", "collision16"].map(
      (population) => fixtures.stages.filter((row) => row.population === population).length,
    ),
    [10, 7, 15, 16],
  );
  assert.ok(fixtures.stages.slice(0, 10).every((row) => row.population === "original10"));
  assert.equal(stage("original10", "unicode-crlf-unknown").expected[0].column, 56);
  assert.equal(stage("registered15", "unicode-any").expected[0].column, 53);
  assert.ok(
    stage("original10", "unicode-crlf-unknown").files.get("src/Tiny.vue")!.includes("\r\n"),
  );
});
test("one collision provider changes in a labelled repair; original source remains whole", () => {
  for (const invalid of fixtures.stages.filter((row) => row.collisionInvalid)) {
    const repair = stage("collision16", invalid.id.replace("/invalid", "/repair"));
    assert.equal(repair.repairCopy, true);
    assert.equal(repair.files.get("src/Tiny.vue"), invalid.files.get("src/Tiny.vue"));
    assert.equal(
      repair.files.get("globals.d.ts"),
      invalid.files
        .get("globals.d.ts")!
        .replace(": any;", ": typeof import('./component').default;"),
    );
    assert.deepEqual(repair.expected, []);
  }
});
test("exact fixture hash refuses changed source rather than regenerating expectations", () => {
  const directory = mkdtempSync(join(tmpdir(), "slot-provider-fixture-control-"));
  try {
    for (const file of Object.keys(fixturePins)) {
      mkdirSync(dirname(join(directory, file)), { recursive: true });
      writeFileSync(join(directory, file), readFileSync(join(root, file)));
    }
    const file =
      "tests/_fixtures/differential/typechecker/unknown-component-slot-payload-11027/source.vue.txt";
    writeFileSync(join(directory, file), readFileSync(join(directory, file), "utf8") + "\n");
    assert.throws(() => loadFixtures(directory), /source\.vue\.txt/u);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
test("literal historical version and published mapper exec refuse future/source substitution", () => {
  assert.equal(installedManifest(publicVersion).dependencies.vize, "0.616.0");
  assert.throws(() => installedManifest("0.617.0"));
  const manifest = {
    name: "vize",
    version: publicVersion,
    bin: { vize: "bin/vize", vz: "bin/vize" },
    typescript: { contentMapper: { exec: ["node", "./bin/vize", "content-mapper"] } },
  };
  assertPublishedManifest(manifest, publicVersion);
  assert.throws(() =>
    assertPublishedManifest(
      { ...manifest, typescript: { contentMapper: { exec: ["/source/vize", "content-mapper"] } } },
      publicVersion,
    ),
  );
});
function mockedAuthority() {
  const candidate = "5a2f6667157c6758e0088e1fd0cae5fc6d36ee43";
  const image = { target: "linux-x64", bytes: 123, sha256: "b".repeat(64), token: "c".repeat(32) };
  const receipt = {
    schema: "vize.platformVsix.v1",
    version: publicVersion,
    source: candidate,
    provider: {
      source: candidate,
      releaseVersion: publicVersion,
      targetPlatform: image.target,
      nativeBuildToken: image.token,
      nodeFile: "vize.node",
      files: [{ path: "vize.node", bytes: image.bytes, sha256: image.sha256 }],
    },
  };
  const raw = Buffer.from(JSON.stringify(receipt));
  const integrity = "sha512-" + Buffer.alloc(64, 5).toString("base64");
  const row = {
    integrity,
    tarball: "https://registry.npmjs.org/vize/-/vize-0.616.0.tgz",
    metadataOrigin: "https://registry.npmjs.org/vize/0.616.0",
  };
  const capsule = {
    version: publicVersion,
    tagSourceSha: candidate,
    producerSource: requiredSource,
    npmSRI: { vize: row },
    receipts: [
      {
        rawBase64: raw.toString("base64"),
        sha256: sha256(raw),
        origin: { runId: "41204556987", artifactId: 123, member: "receipt.json" },
      },
    ],
  };
  return { candidate, image, capsule, row };
}
test("mocked capsule binds exact historical source/tag and one matching native receipt", () => {
  const { candidate, image, capsule } = mockedAuthority();
  validateCapsule(capsule, publicVersion, candidate, requiredSource);
  validateNativeReceipt(capsule, image, publicVersion, candidate, "41204556987");
  assert.throws(() => validateCapsule(capsule, publicVersion, "a".repeat(40), requiredSource));
  for (const changed of [
    { ...image, token: "d".repeat(32) },
    { ...image, bytes: 124 },
    { ...image, sha256: "a".repeat(64) },
  ])
    assert.throws(() =>
      validateNativeReceipt(capsule, changed, publicVersion, candidate, "41204556987"),
    );
  const damaged = structuredClone(capsule);
  damaged.receipts[0].sha256 = "a".repeat(64);
  assert.throws(() =>
    validateNativeReceipt(damaged, image, publicVersion, candidate, "41204556987"),
  );
  assert.throws(() =>
    validateNativeReceipt(
      { ...capsule, receipts: [...capsule.receipts, ...capsule.receipts] },
      image,
      publicVersion,
      candidate,
      "41204556987",
    ),
  );
});
test("mocked exact registry lock refuses source links, changed SRI and archive origin", () => {
  const { capsule, row } = mockedAuthority();
  const locked = { version: publicVersion, integrity: row.integrity, resolved: row.tarball };
  const lock = (value: object) => ({ packages: { "node_modules/vize": value } });
  assertLockedPackage(lock(locked), capsule, "vize", publicVersion);
  for (const value of [
    { ...locked, version: "0.617.0" },
    { ...locked, link: true },
    { ...locked, integrity: "sha512-" + Buffer.alloc(64, 6).toString("base64") },
    { ...locked, resolved: "https://example.test/vize.tgz" },
  ])
    assert.throws(() => assertLockedPackage(lock(value), capsule, "vize", publicVersion));
});

const numberFailure = Buffer.from(
  "src/Tiny.vue(13,51): error TS2345: Argument of type '\"any\"' is not assignable to parameter of type '\"number\"'.\n",
);
test("complete mapper/framework vector preserves exact positions and continuation bytes", () => {
  const row = stage("imported7", "declared-any");
  assertCompilerVectors(row, numberFailure, numberFailure, "/project");
  for (const mutation of [
    Buffer.from(numberFailure.toString().replace("(13,51)", "(13,52)")),
    Buffer.from(numberFailure.toString().replace("number", "unknown")),
    Buffer.concat([numberFailure, numberFailure]),
    Buffer.from(numberFailure.toString().trimEnd()),
  ])
    assert.throws(() => assertCompilerVectors(row, mutation, numberFailure, "/project"));
  assert.throws(() => diagnostics(Buffer.concat([numberFailure, Buffer.from("\n")]), "/project"));
  assert.notDeepEqual(
    diagnostics(Buffer.from(numberFailure.toString().replace(".\n", ". \n")), "/project"),
    diagnostics(numberFailure, "/project"),
  );
  assert.notDeepEqual(
    diagnostics(Buffer.concat([numberFailure, Buffer.from("  continuation \n")]), "/project"),
    diagnostics(Buffer.concat([numberFailure, Buffer.from("  continuation\n")]), "/project"),
  );
});
function syntheticTypes(row: Stage): ReturnType<typeof slotProviderTypes> {
  const kind = row.kind!;
  const flags = { number: 64, any: 1, error: 1, unknown: 2, never: 262144, string: 32 };
  const common = {
    start: 0,
    end: 4,
    text: "item",
    display: kind,
    symbol: null,
    debugIntrinsicName: null,
  };
  return {
    compiler: "6.0.3",
    file: "/control.vue.ts",
    roots: [],
    physicalInputs: [],
    sourceSha256: "a".repeat(64),
    rows: [
      ...row.slotRoles.map((role) => ({
        ...common,
        role: "slot-binding",
        flags: flags[kind],
        any: kind === "any" || kind === "error",
        intrinsicName: kind,
        ownerFunction: role === "props" ? "_slot_props_default_0" : "_slot_default_0",
      })),
      {
        ...common,
        role: "slot-report",
        ownerFunction: null,
        flags: kind === "error" ? 1 : 1024,
        any: kind === "error",
        display: kind === "error" ? "any" : JSON.stringify(kind === "string" ? "other" : kind),
        intrinsicName: kind === "error" ? "error" : null,
      },
    ],
    diagnostics: row.expected.map(({ code, message }) => ({
      file: "/control.vue.ts",
      start: 0,
      length: 1,
      code,
      message,
    })),
  };
}
test("original slot-string binding remains string but its actual report literal is other", () => {
  const row = stage("original10", "slot-string"),
    observed = syntheticTypes(row);
  assertTypeIdentity(row, observed);
  assert.equal(observed.rows[0].flags, 32);
  observed.rows[0].flags = 4;
  assert.throws(() => assertTypeIdentity(row, observed));
  observed.rows[0].flags = 32;
  observed.rows.at(-1)!.display = '"string"';
  assert.throws(() => assertTypeIdentity(row, observed));
});
test("original never control uses the pinned TS6 Never flag, not NonPrimitive", () => {
  const row = stage("original10", "slot-never"),
    observed = syntheticTypes(row);
  assert.equal(observed.rows[0].flags, 262144);
  assertTypeIdentity(row, observed);
  observed.rows[0].flags = 131072;
  assert.throws(() => assertTypeIdentity(row, observed));
});
for (const id of ["declared-any", "error-any"])
  test("regular/error-any identity refusal: " + id, () => {
    const row = stage("imported7", id),
      observed = syntheticTypes(row);
    assertTypeIdentity(row, observed);
    observed.rows[0].intrinsicName = id === "error-any" ? "any" : "error";
    assert.throws(() => assertTypeIdentity(row, observed));
  });

function syntheticJournal(directory: string, negative: boolean) {
  const installed = {
    root: "/public",
    cli: "/public/node_modules/vize/bin/vize",
    native: {
      path: "/public/node_modules/@vizejs/native-linux-x64-gnu/vize.node",
      bytes: 1,
      sha256: "a".repeat(64),
    },
  };
  const child = {
    pid: 12345,
    argv: [process.execPath, "--require", hook, installed.cli, "check"],
    status: negative ? 1 : 0,
    error: null,
    signal: null,
  };
  const base = { schema: "vize-public-native-custody-event-v1", pid: child.pid };
  const rows = [
    {
      ...base,
      event: "initialized",
      installRoot: installed.root,
      nativePath: installed.native.path,
      sha256: installed.native.sha256,
      node: process.execPath,
      argv: [process.execPath, installed.cli, "check"],
    },
    ...["attempt", "returned"].map((event) => ({
      ...base,
      event,
      actualPath: installed.native.path,
      sha256: installed.native.sha256,
      expectedNative: true,
      corsaPath: "",
    })),
    ...(negative ? [] : [{ ...base, event: "exit", code: 0 }]),
  ];
  const journal = join(directory, "synthetic-journal.ndjson");
  const save = () =>
    writeFileSync(journal, rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
  save();
  return { installed, child, rows, journal, save };
}
for (const negative of [false, true])
  test("exact native schema/PID/exit contract " + negative, () => {
    const directory = mkdtempSync(join(tmpdir(), "slot-provider-journal-control-"));
    try {
      const fixture = syntheticJournal(directory, negative);
      nativeJournal(fixture.journal, fixture.installed, fixture.child);
      assert.throws(() =>
        nativeJournal(fixture.journal, fixture.installed, {
          ...fixture.child,
          pid: fixture.child.pid + 1,
        }),
      );
      assert.throws(() =>
        nativeJournal(fixture.journal, fixture.installed, { ...fixture.child, signal: "SIGTERM" }),
      );
      fixture.rows[1].pid++;
      fixture.save();
      assert.throws(() => nativeJournal(fixture.journal, fixture.installed, fixture.child));
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
