import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { Recorder, sha256 } from "../../../tools/support/config/windows_consumer/common.ts";
import { asArray, asNumber, asRecord, asString } from "../../../tools/support/json-value.ts";
import {
  assertLockedPackage,
  validateCapsule,
  validateNativeReceipt,
} from "../../../tools/support/release/constructor_original27_public/authority.ts";
import { installedManifest } from "../../../tools/support/release/constructor_original27_public/fixtures.ts";
import {
  assertOriginalVectors,
  cliPacket,
  diagnostics,
  expectedMapper,
  repairExpected,
  reference,
  sourceReference,
} from "../../../tools/support/release/constructor_original27_public/oracle.ts";
import {
  capture,
  completed,
} from "../../../tools/support/release/constructor_original27_public/capture.ts";
import {
  assertVuePackageVersions,
  assertVueProviderLineage,
  captureVueNamespace,
  vueFrameworkPackages,
} from "../../../tools/support/release/constructor_original27_public/host-versions.ts";

const version = "0.614.0",
  candidate = "c".repeat(40),
  source = "s".repeat(40),
  run = "123";
function fixture(body: (root: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "original27-contract-"));
  try {
    body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
test("source reference retains the entire original population, both orders and all debt", () => {
  const data = reference(),
    ids = asArray(data.caseIds);
  assert.equal(data.source, sourceReference);
  assert.equal(
    data.summarySha256,
    "554c41998ebccfbcca7abb4da9e0f805a5b1507b374bbd38c148447e33142c9b",
  );
  assert.equal(ids.length, 27);
  assert.equal(new Set(ids).size, 27);
  for (const id of [
    "overload-first-zero",
    "overload-last-zero",
    "generic-default-zero",
    "generic-props",
    "slot-poison-any",
    "slot-poison-unknown",
    "slot-poison-never",
    "slot-poison-string",
  ])
    assert.ok(ids.includes(id), id);
  assert.equal(asArray(data.authoredCases).length, 14);
  assert.equal(asArray(data.mapper).length, 16);
  assert.equal(asArray(data.cli).length, 14);
  assert.deepEqual(
    expectedMapper(version)
      .filter((row) => row.code === 2741)
      .map((row) => [row.file, row.line, row.column]),
    [
      ["constructor-public-props-discrepancy/src/Tiny.vue", 57, 495],
      ["constructor-public-props-discrepancy/src/Tiny.vue", 108, 501],
    ],
  );
});
test("only the two target package versions change in the original provider manifest", () => {
  const before = {
    private: true,
    dependencies: {
      vize: "0.602.0",
      "@vizejs/native": "0.602.0",
      vue: "3.5.42",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
      "vue-tsc": "3.3.13",
    },
    overrides: { "@vue/shared": "3.5.42" },
  };
  const original = structuredClone(before),
    after = installedManifest(before, version);
  assert.deepEqual(before, original);
  assert.deepEqual(after, {
    ...before,
    dependencies: {
      ...before.dependencies,
      vize: version,
      "@vizejs/native": version,
    },
  });
  assert.throws(() =>
    installedManifest({ dependencies: { ...before.dependencies, vize: version } }, version),
  );
});
test("complete vectors accept explicit public labels and reject lost or changed authored errors", () => {
  const mapper = expectedMapper(version),
    cli = structuredClone(asArray(reference().cli));
  assertOriginalVectors(version, mapper, cli);
  assert.throws(
    () => assertOriginalVectors(version, mapper.slice(1), cli),
    /complete mapper vector/,
  );
  assert.throws(() => assertOriginalVectors(version, mapper, cli.slice(1)), /complete CLI vector/);
  const changed = structuredClone(mapper);
  changed[0].code = 9999;
  assert.throws(() => assertOriginalVectors(version, changed, cli));
});
test("private generated diagnostics are not filtered, relocated or broadly relabelled", () => {
  const mapper = expectedMapper(version),
    cli = asArray(reference().cli);
  assert.equal(JSON.stringify(mapper).includes("vize@0.0.0-source-"), false);
  assert.equal(JSON.stringify(mapper).match(/vize@0\.614\.0/g)?.length, 2);
  for (const change of [
    (rows: ReturnType<typeof expectedMapper>) => rows.filter((row) => row.code !== 2741),
    (rows: ReturnType<typeof expectedMapper>) => {
      rows.find((row) => row.code === 2741)!.line = 58;
      return rows;
    },
    (rows: ReturnType<typeof expectedMapper>) => {
      rows.find((row) => row.code === 2741)!.message = "different debt";
      return rows;
    },
    (rows: ReturnType<typeof expectedMapper>) => [...rows, structuredClone(rows[0])],
  ])
    assert.throws(() => assertOriginalVectors(version, change(structuredClone(mapper)), cli));
});
test("repair copy removes only its own authored error; bad and restored populations remain exact", () => {
  const mapper = expectedMapper(version),
    cli = asArray(reference().cli);
  const keep = (row: unknown) => asRecord(row).file !== "wrong-prop/src/Tiny.vue";
  assertOriginalVectors(version, mapper.filter(keep), cli.filter(keep), true);
  assert.throws(() => assertOriginalVectors(version, mapper.filter(keep), cli.filter(keep)));
  assert.throws(() => assertOriginalVectors(version, mapper, cli, true));
  assert.throws(() =>
    assertOriginalVectors(
      version,
      mapper.filter((row) => keep(row) && row.code !== 2741),
      cli.filter(keep),
      true,
    ),
  );
  assertOriginalVectors(version, mapper, cli); // Exact restored vector, no stateful waiver.
});
test("only the two complete known source-version continuation lines may be relabelled", () => {
  const rows = structuredClone(asArray(reference().mapper));
  const changed = structuredClone(rows),
    debt = changed.map((row) => asRecord(row)).find((row) => row.code === 2741)!;
  debt.continuation = asArray(debt.continuation).map((line) => asString(line) + " ");
  assert.throws(() => expectedMapper(version, changed));
  assert.throws(() =>
    expectedMapper(
      version,
      rows.filter((row) => asRecord(row).code !== 2741),
    ),
  );
  const added = structuredClone(rows);
  asRecord(added[0]).continuation = asRecord(
    rows.find((row) => asRecord(row).code === 2741),
  ).continuation;
  assert.throws(() => expectedMapper(version, added), /unexpected source-version continuation/);
});
test("repair derivation validates one complete original row before removing only that row", () => {
  for (const role of ["mapper", "cli"] as const) {
    const rows = role === "mapper" ? expectedMapper(version) : asArray(reference().cli);
    const wrong = rows.find((row) => asRecord(row).file === "wrong-prop/src/Tiny.vue")!;
    assert.equal(repairExpected(rows, role).length, rows.length - 1);
    assert.throws(() => repairExpected([...rows, wrong], role), /exactly one/);
    assert.throws(
      () =>
        repairExpected(
          rows.filter((row) => row !== wrong),
          role,
        ),
      /exactly one/,
    );
    const changed = structuredClone(rows),
      at = rows.indexOf(wrong);
    asRecord(changed[at]).message = "changed original diagnostic";
    assert.throws(() => repairExpected(changed, role), /complete original repair row differs/);
  }
});
test("compiler parsing keeps complete continuation lines and refuses unrelated output", () => {
  const rows = diagnostics(
    Buffer.from("case/src/Tiny.vue(57,495): error TS2741: debt\n  exact continuation\n"),
    "/project",
  );
  assert.deepEqual(rows, [
    {
      file: "case/src/Tiny.vue",
      line: 57,
      column: 495,
      code: 2741,
      message: "debt",
      continuation: ["  exact continuation"],
    },
  ]);
  assert.throws(() => diagnostics(Buffer.from("warning: ignored?\n"), "/project"));
  assert.throws(() => diagnostics(Buffer.from([255]), "/project"));
});
test("whole compiler continuations preserve trailing spaces and reject blank or unterminated lines", () => {
  const mapper = expectedMapper(version),
    cli = asArray(reference().cli);
  const render = (rows: ReturnType<typeof expectedMapper>) =>
    rows
      .flatMap((row) => [
        `${asString(row.file)}(${asNumber(row.line)},${asNumber(row.column)}): error TS${asNumber(row.code)}: ${asString(row.message)}`,
        ...asArray(row.continuation).map((line) => asString(line)),
      ])
      .join("\n") + "\n";
  const raw = render(mapper);
  assertOriginalVectors(version, diagnostics(Buffer.from(raw), "/project"), cli);
  const finalHeaderSpace = raw.slice(0, -1) + " \n";
  assert.throws(() =>
    assertOriginalVectors(version, diagnostics(Buffer.from(finalHeaderSpace), "/project"), cli),
  );
  const changed = structuredClone(mapper),
    debt = changed.find((row) => row.code === 2741)!;
  debt.continuation = asArray(debt.continuation).map((line) => String(line) + " ");
  assert.throws(() =>
    assertOriginalVectors(version, diagnostics(Buffer.from(render(changed)), "/project"), cli),
  );
  assert.throws(() => diagnostics(Buffer.from(raw.replace("\n", "\n\n")), "/project"));
  assert.throws(() => diagnostics(Buffer.from(raw + "\n"), "/project"));
  assert.throws(() => diagnostics(Buffer.from(raw.slice(0, -1)), "/project"));
});
test("CLI admission requires all 54 distinct files and complete virtual payloads", () => {
  const ids = asArray(reference().caseIds).map(String);
  const files = ids
    .flatMap((id) => [id + "/component.ts", id + "/src/Tiny.vue"])
    .map((file) => ({ file, diagnostics: [], virtualTs: "retained" }));
  const encode = (rows: unknown[]) => Buffer.from(JSON.stringify({ files: rows, errorCount: 0 }));
  cliPacket(encode(files), 0, "/project", ids);
  assert.throws(() => cliPacket(encode(files.slice(1)), 0, "/project", ids));
  assert.throws(() => cliPacket(encode([...files.slice(1), files[1]]), 0, "/project", ids));
  assert.throws(() =>
    cliPacket(encode([{ ...files[0], virtualTs: null }, ...files.slice(1)]), 0, "/project", ids),
  );
});

// Synthetic receipt and registry rows test refusal behavior; none are publication evidence.
function mockedCapsule() {
  const receipt = {
    schema: "vize.platformVsix.v1",
    version,
    source: candidate,
    provider: {
      source: candidate,
      releaseVersion: version,
      targetPlatform: "linux-x64",
      nodeFile: "native.node",
      nativeBuildToken: "a".repeat(32),
      files: [{ path: "native.node", bytes: 100, sha256: "b".repeat(64) }],
    },
  };
  const raw = Buffer.from(JSON.stringify(receipt));
  return {
    version,
    tagSourceSha: candidate,
    producerSource: source,
    npmSRI: {
      vize: {
        integrity: "sha512-YQ==",
        tarball: "https://registry.npmjs.org/vize/-/vize-0.614.0.tgz",
        metadataOrigin: "https://registry.npmjs.org/vize/0.614.0",
      },
    },
    receipts: [
      {
        rawBase64: raw.toString("base64"),
        sha256: sha256(raw),
        origin: {
          runId: Number(run),
          artifactId: 456,
          member: "mocked-receipt.json",
        },
      },
    ],
  };
}
const identity = {
  target: "linux-x64",
  bytes: 100,
  sha256: "b".repeat(64),
  token: "a".repeat(32),
};
test("capsule identity cannot cross selected source, tag or version", () => {
  const capsule = mockedCapsule();
  validateCapsule(capsule, version, candidate, source);
  assert.throws(() => validateCapsule(capsule, "0.613.0", candidate, source));
  assert.throws(() => validateCapsule(capsule, version, "d".repeat(40), source));
  assert.throws(() => validateCapsule(capsule, version, candidate, "d".repeat(40)));
});
test("native admission binds one producer host receipt to physical bytes, token and run", () => {
  const capsule = mockedCapsule();
  validateNativeReceipt(capsule, identity, version, candidate, run);
  for (const changed of [
    { ...identity, bytes: 101 },
    { ...identity, sha256: "d".repeat(64) },
    { ...identity, token: "d".repeat(32) },
    { ...identity, target: "darwin-arm64" },
  ])
    assert.throws(() => validateNativeReceipt(capsule, changed, version, candidate, run));
  assert.throws(() => validateNativeReceipt(capsule, identity, version, candidate, "124"));
  assert.throws(() => validateNativeReceipt(capsule, identity, version, "d".repeat(40), run));
  assert.throws(() =>
    validateNativeReceipt({ ...capsule, receipts: [] }, identity, version, candidate, run),
  );
  assert.throws(() =>
    validateNativeReceipt(
      { ...capsule, receipts: [...capsule.receipts, ...capsule.receipts] },
      identity,
      version,
      candidate,
      run,
    ),
  );
  assert.throws(() =>
    validateNativeReceipt(
      {
        ...capsule,
        receipts: [{ ...capsule.receipts[0], sha256: "d".repeat(64) }],
      },
      identity,
      version,
      candidate,
      run,
    ),
  );
});
test("public package lock must join exact authenticated registry SRI and tarball", () => {
  const capsule = mockedCapsule();
  const row = {
    version,
    integrity: capsule.npmSRI.vize.integrity,
    resolved: capsule.npmSRI.vize.tarball,
  };
  const lock = (value: unknown) => ({
    packages: { "node_modules/vize": value },
  });
  assertLockedPackage(lock(row), capsule, "vize", version);
  for (const changed of [
    { ...row, version: "0.613.0" },
    { ...row, link: true },
    { ...row, integrity: "sha512-Yg==" },
    { ...row, resolved: "https://example.org/package.tgz" },
  ])
    assert.throws(() => assertLockedPackage(lock(changed), capsule, "vize", version));
  assert.throws(() =>
    assertLockedPackage(
      lock(row),
      {
        ...capsule,
        npmSRI: {
          vize: {
            ...capsule.npmSRI.vize,
            metadataOrigin: "https://example.org/metadata",
          },
        },
      },
      "vize",
      version,
    ),
  );
});
test("failed inert tooling child leaves exact raw bytes before process assertions", () =>
  fixture((root) => {
    const record = new Recorder(root);
    const result = capture(
      record,
      "inert",
      [
        process.execPath,
        "-e",
        "process.stdout.write(Buffer.from([255,0,97]));process.stderr.write('retained');process.exitCode=2",
      ],
      root,
    );
    assert.throws(() => completed(result, 0));
    assert.deepEqual(readFileSync(join(root, "inert.stdout.raw")), Buffer.from([255, 0, 97]));
    assert.equal(readFileSync(join(root, "inert.stderr.raw"), "utf8"), "retained");
    assert.equal(
      asRecord(JSON.parse(readFileSync(join(root, "inert.process.json"), "utf8"))).status,
      2,
    );
  }));

const vuePackages = () => [
  ...vueFrameworkPackages.map((name) => ({ name, version: "3.5.42" })),
  { name: "@vue/language-core", version: "3.3.13" },
];
test("Vue framework and language tooling retain their distinct exact frozen versions", () => {
  const rows = vuePackages();
  assertVuePackageVersions(rows);
  for (const [name, version] of [
    ["@vue/compiler-core", "3.3.13"],
    ["@vue/language-core", "3.5.42"],
  ])
    assert.throws(() =>
      assertVuePackageVersions(rows.map((row) => (row.name === name ? { name, version } : row))),
    );
  assert.throws(() =>
    assertVuePackageVersions([...rows, { name: "@vue/unknown", version: "3.5.42" }]),
  );
  assert.throws(() => assertVuePackageVersions([...rows, rows[0]]));
  assert.throws(() => assertVuePackageVersions(rows.slice(1)));
});
test("the original overrides and VueTSC language-core dependency lineage remain exact", () => {
  const overrides = {
    "@vue/compiler-dom": "3.5.42",
    "@vue/compiler-core": "3.5.42",
    "@vue/compiler-sfc": "3.5.42",
    "@vue/shared": "3.5.42",
  };
  const official = {
    name: "vue-tsc",
    version: "3.3.13",
    dependencies: { "@vue/language-core": "3.3.13" },
  };
  assertVueProviderLineage(overrides, official);
  assert.throws(() =>
    assertVueProviderLineage({ ...overrides, "@vue/shared": "3.3.13" }, official),
  );
  assert.throws(() => assertVueProviderLineage(overrides, { ...official, version: "3.5.42" }));
  assert.throws(() =>
    assertVueProviderLineage(overrides, {
      ...official,
      dependencies: { "@vue/language-core": "^3.3.0" },
    }),
  );
});
test("mock package-family failure retains the complete actual namespace manifests", () =>
  fixture((root) => {
    const modules = join(root, "node_modules"),
      rows = vuePackages();
    for (const row of rows) {
      const path = join(modules, row.name);
      mkdirSync(path, { recursive: true });
      writeFileSync(
        join(path, "package.json"),
        JSON.stringify({
          ...row,
          version: row.name === "@vue/language-core" ? "3.5.42" : row.version,
        }),
      );
    }
    const record = new Recorder(join(root, "proof"));
    assert.throws(() => captureVueNamespace(modules, record));
    const saved = JSON.parse(
      readFileSync(join(record.root, "vue-namespace-inventory.json"), "utf8"),
    ) as unknown;
    assert.equal(asArray(saved).length, 10);
    const language = asArray(saved)
      .map((row) => asRecord(row))
      .find((row) => row.directory === "language-core")!;
    assert.equal(
      asRecord(JSON.parse(Buffer.from(asString(language.rawBase64), "base64").toString("utf8")))
        .version,
      "3.5.42",
    );
    writeFileSync(join(modules, "@vue/language-core/package.json"), JSON.stringify(rows.at(-1)));
    for (const [directory, name] of [
      ["compiler-core", "@vue/compiler-dom"],
      ["compiler-dom", "@vue/compiler-core"],
    ])
      writeFileSync(
        join(modules, "@vue", directory, "package.json"),
        JSON.stringify({ name, version: "3.5.42" }),
      );
    assert.throws(() => captureVueNamespace(modules, record), /manifest name must match/);
  }));
