import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { readTarGz } from "../../../tools/support/archive/tar.ts";
import { capturePublicPayload } from "../../../tools/support/release/human_reports_11252_11253_public/capture-payload.ts";
import { prepareInstalledGraph } from "../../../tools/support/release/human_reports_11252_11253_public/package-graph.ts";
import { retainCaptureBeforeQualification } from "../../../tools/support/release/human_reports_11252_11253_public/pipeline.ts";
import { validateRecordedInstall } from "../../../tools/support/release/human_reports_11252_11253_public/installation-record.ts";
import type {
  Bundle,
  CaptureInstallReceipt,
  Fixture,
  InstallReceipt,
  Recipe,
  ReleaseAuthority,
  Request,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
import { pkg } from "./human-reports-11252-11253-bin.fixture.ts";

// These are inert synthetic filesystem/egress fixtures, never installations,
// native images, product executions or authority for the original public roles.
function materialize() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "human-capture-payload-")));
  const packages = ["first", "second"].map((name) =>
    pkg(
      name,
      "1.0.0",
      join(root, "node_modules", name),
      { name, version: "1.0.0" },
      { "index.js": Buffer.from("// inert synthetic published payload\n") },
    ),
  );
  for (const row of packages) {
    assert(row.archive);
    for (const member of readTarGz(row.archive)) {
      const target = join(row.directory, member.name.slice(8));
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, member.data);
    }
  }
  const lock = Buffer.from(
    "lockfileVersion: '9.0'\npackages:\n" +
      packages
        .map((p) => `  ${p.name}@${p.version}:\n    resolution: {integrity: ${p.integrity}}\n`)
        .join(""),
  );
  return { root, packages, lock, close: () => rmSync(root, { recursive: true, force: true }) };
}
test("complete public-payload graph survives an early extra until the unchanged final file-set verdict", async () => {
  const f = materialize(),
    events: Array<{ kind: string; value: unknown }> = [];
  try {
    writeFileSync(join(f.packages[0]!.directory, "unknown-extra.txt"), "not a public payload\n");
    const graph = await prepareInstalledGraph(
      f.root,
      [
        {
          dependencies: Object.fromEntries(
            f.packages.map((p) => [
              p.name,
              { version: p.version, path: p.directory, resolved: p.resolved },
            ]),
          ),
        },
      ],
      f.lock,
      { os: "linux", cpu: "x64", libc: "glibc", libcVersion: "synthetic", libcSource: "synthetic" },
      new Map(),
      new Map(),
      (kind, value) => events.push({ kind, value }),
      {
        archive: async (name) => {
          const row = f.packages.find((p) => p.name === name);
          assert(row?.archive);
          return {
            name,
            version: row.version,
            resolved: row.resolved,
            integrity: row.integrity,
            body: row.archive,
            url: "https://registry.npmjs.org/synthetic",
            status: 200,
            date: null,
            rawBase64: "e30=",
            rawSha256: "0".repeat(64),
            metadata: {},
          };
        },
        metadata: async () => assert.fail("no optional metadata or external transport"),
      },
    );
    assert.equal(graph.packages.length, 2);
    assert(
      graph.packages.every(
        (p) => p.publicPayloadBytesEqualArchive && p.completeInstalledFileSetQualified === false,
      ),
    );
    assert(graph.packages.every((p) => !("allInstalledBytesEqualArchive" in p)));
    assert(events.some((e) => e.kind === "complete-public-payload-graph-before-command"));
    assert.throws(() => graph.qualify(), /public archive and installed package file sets differ/u);
    assert.equal(
      graph.packages.length,
      2,
      "the full retained prefix is not discarded by qualification",
    );
    assert(!JSON.stringify(events).includes('"type":"Buffer"'));
  } finally {
    f.close();
  }
});
test("a modified mandatory public payload refuses without exporting the unknown body", () => {
  const f = materialize(),
    events: unknown[] = [],
    secret = "SYNTHETIC_UNEXPECTED_TOKEN_894";
  try {
    writeFileSync(join(f.packages[0]!.directory, "index.js"), secret);
    assert.throws(
      () => capturePublicPayload(f.packages[0]!, (kind, value) => events.push({ kind, value })),
      /public payload mismatch/u,
    );
    const encoded = JSON.stringify(events);
    assert(!encoded.includes(secret));
    assert(!encoded.includes(Buffer.from(secret).toString("base64")));
    assert(encoded.includes('"complete":false'));
    assert(encoded.includes("actualSha256"));
  } finally {
    f.close();
  }
});
function pending(): CaptureInstallReceipt {
  // Deliberately only the fields read by the egress boundary. This is not a
  // fake successful provider/native/installation qualification fixture.
  return {
    schema: "vize.humanReports11252_11253.capturePrerequisites.v1",
    kind: "original",
    packages: [],
    mandatoryExecutableChainsAuthenticated: true,
    completeInstalledFileSetQualified: false,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
  } as unknown as CaptureInstallReceipt;
}
function capture(install = pending()) {
  return {
    request: {} as Request,
    release: {} as ReleaseAuthority,
    installs: [install],
    observations: [],
    companions: [],
    controller: {} as Bundle["controller"],
  };
}
test("a pending capture is type-distinct and the strict archived install refuses its schema first", () => {
  const value = pending();
  // @ts-expect-error a pending capture is not a qualified installation.
  const strict: InstallReceipt = value;
  assert.throws(
    () =>
      validateRecordedInstall(
        strict,
        {} as Fixture,
        {} as Recipe,
        {} as ReleaseAuthority,
        {} as Request["reviewedController"],
      ),
    /install\.v1/u,
  );
});
test("full raw capture is saved with literal false flags before a real final qualification refusal", () => {
  const order: string[] = [],
    raw: unknown[] = [],
    failure = new Error("strict full-set refusal");
  assert.throws(
    () =>
      retainCaptureBeforeQualification(
        capture(),
        (value) => {
          order.push("saved");
          raw.push(value);
        },
        () => {
          order.push("qualify");
          throw failure;
        },
      ),
    (error) => error === failure,
  );
  assert.deepEqual(order, ["saved", "qualify"]);
  const row = raw[0] as Record<string, unknown>;
  for (const key of [
    "completeInstalledFileSetQualified",
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
    "outcomeAssessed",
  ])
    assert.equal(row[key], false);
});
test("failed raw persistence prevents qualification; pending or empty callback results cannot cross strict egress", () => {
  let called = false;
  assert.throws(
    () =>
      retainCaptureBeforeQualification(
        capture(),
        () => {
          throw new Error("write failed");
        },
        () => {
          called = true;
          return [];
        },
      ),
    /write failed/u,
  );
  assert.equal(called, false);
  assert.throws(() =>
    retainCaptureBeforeQualification(
      capture(),
      () => {},
      () => [],
    ),
  );
  const untrusted = pending() as unknown as InstallReceipt;
  assert.throws(
    () =>
      retainCaptureBeforeQualification(
        capture(),
        () => {},
        () => [untrusted],
      ),
    /pending capture/u,
  );
  const relabeled = {
    ...pending(),
    schema: "vize.humanReports11252_11253.install.v1",
  } as unknown as InstallReceipt;
  assert.throws(
    () =>
      retainCaptureBeforeQualification(
        capture(),
        () => {},
        () => [relabeled],
      ),
    /pending verdict/u,
  );
});
