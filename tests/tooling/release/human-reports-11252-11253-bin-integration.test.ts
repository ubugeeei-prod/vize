import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { readTarGz } from "../../../tools/support/archive/tar.ts";
import { fileImage } from "../../../tools/support/release/computed_event_custody.ts";
import { installedGraph } from "../../../tools/support/release/human_reports_11252_11253_public/package-graph.ts";
import {
  bindBinPolicy,
  binPolicyPath,
} from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-policy.ts";
import {
  validateBinRecord,
  recheckBinPackage,
} from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-record.ts";
import type {
  InstallReceipt,
  Request,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
import {
  gitBlob,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import {
  fixture,
  pkg,
  configurationFixture,
  expectedNode,
} from "./human-reports-11252-11253-bin.fixture.ts";

// Synthetic on-disk packages only: no install, manager, native or compiler run.
async function setup() {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "human-source-bin-"))),
    root = join(base, "consumer-original");
  const facts = fixture();
  const relocate = (value: string) => value.replaceAll(facts.root, root);
  const packages = [facts.parent, ...facts.dependencyOwners].map((row) => ({
    ...row,
    directory: relocate(row.directory),
  }));
  const parent = packages[0]!,
    deps = packages.slice(1);
  const carrier = pkg(
    "carrier",
    "1.0.0",
    join(root, "node_modules/carrier"),
    { name: "carrier", version: "1.0.0", dependencies: { vue: "3.5.44" } },
    {},
  );
  packages.push(carrier);
  facts.lock = Buffer.concat([
    facts.lock,
    Buffer.from(`\n  carrier@1.0.0:\n    resolution: {integrity: ${carrier.integrity}}\n`),
  ]);
  for (const row of packages) {
    assert(row.archive);
    for (const member of readTarGz(row.archive)) {
      const target = join(row.directory, member.name.slice("package/".length));
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, member.data);
    }
  }
  for (const row of facts.inventory.filter(
    (row) => row.path.includes("/.bin/") && row.kind === "file",
  )) {
    const target = relocate(row.path);
    assert(row.body);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, row.body.toString().replaceAll(facts.root, root));
  }
  const typescript = deps.find((row) => row.name === "typescript");
  assert(typescript);
  for (const name of ["tsc", "tsserver"]) {
    const target = join(typescript.directory, "node_modules/.bin", name);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, expectedNode(name, true, true));
  }
  writeFileSync(join(root, "package.json"), facts.settings.rootPackage);
  writeFileSync(join(root, "pnpm-lock.yaml"), facts.lock);
  const managerPath = join(base, "pnpm-launcher"),
    managerRaw = Buffer.from("#!/bin/sh\n# synthetic unexecuted launcher\n");
  writeFileSync(managerPath, managerRaw);
  const nodePath = join(base, "node-unexecuted-fixture");
  writeFileSync(nodePath, "synthetic node image\n");
  const node = fileImage(nodePath),
    pnpm = fileImage(managerPath);
  const user = join(base, "user.npmrc"),
    global = join(base, "global.npmrc");
  writeFileSync(user, "");
  writeFileSync(global, "");
  const contextPath = join(base, "context.json");
  writeFileSync(
    contextPath,
    JSON.stringify({
      root,
      node,
      pnpm,
      originalArgv: ["install"],
      registryEnvironment: { npm_config_userconfig: user, npm_config_globalconfig: global },
    }),
  );
  const policyRaw = facts.inputPolicyRaw!;
  const reviewed: Request["reviewedController"] = {
    head: "1".repeat(40),
    files: [
      {
        path: binPolicyPath,
        bytes: policyRaw.length,
        sha256: sha256(policyRaw),
        gitBlob: gitBlob(policyRaw),
      },
    ],
  };
  const policy = bindBinPolicy(policyRaw, reviewed);
  const leaf = (row: typeof parent) => ({
    version: row.version,
    resolved: row.resolved,
    path: row.directory,
  });
  const listing = [
    {
      dependencies: {
        vue: { ...leaf(parent), deduped: true },
        carrier: {
          ...leaf(carrier),
          dependencies: {
            vue: {
              ...leaf(parent),
              dependencies: Object.fromEntries(deps.map((row) => [row.name, leaf(row)])),
            },
          },
        },
      },
    },
  ];
  const archives = new Map<string, Buffer>(),
    events: Array<{ kind: string; value: unknown }> = [];
  const graph = await installedGraph(
    root,
    listing,
    facts.lock,
    { os: "linux", cpu: "x64", libc: "glibc", libcVersion: "synthetic", libcSource: "synthetic" },
    new Map(),
    archives,
    (kind, value) => events.push({ kind, value }),
    {
      archive: async (name, version) => {
        const row = packages.find((row) => row.name === name && row.version === version);
        assert(row?.archive);
        return {
          name,
          version,
          url: "https://registry.npmjs.org/synthetic",
          status: 200,
          date: null,
          rawBase64: "e30=",
          rawSha256: sha256("{}"),
          metadata: { name, version },
          resolved: row.resolved,
          integrity: row.integrity,
          body: row.archive,
        };
      },
      metadata: async () => {
        assert.fail("no absent candidate or native registry query expected");
      },
    },
    {
      path: contextPath,
      managerVersion: "12.16.1",
      conformance: {
        role: "imports-vue-ts-11253",
        policy,
        manager: {
          node,
          pnpm,
          reportedVersion: "12.16.1",
          contextPath,
          configuration: configurationFixture(root, "linux", facts.settings.rootPackage),
          version: {
            pid: 123,
            status: 0,
            signal: null,
            error: null,
            stdoutBase64: Buffer.from("12.16.1\n").toString("base64"),
            stderrBase64: "",
          },
        },
      },
    },
  );
  const positive = graph.packages.find((row) => row.allInstalledBytesEqualArchive === false);
  assert(positive && positive.allInstalledBytesEqualArchive === false);
  // This partial type is deliberately only the exact record-check input;
  // it is not an installation/native/provider qualification fixture.
  const install = {
    root,
    packages: graph.packages,
    pnpm,
    pnpmVersion: "12.16.1",
    platform: "linux",
    arch: "x64",
    lockRawBase64: facts.lock.toString("base64"),
    binPolicy: policy,
    declaredOptionalNotInstalled: [],
  } as unknown as InstallReceipt;
  return { base, root, parent, graph, positive, install, reviewed, events };
}
test("first deduped parent does not hide the later complete six-owner graph; positive receipt stays distinct", async () => {
  const s = await setup();
  try {
    assert.equal(s.graph.packages.length, 8);
    assert.equal(s.positive.binContext.dependencyOwners.length, 6);
    assert.equal(s.positive.sourceConformance.publicPayloadCount, 37);
    assert.equal(s.positive.allInstalledBytesEqualArchive, false);
    assert.equal(s.positive.sourceConformance.sourceConformantBinCount, 2);
    assert.equal(s.positive.sourceConformance.actualWriterRuntimeAttribution, false);
    assert.equal(s.positive.binContext.writerImageOwner, "UNKNOWN");
    validateBinRecord(s.positive, s.install, s.reviewed);
    recheckBinPackage(s.positive, s.install);
    assert.equal(s.graph.absent.length, 0);
    assert(
      !JSON.stringify(s.events).includes('"type":"Buffer"'),
      "archive/native bodies are excluded from evidence",
    );
  } finally {
    rmSync(s.base, { recursive: true, force: true });
  }
});
test("archived positive records use recorded paths only and preserve independent controller policy authority", async () => {
  const s = await setup();
  try {
    renameSync(s.root, s.root + "-parked");
    validateBinRecord(s.positive, s.install, s.reviewed);
    const bad = structuredClone(s.reviewed);
    bad.files[0]!.gitBlob = "0".repeat(40);
    assert.throws(() => validateBinRecord(s.positive, s.install, bad));
    const forged = structuredClone(s.install.binPolicy);
    const raw = Buffer.from('{"self":"observed"}\n');
    forged.rawBase64 = raw.toString("base64");
    forged.gitBlob = gitBlob(raw);
    forged.sha256 = sha256(raw);
    forged.bytes = raw.length;
    assert.throws(() => bindBinPolicy(raw, s.reviewed));
    assert.notDeepEqual(forged, s.install.binPolicy);
  } finally {
    rmSync(s.base, { recursive: true, force: true });
  }
});
test("whole live recheck refuses payload/shim changes, new files, links and unknown empty directories", async () => {
  for (const mutation of ["payload", "shim", "file", "directory", "link"] as const) {
    const s = await setup();
    try {
      if (mutation === "payload")
        writeFileSync(join(s.parent.directory, "dist/inert-0.txt"), "changed");
      if (mutation === "shim")
        writeFileSync(join(s.parent.directory, "node_modules/.bin/tsc"), "changed");
      if (mutation === "file") writeFileSync(join(s.parent.directory, "unexpected"), "unknown");
      if (mutation === "directory") mkdirSync(join(s.parent.directory, "unexpected-empty"));
      if (mutation === "link")
        symlinkSync(join(s.root, "package.json"), join(s.parent.directory, "unexpected-link"));
      assert.throws(() => recheckBinPackage(s.positive, s.install));
    } finally {
      rmSync(s.base, { recursive: true, force: true });
    }
  }
});
test("record mutations cannot replace a full declaration census, target binding, policy or source-derived body", async () => {
  const s = await setup();
  try {
    for (const mutate of [
      (p: typeof s.positive) => {
        p.binContext.dependencyOwners.pop();
      },
      (p: typeof s.positive) => {
        p.binContext.targets[0]!.rawBase64 = Buffer.from("other").toString("base64");
      },
      (p: typeof s.positive) => {
        p.sourceConformance.files.find((row) => row.bucket === "SOURCE_CONFORMANT_BIN")!.rawBase64 =
          Buffer.from("other").toString("base64");
      },
      (p: typeof s.positive) => {
        p.sourceConformance.actualInventoryEntries++;
      },
      (p: typeof s.positive) => {
        p.binContext.graph[0]!.directory = "/foreign";
      },
    ]) {
      const bad = structuredClone(s.positive);
      mutate(bad);
      assert.throws(() => validateBinRecord(bad, s.install, s.reviewed));
    }
    assert.equal(
      createHash("sha256")
        .update(readFileSync(join(s.parent.directory, "package.json")))
        .digest("hex"),
      s.positive.files.find((row) => row.path.endsWith("/package.json"))!.sha256,
    );
  } finally {
    rmSync(s.base, { recursive: true, force: true });
  }
});
