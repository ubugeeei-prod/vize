import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";
import { binaryRelativePath } from "../differential/build-receipt.ts";
import { decodeFrames } from "../differential/lsp-wire.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { JsonRpcMessage } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  snapshotSessionCaptures,
  verifySessionCaptures,
} from "./support/lsp/verify-session-captures.ts";
import { archiveNative, fixture, object, sha256 } from "./support/project-uri-membership-10622.ts";
import {
  assertNativeCompletionExchange,
  nativeImportRelocation,
  normalizeMemberPacket,
  normalizePacket,
  sdkPositionUnitEvidence,
} from "./support/project-uri-membership-position-10622.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { prepareProjectUriProvider } from "./support/project-uri-membership-provider-10622.ts";

const directory = path.join(fixture, "position-unit");
const json = (name: string) => JSON.parse(fs.readFileSync(path.join(directory, name), "utf8"));
const profileAuthority = json("authority.json");
assert.equal(
  sha256(fs.readFileSync(path.join(directory, "authority.json"))),
  "d984207dfcad837445345f84805dfb19820ac49b176aebd02ccb8262ce336b58",
);
for (const [file, hash] of Object.entries(profileAuthority.files))
  assert.equal(sha256(fs.readFileSync(path.join(directory, file))), hash, file);
type Profile = {
  name: string;
  source: string;
  sourceSha256: string;
  native: string;
  nativeSourceSha256: string;
  roots: { workspace: string; nativeProject: string };
  document: { uri: string; languageId: string; version: number };
  nativeExchange: {
    pid: number;
    request: {
      id: number;
      params: { textDocument: { uri: string }; position: { line: number; character: number } };
    };
    response: unknown;
  };
  nativePositionEncoding: string;
  nativeUtf8Offset: number;
  nativeUtf16Offset: number;
  packet: { label: string; method: string; params: unknown; response: JsonRpcMessage };
};
const profiles = json("profiles.json") as Profile[];
const paths = {
  authored: new Set(["src/AstralLF.vue", "src/AstralCRLF.vue", "src/Child.vue", "src/model.ts"]),
  native: new Set([
    "src/AstralLF.vue.ts",
    "src/AstralCRLF.vue.ts",
    "src/Child.vue.ts",
    "src/model.ts",
  ]),
};
const text = (profile: Profile) =>
  gunzipSync(fs.readFileSync(path.join(directory, profile.native))).toString();
const normalized = (profile: Profile, row = profile.packet) =>
  normalizeMemberPacket(
    row,
    profile.roots,
    { ...profile.document, text: text(profile) },
    profile.nativeExchange.request.params,
    { roots: profile.roots, text: text(profile) },
    paths,
  );
const provider = prepareProjectUriProvider();
const hasParser = !!provider.vue;

test("pinned native SDK source and actual public astral packets prove UTF8 completion-data offsets", () => {
  sdkPositionUnitEvidence();
  assert.equal(profiles.length, 2);
  for (const profile of profiles) {
    const source = fs.readFileSync(path.join(directory, profile.source));
    assert.equal(sha256(source), profile.sourceSha256);
    assert.ok(source.includes(Buffer.from("🧭é日本語")));
    assert.equal(source.includes(Buffer.from("\r\n")), profile.name === "CRLF");
    assert.equal(sha256(text(profile)), profile.nativeSourceSha256);
    assert.equal(profile.nativePositionEncoding, "utf-16");
    const observed = assertNativeCompletionExchange(
      profile.packet,
      { ...profile.document, text: text(profile) },
      profile.nativeExchange.request,
      profile.nativeExchange.response,
    );
    assert.deepEqual(observed, {
      utf8Bytes: profile.nativeUtf8Offset,
      utf16Units: profile.nativeUtf16Offset,
    });
    assert.equal(observed.utf8Bytes, 15994);
    assert.equal(observed.utf16Units, 15985);
  }
});

