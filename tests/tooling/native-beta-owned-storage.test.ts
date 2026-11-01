import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  verifyAfterAttachedStorage,
  verifyObserverDependencies,
  storageObserverPath,
  storageObserverSha256,
  commonObserverSha256,
  assertImageServerInfo,
} from "./support/native-beta-dto/beta-attached-route.ts";
import { requireCandidateStableStderr } from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  parseNativeIo,
  requireNoUnknownNativeWrites,
} from "./support/native-beta-dto/native-io.ts";
import { storageFixture } from "./support/native-beta-storage-fixture.ts";

function fixture() {
  const storage = storageFixture(),
    source = "b".repeat(40);
  const root = storage.root;
  return {
    ...storage,
    source,
    owners: [
      {
        nativeOwner: 123,
        client: [
          {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize",
            params: { rootPath: root, rootUri: `file://${root}` },
          },
          {
            jsonrpc: "2.0",
            id: 2,
            method: "custom/projectInfo",
            params: { textDocument: { uri: `file://${root}/main.ts` } },
          },
          {
            jsonrpc: "2.0",
            id: 3,
            method: "textDocument/diagnostic",
            params: { textDocument: { uri: storage.attached.materializedBatch.mappedUri } },
          },
        ],
        server: [
          {
            jsonrpc: "2.0",
            id: 1,
            result: { serverInfo: { name: "typescript", version: "7.1.0-beta" } },
          },
          { jsonrpc: "2.0", id: 2, result: { configFilePath: root + "/tsconfig.json" } },
          { jsonrpc: "2.0", id: 3, result: { kind: "full", items: [] } },
        ],
        apiClient: [],
        apiServer: [],
      },
    ],
    context: {
      source,
      role: "after",
      sdkVersion: "7.1.0-beta",
      sdkSha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
      sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
      projectRoots: [storage.attached.root],
    },
  };
}
const verify = (value: ReturnType<typeof fixture>) =>
  verifyAfterAttachedStorage(value.owners, value.context, value.source, value.attached);

test("historical physical contract retains its source getter, exact files and generated config", () => {
  const value = fixture(),
    proof = verify(value);
  assert.equal(proof.materializedRoot, value.root);
  assert.equal(proof.authoredRoot, value.attached.root);
  assert.equal(proof.documentUris.length, 6);
  assert.deepEqual(proof.storageAuthority, value.attached.materializedBatch.storageAuthority);
  const stable = fixture();
  stable.context.sdkVersion = "7.0.2";
  stable.context.sdkSha256 = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
  stable.owners[0]!.server[0]!.result.serverInfo!.version = "7.0.2";
  stable.owners[0]!.server[0]!.result.serverInfo!.name = "typescript-go";
  assert.equal(verify(stable).materializedRoot, stable.root);
});

test("server identity is one exact image/version/name tuple with no fallback labels", () => {
  const beta = fixture().context;
  const stable = {
    sdkVersion: "7.0.2",
    sdkSha256: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  };
  assertImageServerInfo({ name: "typescript", version: "7.1.0-beta" }, beta);
  assertImageServerInfo({ name: "typescript-go", version: "7.0.2" }, stable);
  for (const value of [
    { name: "typescript-go", version: "7.1.0-beta" },
    { name: "typescript", version: "7.0.2" },
    { name: "future", version: "7.1.0-beta" },
    { name: "typescript", version: "7.1.0-beta", extra: true },
  ])
    assert.throws(() => assertImageServerInfo(value, beta));
  assert.throws(() => assertImageServerInfo({ name: "typescript", version: "7.0.2" }, stable));
  for (const sdkSha256 of [stable.sdkSha256, "0".repeat(64)])
    assert.throws(() =>
      assertImageServerInfo({ name: "typescript", version: "7.1.0-beta" }, { ...beta, sdkSha256 }),
    );
  assert.throws(() =>
    assertImageServerInfo(
      { name: "typescript", version: "future" },
      { ...beta, sdkVersion: "future" },
    ),
  );
});

