import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { isDeepStrictEqual } from "node:util";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import { decodeFrames } from "./lsp-wire.ts";
import type { FixtureData, LspRow } from "./lsp-types.ts";
import {
  NATIVE_CAPACITY_UNSUPPORTED_REASON,
  type NativeCapacitySourceSession,
} from "./lsp-native-capacity-contract.ts";

export const LOCAL_RESOLVE_SESSION_ID = "lsp/source-session/local-completion-resolve-10592";
export const LOCAL_RESOLVE_SESSION_DIRECTORY =
  "tests/_fixtures/differential/lsp/local-completion-resolve-10592";
export const SOURCE_SESSION_UNSUPPORTED_REASON =
  "The initial-document adapter cannot execute this pinned multi-root configuration transition; the dedicated source executor supplies its own full wire and source custody.";

type LocalResolveSourceSession = {
  kind: "local-completion-resolve-v1";
  state: "source-inspected";
  executor: "tests/tooling/lsp-local-completion-resolve.test.ts";
  plannedClientRpcCount: 23;
  directWrapperRpcCount: 22;
  workspaceFolders: ["a", "b"];
  documentPaths: ["a/src/pages/products/[id].vue", "b/src/pages/products/[id].vue"];
  transition: { target: "a/vite.config.mjs"; input: "transition/a-vite.config.mjs"; eventType: 2 };
  localItem: "protocol/local-item.json";
  capabilities: "protocol/initialize-capabilities.json";
};

export type LspSourceSession = LocalResolveSourceSession | NativeCapacitySourceSession;

export function sourceSessionUnsupportedLegacy(session?: LspSourceSession): LspRow["legacy"] {
  assert.ok(
    session === undefined ||
      session.kind === "local-completion-resolve-v1" ||
      session.kind === "native-diagnostic-capacity-v1",
  );
  return {
    state: "unsupported",
    verdict: "unsupported-transition",
    comparator: "source-inspected-session",
    reason:
      session?.kind === "native-diagnostic-capacity-v1"
        ? NATIVE_CAPACITY_UNSUPPORTED_REASON
        : SOURCE_SESSION_UNSUPPORTED_REASON,
  };
}