test(
  "completion comparison rejects foreign frames, root lookalikes, unknown IDs, ranges and revisions",
  { skip: !hasParser },
  () => {
    const profile = profiles[0];
    const expected = normalized(profile);
    for (const delta of [-1, 1]) {
      const changed = structuredClone(profile.packet);
      const item = object(
        object(changed.response).result instanceof Array
          ? (object(changed.response).result as unknown[])[0]
          : undefined,
      );
      const data = object(object(object(item.data).vizeCompletion).item);
      object(data.data).position = Number(object(data.data).position) + delta;
      assert.throws(() => normalized(profile, changed));
    }
    const foreign = structuredClone(profile.nativeExchange.request);
    foreign.params.textDocument.uri = "file:///foreign-frame.vue.ts";
    assert.throws(() =>
      assertNativeCompletionExchange(
        profile.packet,
        { ...profile.document, text: text(profile) },
        foreign,
        profile.nativeExchange.response,
      ),
    );
    const wrongId = {
      ...profile.nativeExchange.request,
      id: profile.nativeExchange.request.id + 1,
    };
    assert.throws(() =>
      assertNativeCompletionExchange(
        profile.packet,
        { ...profile.document, text: text(profile) },
        wrongId,
        profile.nativeExchange.response,
      ),
    );
    for (const addition of [
      `\nconst quoted = ${JSON.stringify(profile.roots.nativeProject)};`,
      `\n/* ${profile.roots.nativeProject} */`,
      `\nconst lookalike = \`import Child from "${profile.roots.nativeProject}/src/Child.vue.ts";\`;`,
    ])
      assert.throws(() => nativeImportRelocation(text(profile) + addition, profile.roots));
    assert.throws(() =>
      nativeImportRelocation(text(profile), {
        ...profile.roots,
        workspace: profile.roots.nativeProject,
      }),
    );
    for (const key of ["opaqueId", "range", "revision"]) {
      const changed = structuredClone(profile.packet);
      const items = object(changed.response).result;
      assert.ok(Array.isArray(items));
      if (key === "opaqueId") object(items[0]).opaqueId = profile.roots.nativeProject;
      if (key === "range")
        object(items[0]).range = {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 1 },
        };
      if (key === "revision") object(object(object(items[0]).data).vizeCompletion).revision = 2;
      assert.throws(() => assert.deepEqual(normalized(profile, changed), expected));
    }
    const oldRows = JSON.parse(
      fs.readFileSync(path.join(fixture, "public-0.567.0.packets.json"), "utf8"),
    ).rows;
    const reference = oldRows.find(
      (row: { label: string }) => row.label === "round0-local-references",
    );
    const beforeRoots = JSON.parse(
      fs.readFileSync(path.join(fixture, "authority.json"), "utf8"),
    ).roots;
    const changed = structuredClone(reference);
    changed.response.result[0].range.start.character++;
    assert.throws(() =>
      assert.deepEqual(
        normalizePacket(changed, beforeRoots),
        normalizePacket(reference, beforeRoots),
      ),
    );
  },
);