test("wrong source, role, owner, path, URI, physical contents or config cannot grant authority", () => {
  type Fixture = ReturnType<typeof fixture>;
  const mutations: Array<[string, (value: Fixture) => void]> = [
    [
      "before role",
      (f) => {
        f.context.role = "before";
      },
    ],
    [
      "foreign source",
      (f) => {
        f.context.source = "a".repeat(40);
      },
    ],
    [
      "unsupported SDK",
      (f) => {
        f.context.sdkVersion = "future";
      },
    ],
    [
      "wrong native image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "missing source project",
      (f) => {
        f.context.projectRoots = [];
      },
    ],
    [
      "foreign client root",
      (f) => {
        f.attached.materializedBatch.storageAuthority.clientProjectRoot = "/tmp/foreign";
      },
    ],
    [
      "noncanonical mirror",
      (f) => {
        f.attached.materializedBatch.storageAuthority.materializedRoot += "/../project";
      },
    ],
    [
      "canonical root mismatch",
      (f) => {
        f.attached.materializedBatch.storageAuthority.canonicalMaterializedRoot = "/tmp/foreign";
      },
    ],
    [
      "symlink root",
      (f) => {
        f.attached.materializedBatch.storageAuthority.rootIsSymlink = true;
      },
    ],
    [
      "nondirectory root",
      (f) => {
        f.attached.materializedBatch.storageAuthority.rootIsDirectory = false;
      },
    ],
    [
      "old nested overlay is not new private source root",
      (f) => {
        f.attached.materializedBatch.storageAuthority.materializedRoot =
          f.attached.root + "/node_modules/.vize/corsa-overlay";
      },
    ],
    [
      "wrong main URI",
      (f) => {
        f.attached.mainUri = "file:///tmp/foreign/main.ts";
      },
    ],
    [
      "mismatched independent virtual mapping",
      (f) => {
        f.attached.materializedBatch.mappedUri = "file:///tmp/foreign/Virtual.vue.ts";
      },
    ],
    [
      "wrong virtual source",
      (f) => {
        f.attached.materializedBatch.source += " ";
      },
    ],
    [
      "wrong source-phase text",
      (f) => {
        f.attached.rows[0]!.mainText += " ";
      },
    ],
    [
      "missing physical member",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents.pop();
      },
    ],
    [
      "wrong physical file",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.relativePath = "support.ts";
      },
    ],
    [
      "foreign physical URI",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.mappedUri =
          "file:///tmp/foreign/main.ts";
      },
    ],
    [
      "wrong physical byte count",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.bytes++;
      },
    ],
    [
      "wrong physical digest",
      (f) => {
        f.attached.materializedBatch.storageAuthority.documents[0]!.sha256 = "0".repeat(64);
      },
    ],
    [
      "wrong config path",
      (f) => {
        f.attached.materializedBatch.storageAuthority.config.path = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "wrong config digest",
      (f) => {
        f.attached.materializedBatch.storageAuthority.config.sha256 = "0".repeat(64);
      },
    ],
    [
      "wrong config structure with honest hash",
      (f) => {
        const config = f.attached.materializedBatch.storageAuthority.config;
        config.text = JSON.stringify({
          extends: "/tmp/foreign/tsconfig.json",
          compilerOptions: { noEmit: true },
          include: ["**/*"],
          exclude: [],
        });
        config.bytes = Buffer.byteLength(config.text);
        config.sha256 = createHash("sha256").update(config.text).digest("hex");
      },
    ],
    [
      "wrong native owner root",
      (f) => {
        f.owners[0]!.client[0]!.params.rootPath = "/tmp/foreign";
      },
    ],
    [
      "wrong native root URI",
      (f) => {
        f.owners[0]!.client[0]!.params.rootUri = "file:///tmp/foreign";
      },
    ],
    [
      "wrong actual config reply",
      (f) => {
        f.owners[0]!.server[1]!.result.configFilePath = "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "unregistered source-relative URI",
      (f) => {
        f.owners[0]!.client[2]!.params.textDocument!.uri = `file://${f.root}/future.ts`;
      },
    ],
    [
      "sibling lookup URI",
      (f) => {
        f.owners[0]!.client[2]!.params.textDocument!.uri = `file://${f.root}/../node_modules/escape.ts`;
      },
    ],
    [
      "no owner",
      (f) => {
        f.owners.pop();
      },
    ],
    [
      "invalid owner",
      (f) => {
        f.owners[0]!.nativeOwner = 0;
      },
    ],
    [
      "duplicate owner",
      (f) => {
        f.owners.push(structuredClone(f.owners[0]!));
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const value = fixture();
    mutate(value);
    assert.throws(() => verify(value), name);
  }
});

test("compiled observer helper identity is exact and cannot replace source production", () => {
  const expected = [{ path: storageObserverPath, sha256: storageObserverSha256 }];
  const harness = {
    path: storageObserverPath.replace("/native_beta/storage_authority.rs", "/native_beta.rs"),
    sha256: commonObserverSha256,
  };
  verifyObserverDependencies(expected, structuredClone(expected), commonObserverSha256, harness);
  for (const value of [
    [],
    [...expected, ...expected],
    [{ ...expected[0], path: "production.rs" }],
    [{ ...expected[0], sha256: "0".repeat(64) }],
    [{ ...expected[0], extra: true }],
  ])
    assert.throws(() => verifyObserverDependencies(value, value, commonObserverSha256, harness));
  assert.throws(() => verifyObserverDependencies(expected, [], commonObserverSha256, harness));
  assert.throws(() => verifyObserverDependencies(expected, expected, "0".repeat(64), harness));
  assert.throws(
    () =>
      verifyObserverDependencies(
        expected,
        expected,
        "26eb21864007e6011c69d0a95e1be173fd586605a87400bdc3aacecb5cb31dc6",
        harness,
      ),
    "historical observer bytes cannot stand in for the current compiled observer",
  );
  assert.throws(() =>
    verifyObserverDependencies(expected, expected, commonObserverSha256, {
      ...harness,
      sha256: "0".repeat(64),
    }),
  );
  assert.throws(() =>
    verifyObserverDependencies(expected, expected, commonObserverSha256, {
      ...harness,
      path: "production.rs",
    }),
  );
});

test("new root authority cannot bypass current zero-stderr or unknown-write guards", () => {
  verify(fixture());
  requireCandidateStableStderr(Buffer.alloc(0));
  assert.throws(() => requireCandidateStableStderr(Buffer.from("context canceled\n")));
  const write = parseNativeIo('1.000001 write(2, "\\x78", 1) = ?');
  assert.equal(write?.outcome, "unknown");
  if (!write || write.outcome !== "unknown") throw new Error("unknown write control missing");
  assert.throws(() =>
    requireNoUnknownNativeWrites([{ ...write, tid: 123, file: "synthetic", line: 1 }]),
  );
});