export function validateLocalResolveSourceSession(
  data: FixtureData,
  files: ReadonlyMap<string, Buffer>,
  plan: unknown,
) {
  assert.equal(data.id, LOCAL_RESOLVE_SESSION_ID);
  assert.equal(data.entry, "a/src/pages/products/[id].vue");
  assert.equal(data.method, "source-session/local-completion-resolve");
  assert.equal(data.openDocument, false);
  assert.deepEqual(data.initializationOptions, { typecheck: false, lint: false });
  const spec: LspSourceSession = {
    kind: "local-completion-resolve-v1",
    state: "source-inspected",
    executor: "tests/tooling/lsp-local-completion-resolve.test.ts",
    plannedClientRpcCount: 23,
    directWrapperRpcCount: 22,
    workspaceFolders: ["a", "b"],
    documentPaths: ["a/src/pages/products/[id].vue", "b/src/pages/products/[id].vue"],
    transition: {
      target: "a/vite.config.mjs",
      input: "transition/a-vite.config.mjs",
      eventType: 2,
    },
    localItem: "protocol/local-item.json",
    capabilities: "protocol/initialize-capabilities.json",
  };
  assert.deepEqual(data.sourceSession, spec, "only the closed source-session profile is admitted");
  assert.deepEqual(plan, {
    schema: "vize.lsp.local-completion-resolve-session",
    version: 1,
    plannedClientRpcCount: 23,
    directWrapperRpcCount: 22,
    genericAdapter: "unsupported-transition",
    executor: spec.executor,
    before: {
      version: "0.562.0",
      failureRequestId: 12,
      error: { code: -32801, message: "Content modified" },
      observedPrefixRequests: 12,
      observedCleanupRequests: 1,
    },
  });
  const read = (name: string) => {
    const value = files.get(name);
    assert.ok(value, `pinned input ${name}`);
    return value;
  };
  const json = (name: string) => JSON.parse(read(name).toString("utf8"));
  assert.deepEqual(
    [...files.keys()],
    [
      ...spec.workspaceFolders.flatMap((folder) =>
        ["package.json", "vite.config.mjs", "tsconfig.json", "src/pages/products/[id].vue"].map(
          (name) => `${folder}/${name}`,
        ),
      ),
      spec.transition.input,
      spec.localItem,
      spec.capabilities,
      "custody/public-before-observation.json.gz",
      "custody/public-before-wire.jsonl.gz",
      "custody/public-before-identity.json",
      "custody/source-law.rs",
    ],
    "the closed session retains its complete actual inputs and custody",
  );
  const localItem = json(spec.localItem);
  assert.deepEqual(localItem, {
    detail: "Vue Router file route param: string",
    kind: 10,
    label: "id",
    sortText: "0",
  });
  const before = JSON.parse(
    gunzipSync(read("custody/public-before-observation.json.gz")).toString("utf8"),
  ) as Array<Record<string, unknown>>;
  const authored = before.find((entry) => entry.kind === "authored-inputs");
  assert.ok(authored && Array.isArray(authored.files));
  const initialFiles = authored.files as Array<{ path: string; source: string }>;
  assert.equal(initialFiles.length, 8);
  for (const entry of initialFiles) assert.equal(read(entry.path).toString("utf8"), entry.source);
  assert.deepEqual(
    initialFiles.map(({ path: name }) => name),
    spec.workspaceFolders.flatMap((folder) =>
      ["package.json", "vite.config.mjs", "tsconfig.json", "src/pages/products/[id].vue"].map(
        (name) => `${folder}/${name}`,
      ),
    ),
  );
  const changed = before.find((entry) => entry.kind === "configuration-write");
  assert.ok(changed);
  assert.equal(changed.path, spec.transition.target);
  assert.equal(changed.source, read(spec.transition.input).toString("utf8"));
  assert.ok(
    before.some(
      (entry) =>
        entry.kind === "error" &&
        entry.method === "completionItem/resolve" &&
        isDeepStrictEqual(entry.params, localItem),
    ),
  );

  const records = gunzipSync(read("custody/public-before-wire.jsonl.gz"))
    .toString("utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const owners = records.filter((entry) => entry.kind === "spawn" && entry.argv[0] === "lsp");
  assert.equal(owners.length, 1);
  const messages = (direction: string) =>
    decodeFrames(
      Buffer.concat(
        records
          .filter(
            (entry) =>
              entry.kind === "chunk" &&
              entry.pid === owners[0].pid &&
              entry.direction === direction,
          )
          .map((entry) => Buffer.from(entry.chunkBase64, "base64")),
      ),
    ).messages;
  const client = messages("client");
  const server = messages("server");
  const initialized = client.find((entry) => entry.method === "initialize")!;
  assert.deepEqual(initialized.params?.capabilities, json(spec.capabilities));
  assert.deepEqual(initialized.params?.initializationOptions, data.initializationOptions);
  const folders = initialized.params?.workspaceFolders as Array<{ uri: string; name: string }>;
  assert.deepEqual(
    folders.map(({ name }) => name),
    spec.workspaceFolders,
  );
  const firstFolder = folders[0];
  assert.ok(firstFolder);
  const opened = client.filter((entry) => entry.method === "textDocument/didOpen");
  assert.equal(opened.length, 2);
  for (const [index, packet] of opened.entries()) {
    const documentPath: string | undefined = spec.documentPaths[index];
    const folder: { uri: string; name: string } | undefined = folders[index];
    assert.ok(documentPath && folder);
    const relative = documentPath.slice(2);
    assert.deepEqual(packet.params, {
      textDocument: {
        uri: new URL(relative.split("/").map(encodeURIComponent).join("/"), `${folder.uri}/`).href,
        languageId: "vue",
        version: 1,
        text: read(documentPath).toString("utf8"),
      },
    });
  }
  assert.deepEqual(
    client.find((entry) => entry.method === "workspace/didChangeWatchedFiles")?.params,
    {
      changes: [
        {
          uri: new URL("vite.config.mjs", `${firstFolder.uri}/`).href,
          type: spec.transition.eventType,
        },
      ],
    },
  );
  const requests = client.filter((entry) => typeof entry.id === "number" && entry.method);
  assert.equal(requests.length, 13, "the observed failed prefix includes its actual cleanup only");
  assert.deepEqual(
    requests.map(({ id }) => id),
    Array.from({ length: 13 }, (_, index) => index + 1),
  );
  assert.deepEqual(
    requests.find(({ id }) => id === 12),
    { jsonrpc: "2.0", id: 12, method: "completionItem/resolve", params: localItem },
  );
  assert.deepEqual(
    server.find(({ id }) => id === 12),
    { jsonrpc: "2.0", id: 12, error: { code: -32801, message: "Content modified" } },
  );
  assert.equal(requests.at(-1)?.method, "shutdown");
  const identity = json("custody/public-before-identity.json");
  assert.equal(identity.schema, "vize.lsp.public-local-resolve-before");
  assert.equal(identity.versionObserved, "0.562.0");
  assert.deepEqual(
    identity.packages.map(({ version }: { version: string }) => version),
    ["0.562.0", "0.562.0", "0.562.0"],
  );
  assert.equal(
    owners[0].binarySha256,
    identity.members.find(
      ({ path: name }: { path: string }) => name === "node_modules/vize/bin/vize",
    ).sha256,
  );
  assert.equal(data.provenance.witness?.path, "local-completion-resolve-10592/source-law.rs.txt");
  assert.match(data.provenance.witness?.sha256 ?? "", /^[a-f0-9]{64}$/u);
  assert.equal(sha256(read("custody/source-law.rs")), data.provenance.witness?.sha256);
  return {
    spec,
    localItem,
    capabilities: json(spec.capabilities),
    initialFiles: initialFiles.map(({ path: name }) => ({ path: name, bytes: read(name) })),
    changedConfig: read(spec.transition.input),
  };
}

export function readLocalResolveSourceSession(repository: string) {
  const directory = path.join(repository, LOCAL_RESOLVE_SESSION_DIRECTORY);
  const rawCase = fs.readFileSync(path.join(directory, "case.json"));
  const data = JSON.parse(rawCase.toString("utf8")) as FixtureData;
  const files = new Map(
    data.files!.map((entry) => [
      entry.runtimePath,
      readPinnedArtifact(directory, { path: entry.source, sha256: entry.sha256 }),
    ]),
  );
  const plan = JSON.parse(
    readPinnedArtifact(directory, {
      path: data.expected.source,
      sha256: data.expected.sha256,
    }).toString("utf8"),
  );
  return {
    ...validateLocalResolveSourceSession(data, files, plan),
    data,
    files,
    caseSha256: sha256(rawCase),
  };
}