function nativeExchanges(file: string, runtime: string) {
  const events = fs
    .readFileSync(file, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const owners = new Map<number, string>();
  const docs = new Map<string, { uri: string; text: string }>();
  const queries = new Map<
    string,
    { request: Profile["nativeExchange"]["request"]; document: { uri: string; text: string } }
  >();
  const encodings = new Map<number, string>();
  const streams = new Map<string, { chunks: Buffer[]; packets: unknown[] }>();
  const completed: Array<{
    pid: number;
    document: { uri: string; text: string };
    request: Profile["nativeExchange"]["request"];
    response: unknown;
    encoding: string;
  }> = [];
  for (const event of events) {
    if (event.kind === "spawn") {
      assert.equal(fs.realpathSync(event.binary), fs.realpathSync(runtime));
      assert.equal(event.binarySha256, sha256(fs.readFileSync(runtime)));
      owners.set(event.pid, event.argv.includes("--lsp") ? "lsp" : "other");
    }
    if (owners.get(event.pid) !== "lsp") continue;
    if (!["chunk", "frame"].includes(event.kind)) continue;
    const key = `${event.pid}/${event.direction}`;
    const stream = streams.get(key) ?? { chunks: [], packets: [] };
    streams.set(key, stream);
    if (event.kind === "chunk") {
      stream.chunks.push(Buffer.from(event.chunkBase64, "base64"));
      continue;
    }
    assert.deepEqual(decodeFrames(Buffer.from(event.frameBase64, "base64")).messages, [
      event.packet,
    ]);
    const packet = event.packet;
    stream.packets.push(packet);
    if (event.direction === "server" && packet.result?.capabilities)
      encodings.set(event.pid, packet.result.capabilities.positionEncoding);
    if (event.direction === "client" && packet.method === "textDocument/didOpen") {
      const doc = packet.params.textDocument;
      docs.set(`${event.pid}/${doc.uri}`, { uri: doc.uri, text: doc.text });
    }
    if (event.direction === "client" && packet.method === "textDocument/completion") {
      const document = docs.get(`${event.pid}/${packet.params.textDocument.uri}`);
      assert.ok(document);
      queries.set(`${event.pid}/${packet.id}`, { request: packet, document });
    }
    const query = queries.get(`${event.pid}/${packet.id}`);
    if (event.direction === "server" && query) {
      assert.equal(encodings.get(event.pid), "utf-16");
      completed.push({ pid: event.pid, ...query, response: packet, encoding: "utf-16" });
    }
  }
  for (const stream of streams.values())
    assert.deepEqual(decodeFrames(Buffer.concat(stream.chunks)).messages, stream.packets);
  assert.equal(completed.length, 2);
  return completed;
}

test("source LF and CRLF astral controls bind complete member packets to actual native UTF8 query offsets", async (t) => {
  if (!provider.requestedSource) {
    t.skip("source-only observation was not requested; provider readiness grants no source credit");
    return;
  }
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "real native SDK7.0.2",
    "native runtime unavailable",
    provider.dependenciesRequired,
  );
  const vue = requireTypecheckDependency(
    t,
    provider.vue,
    "real isolated Vue3.5.41",
    "run the source qualification workflow",
    provider.dependenciesRequired,
  );
  if (!runtime || !vue) return;
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(), "Version 7.0.2");
  const sdk = sdkPositionUnitEvidence(runtime);
  const base = path.join(testOutputRoot, "project-uri-membership-10622");
  fs.mkdirSync(base, { recursive: true });
  const output = fs.mkdtempSync(path.join(base, "position-unit-"));
  const workspace = path.join(output, "workspace");
  fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
  for (const profile of profiles)
    fs.copyFileSync(
      path.join(directory, profile.source),
      path.join(workspace, `src/Astral${profile.name}.vue`),
    );
  for (const name of ["Child.vue", "model.ts"])
    fs.copyFileSync(path.join(directory, `${name}.txt`), path.join(workspace, "src", name));
  fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}\n');
  fs.copyFileSync(
    path.join(root, "tests/_fixtures/differential/lsp/warm-type-backed-requests/tsconfig.json"),
    path.join(workspace, "tsconfig.json"),
  );
  fs.mkdirSync(path.join(workspace, "node_modules"));
  fs.symlinkSync(fs.realpathSync(vue), path.join(workspace, "node_modules/vue"), "dir");
  fs.symlinkSync(
    fs.realpathSync(path.join(vue, "../@vue")),
    path.join(workspace, "node_modules/@vue"),
    "dir",
  );
  const proxy = path.join(output, "compiler-proxy");
  const observer = fs.readFileSync(
    path.join(root, "editors/vscode/test/fixtures/passive-vize-server.cjs"),
  );
  fs.writeFileSync(proxy, Buffer.concat([Buffer.from(`#!${process.execPath}\n`), observer]), {
    mode: 0o755,
  });
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    `export default ${JSON.stringify({ vue: { languageServer: { editor: true, typecheck: true, lint: false }, typeChecker: { corsaPath: proxy } } })};\n`,
  );
  const native = path.join(output, "native-protocol.jsonl");
  const previous = {
    VIZE_TEST_OBSERVED_SERVER_PATH: process.env.VIZE_TEST_OBSERVED_SERVER_PATH,
    VIZE_TEST_CONFIG_PROTOCOL_PATH: process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH,
  };
  process.env.VIZE_TEST_OBSERVED_SERVER_PATH = runtime;
  process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH = native;
  const captures = snapshotSessionCaptures(root);
  const closeProviderSession = provider.openSession();
  const session = new LspSession({
    repoRoot: root,
    binary: process.env.VIZE_LSP_BIN ?? binaryRelativePath(),
  });
  const observed: Array<Profile["packet"]> = [];
  const responses: unknown[] = [];
  let liveExchanges: ReturnType<typeof nativeExchanges> | undefined;
  session.responseObservers.push((packet) => responses.push(packet));
  try {
    await session.initialize(workspace, { editor: true, typecheck: true, lint: false });
    for (const profile of profiles) {
      const file = path.join(workspace, `src/Astral${profile.name}.vue`);
      const uri = pathToFileURL(file).href;
      const source = fs.readFileSync(file, "utf8");
      assert.equal(sha256(source), profile.sourceSha256);
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
        45000,
      );
      const params = structuredClone(profile.packet.params);
      object(object(params).textDocument).uri = uri;
      let id: number | undefined;
      const result = await session.request(
        "textDocument/completion",
        params,
        90000,
        (requestId) => {
          id = requestId;
          return [];
        },
      );
      observed.push({
        label: profile.name,
        method: "textDocument/completion",
        params,
        response: { jsonrpc: "2.0", id, result },
      });
    }
    const exchanges = nativeExchanges(native, runtime);
    liveExchanges = exchanges;
    for (const [index, profile] of profiles.entries()) {
      const exchange = exchanges[index];
      const physical = fileURLToPath(exchange.document.uri);
      const marker = `${path.sep}src${path.sep}`;
      const nativeProject = physical.slice(0, physical.lastIndexOf(marker));
      assert.equal(fs.realpathSync(nativeProject), nativeProject);
      assert.ok(fs.statSync(nativeProject).isDirectory());
      assert.equal(fs.statSync(path.resolve(nativeProject, "../..")).mode & 0o777, 0o700);
      assertNativeCompletionExchange(
        observed[index],
        exchange.document,
        exchange.request,
        exchange.response,
      );
      const current = normalizeMemberPacket(
        observed[index],
        { workspace: fs.realpathSync(workspace), nativeProject },
        exchange.document,
        exchange.request.params,
        { roots: profile.roots, text: text(profile) },
        paths,
      );
      assert.deepEqual(current, normalized(profile));
    }
    fs.writeFileSync(
      path.join(output, "source-unit-proof.json"),
      JSON.stringify({ sdk, observerSha256: sha256(observer), observed, exchanges }, null, 2) +
        "\n",
    );
  } finally {
    await session.shutdown();
    await session.waitForClosed();
    closeProviderSession();
    fs.writeFileSync(path.join(output, "stderr.log"), session.stderrText);
    fs.writeFileSync(
      path.join(output, "complete-outer-packets.json"),
      JSON.stringify(responses, null, 2) + "\n",
    );
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  assert.deepEqual(
    nativeExchanges(native, runtime),
    liveExchanges,
    "closed full native streams retain the exact live-owned completion exchanges",
  );
  fs.writeFileSync(
    path.join(output, "native-archive.json"),
    JSON.stringify(await archiveNative(native), null, 2) + "\n",
  );
  const verified = verifySessionCaptures(root, captures);
  assert.equal(verified.sessions.length, 1);
  assert.deepEqual(verified.sessions[0].process, { exitStatus: 0, signal: null, error: null });
  fs.writeFileSync(
    path.join(output, "outer-session-receipts.json"),
    JSON.stringify(verified, null, 2) + "\n",
  );
});
