import assert from "node:assert/strict";
import { test } from "node:test";
import { createHash } from "node:crypto";
import { logicalFixture } from "./support/native-beta-logical-fixture.ts";
import { storageFixture } from "./support/native-beta-storage-fixture.ts";
import { verifyAfterLogicalRevision } from "./support/native-beta-dto/logical-revision.ts";
import { verifyBetaLogicalRoute } from "./support/native-beta-dto/logical-attached-route.ts";
import { requireCandidateStableStderr } from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  requireNoUnknownNativeWrites,
  parseNativeIo,
} from "./support/native-beta-dto/native-io.ts";
const object = (v: unknown) => v as Record<string, unknown>;
const verify = (f: ReturnType<typeof logicalFixture>) =>
  verifyAfterLogicalRevision(f.owners, f.context, f.source, f.attached);
test("current source retains the complete logical revision and observes real disk without materializing Virtual", () => {
  const f = logicalFixture(),
    proof = verify(f);
  assert.equal(proof.logicalRoot, f.root);
  assert.equal(proof.authoredRoot, f.root);
  assert.deepEqual(proof.documentUris, [
    f.attached.mainUri,
    f.attached.supportUri,
    f.attached.materializedBatch.virtualUri,
  ]);
  assert.deepEqual(proof.revisionAuthority, f.attached.materializedBatch.storageAuthority);
  assert.equal(f.attached.materializedBatch.storageAuthority.documents[2]!.disk, null);
  const stable = logicalFixture();
  stable.context.sdkVersion = "7.0.2";
  stable.context.sdkSha256 = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
  object(stable.owners[0]!.server[0]!).result = {
    serverInfo: { name: "typescript-go", version: "7.0.2" },
  };
  assert.equal(verify(stable).logicalRoot, stable.root);
});
test("logical source identity, complete map, file bytes, original config and original URI joins fail closed", () => {
  type F = ReturnType<typeof logicalFixture>;
  const mutations: Array<[string, (f: F) => void]> = [
    [
      "before",
      (f) => {
        f.context.role = "before";
      },
    ],
    [
      "source",
      (f) => {
        f.context.source = "0".repeat(40);
      },
    ],
    [
      "image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "unsupported SDK",
      (f) => {
        f.context.sdkVersion = "future";
      },
    ],
    [
      "source root",
      (f) => {
        f.context.projectRoots = [];
      },
    ],
    [
      "schema1",
      (f) => {
        f.attached.materializedBatch.storageAuthority.schema = 1;
      },
    ],
    [
      "wrong mode",
      (f) => {
        f.attached.materializedBatch.storageAuthority.mode = "physical";
      },
    ],
    [
      "logical root",
      (f) => {
        f.attached.materializedBatch.storageAuthority.logicalRoot += "/mirror";
      },
    ],
    [
      "canonical alias",
      (f) => {
        f.attached.materializedBatch.storageAuthority.canonicalLogicalRoot += "/../root";
      },
    ],
    [
      "wrong config path",
      (f) => {
        f.attached.materializedBatch.storageAuthority.config.path = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "generated config",
      (f) => {
        f.attached.materializedBatch.storageAuthority.config.text = '{"extends":"other.json"}';
      },
    ],
    [
      "wrong config digest",
      (f) => {
        f.attached.materializedBatch.storageAuthority.config.sha256 = "0".repeat(64);
      },
    ],
    [
      "source config changed",
      (f) => {
        f.attached.config += " ";
      },
    ],
    [
      "missing retained owner",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments.pop();
      },
    ],
    [
      "extra retained owner",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments.push({
          ...f.attached.materializedBatch.storageAuthority.retainedDocuments[0]!,
          uri: "file:///tmp/extra.ts",
        });
      },
    ],
    [
      "duplicate retained owner",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments.push(
          f.attached.materializedBatch.storageAuthority.retainedDocuments[0]!,
        );
      },
    ],
    [
      "retained order",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments.reverse();
      },
    ],
    [
      "retained bytes",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments[0]!.bytes++;
      },
    ],
    [
      "retained text",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments[0]!.text += " ";
      },
    ],
    [
      "retained digest",
      (f) => {
        f.attached.materializedBatch.storageAuthority.retainedDocuments[0]!.sha256 = "0".repeat(64);
      },
    ],
    [
      "missing document",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents.pop();
      },
    ],
    [
      "wrong doc text",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.text += " ";
      },
    ],
    [
      "wrong doc digest",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.sha256 = "0".repeat(64);
      },
    ],
    [
      "private mapped alias",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.mappedUri =
          "file:///tmp/vize-corsa-overlay-old/project/main.ts";
      },
    ],
    [
      "virtual disk exists",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[2]!.disk = {
          path: f.root + "/Virtual.vue.ts",
          bytes: 33,
          sha256: "0".repeat(64),
        };
      },
    ],
    [
      "disk source missing",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.disk = null;
      },
    ],
    [
      "wrong disk source hash",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.disk!.sha256 = "0".repeat(64);
      },
    ],
    [
      "independent mappedVirtual alias",
      (f) => {
        f.attached.materializedBatch.mappedUri = "file:///tmp/old/Virtual.vue.ts";
      },
    ],
    [
      "source phase changed",
      (f) => {
        f.attached.rows[1]!.supportOverlay += " ";
      },
    ],
    [
      "owner initialize alias",
      (f) => {
        object(object(f.owners[0]!.client[0]).params).rootPath = "/tmp/physical";
      },
    ],
    [
      "owner project config alias",
      (f) => {
        object(f.owners[0]!.server[1]).result = { configFilePath: "/tmp/physical/tsconfig.json" };
      },
    ],
    [
      "semantic old alias",
      (f) => {
        object(object(object(f.owners[0]!.client[2]).params).textDocument).uri =
          "file:///tmp/physical/main.ts";
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = logicalFixture();
    mutate(f);
    assert.throws(() => verify(f), name);
  }
  const inconsistent = logicalFixture(),
    doc = inconsistent.attached.materializedBatch.storageAuthority.documents[0]!;
  doc.text += " ";
  doc.bytes = Buffer.byteLength(doc.text);
  doc.sha256 = createHash("sha256").update(doc.text).digest("hex");
  const held = inconsistent.attached.materializedBatch.storageAuthority.retainedDocuments.find(
    (row) => row.uri === doc.externalUri,
  )!;
  held.text = doc.text;
  held.bytes = doc.bytes;
  held.sha256 = doc.sha256;
  assert.throws(
    () => verify(inconsistent),
    "consistent forged revision still differs from compiled source bytes",
  );
  const physical = logicalFixture();
  object(physical.attached.materializedBatch).storageAuthority =
    storageFixture().attached.materializedBatch.storageAuthority;
  assert.throws(() => verify(physical), "no observed schema selector or physical fallback");
});
test("logical beta preserves all52 ordered semantic results and admits no physical membership or semantic API", () => {
  const f = logicalFixture(),
    proof = verifyBetaLogicalRoute(f.owners, f.context, f.source, f.root, f.phases, f.attached);
  assert.equal(proof.owners[0]!.complete, true);
  assert.equal(proof.owners[0]!.semanticPackets.length, 55);
  assert.deepEqual(proof.owners[0]!.configurationMembership, []);
  assert.deepEqual(proof.sourcePhases, f.phases);
  assert.ok(f.owners[0]!.apiServer.some((row) => "error" in object(row)));
  for (const method of ["getDefaultProjectForFile", "getSemanticDiagnostics"]) {
    const bad = logicalFixture(),
      owner = bad.owners[0]!;
    owner.apiClient = [
      ...owner.apiClient,
      { jsonrpc: "2.0", id: 99, method, params: { file: bad.root + "/main.ts", snapshot: 3 } },
    ];
    owner.apiServer = [...owner.apiServer, { jsonrpc: "2.0", id: 99, result: {} }];
    assert.throws(
      () =>
        verifyBetaLogicalRoute(
          bad.owners,
          bad.context,
          bad.source,
          bad.root,
          bad.phases,
          bad.attached,
        ),
      method,
    );
  }
  const wrong = logicalFixture();
  const replies = wrong.owners[0]!.server.filter(
    (row) => object(row).id === 3 || object(row).id === 16,
  );
  const a = object(replies[0]).result;
  object(replies[0]).result = object(replies[1]).result;
  object(replies[1]).result = a;
  assert.throws(() =>
    verifyBetaLogicalRoute(
      wrong.owners,
      wrong.context,
      wrong.source,
      wrong.root,
      wrong.phases,
      wrong.attached,
    ),
  );
});
test("logical evidence does not bypass candidate zero-stderr or unknown-write refusal", () => {
  verify(logicalFixture());
  assert.throws(() =>
    requireCandidateStableStderr(Buffer.from("1 paths ineligible for watching\n")),
  );
  const row = parseNativeIo('1793460256.100000 write(2, "\\x78", 1) = ?');
  assert.ok(row && row.outcome === "unknown");
  assert.throws(() =>
    requireNoUnknownNativeWrites([{ ...row, tid: 1, file: "syscalls.1", line: 1 }]),
  );
});
