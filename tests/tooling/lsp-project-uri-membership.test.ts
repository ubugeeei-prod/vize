import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";
import { binaryRelativePath } from "../differential/build-receipt.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { JsonRpcMessage } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  snapshotSessionCaptures,
  verifySessionCaptures,
} from "./support/lsp/verify-session-captures.ts";
import {
  archiveNative,
  auditNative,
  beforeAuthority,
  fixture,
  membershipCounters,
  normalizedMemberPacket,
  object,
  read,
  relocate,
  sha256,
  type NativeAudit,
} from "./support/project-uri-membership-10622.ts";
import { createPassiveCompilerProxy } from "./support/project-uri-membership-position-10622.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import {
  prepareProjectUriProvider,
  resolveProjectUriVue,
} from "./support/project-uri-membership-provider-10622.ts";

type Row = {
  label: string;
  method: string;
  params: { textDocument: { uri: string }; [key: string]: unknown };
  response: JsonRpcMessage;
};
type ObservedRow = Row & { elapsedMs: number };
const before = beforeAuthority();
const provider = prepareProjectUriProvider();
const expectedRows: Row[] = before.packets.rows;
const output = path.join(testOutputRoot, "project-uri-membership-10622");
const historical = path.join(root, "tests/_fixtures/differential/lsp/warm-type-backed-requests");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const beforeMain = {
  uri: `file://${before.authority.roots.nativeProject}/src/c/Comp0.vue.ts`,
  text: gunzipSync(
    fs.readFileSync(path.join(fixture, "public-0.567.0.native-main.ts.gz")),
  ).toString(),
  languageId: "typescript",
  version: 1,
};
const beforeQueries = read("public-0.567.0.native-completion-queries.json") as Array<{
  params: NativeAudit["completions"][number];
}>;

test("the original published 15 whole packets and 534 native sources remain immutable", () => {
  assert.equal(before.authority.sourceRevision, "2934b35b18393c188b3da5f8e50595299589fb8c");
  assert.equal(before.observation.provider.cliVersion, "vize 0.567.0");
  assert.equal(before.observation.dependencies.runtime.version, "Version 7.0.2");
  assert.equal(before.observation.dependencies.vue.version, "3.5.41");
  assert.equal(expectedRows.length, 15);
  assert.equal(expectedRows.filter((row) => row.label.endsWith("member-completion")).length, 2);
  assert.equal(before.inputs.files.length, 534);
  assert.equal(read("public-0.567.0.native-sources.json").length, 534);
  for (const [file, digest] of Object.entries(before.inputs.authority))
    assert.equal(sha256(fs.readFileSync(path.join(historical, file))), digest, file);
});

test(
  "generated position relocation refuses changed code, another owner, and an off-by-one position",
  { skip: !provider.vue },
  () => {
    const row = expectedRows.find((entry) => entry.label === "round0-member-completion");
    assert.ok(row);
    normalizedMemberPacket(row, before.authority.roots, beforeMain, beforeQueries[0].params);
    assert.throws(() =>
      normalizedMemberPacket(
        row,
        before.authority.roots,
        { ...beforeMain, text: `${beforeMain.text}changed` },
        beforeQueries[0].params,
      ),
    );
    assert.throws(() =>
      normalizedMemberPacket(
        row,
        before.authority.roots,
        { ...beforeMain, uri: "file:///another-owner.vue.ts" },
        beforeQueries[0].params,
      ),
    );
    const moved = structuredClone(row);
    const items = object(moved.response).result;
    assert.ok(Array.isArray(items));
    const data = object(object(object(object(items[0]).data).vizeCompletion).item);
    object(data.data).position = Number(object(data.data).position) + 1;
    assert.throws(() =>
      normalizedMemberPacket(moved, before.authority.roots, beforeMain, beforeQueries[0].params),
    );
  },
);

test("source work evidence refuses absent counters and legacy pair scans", () => {
  assert.throws(() => membershipCounters(""));
  const rows = ["merge_dependencies", "merge_materialized", "project_filter"].map(
    (stage) =>
      `phase="canonical_uri_membership" stage="${stage}" entries=4 parses=8 probes=4 physical_resolutions=8 legacy_comparisons=0 physical_resets=1 primary_comparisons=1`,
  );
  assert.equal(membershipCounters(rows.join("\n")).length, 3);
  assert.throws(() =>
    membershipCounters(rows.join("\n").replace("legacy_comparisons=0", "legacy_comparisons=1")),
  );
  assert.throws(() =>
    membershipCounters(rows.join("\n").replace("physical_resolutions=8", "physical_resolutions=9")),
  );
  assert.throws(() => membershipCounters(rows.slice(0, 2).join("\n")));
});

test("explicit provider failures never select a fallback or invoke the installer", () => {
  const parent = path.join(output, "provider-rejections");
  fs.mkdirSync(parent, { recursive: true });
  const directory = fs.mkdtempSync(path.join(parent, "wrong-version-"));
  fs.writeFileSync(path.join(directory, "package.json"), '{"name":"vue","version":"3.5.40"}\n');
  let installations = 0;
  const install = (): never => {
    installations++;
    throw new Error("unexpected dependency fallback");
  };
  try {
    for (const explicitVue of ["", path.join(directory, "missing"), directory])
      assert.throws(() =>
        resolveProjectUriVue(
          { requestedSource: true, dependenciesRequired: true, explicitVue },
          install,
        ),
      );
    assert.equal(installations, 0);
    assert.deepEqual(
      resolveProjectUriVue(
        { requestedSource: false, dependenciesRequired: false, explicitVue: undefined },
        install,
      ),
      { mode: "not-requested", provider: undefined },
    );
    if (provider.vue)
      assert.equal(
        resolveProjectUriVue(
          { requestedSource: true, dependenciesRequired: true, explicitVue: provider.vue },
          install,
        ).mode,
        "explicit",
      );
    assert.equal(installations, 0);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

function prepareWorkspace(directory: string, runtime: string, vue: string, observed: boolean) {
  const workspace = path.join(directory, "workspace");
  fs.mkdirSync(workspace, { recursive: true });
  for (const file of ["gen.mjs", "package.json", "tsconfig.json"])
    fs.copyFileSync(path.join(historical, file), path.join(workspace, file));
  execFileSync(process.execPath, [path.join(workspace, "gen.mjs"), "400"]);
  const files: Array<{ path: string; bytes: number; sha256: string }> = [];
  for (const part of ["c", "lib"])
    for (const name of fs.readdirSync(path.join(workspace, "src", part)).sort()) {
      const relative = `src/${part}/${name}`;
      const bytes = fs.readFileSync(path.join(workspace, relative));
      files.push({ path: relative, bytes: bytes.length, sha256: sha256(bytes) });
    }
  assert.deepEqual(files, before.inputs.files, "all original generated source bytes");
  assert.equal(
    fs.readFileSync(path.join(workspace, "tsconfig.json"), "utf8"),
    before.inputs.tsconfig,
  );
  fs.mkdirSync(path.join(workspace, "node_modules"));
  fs.symlinkSync(vue, path.join(workspace, "node_modules/vue"), "dir");
  fs.symlinkSync(
    fs.realpathSync(path.join(vue, "../@vue")),
    path.join(workspace, "node_modules/@vue"),
    "dir",
  );
  let selectedRuntime = runtime;
  if (observed) {
    const { proxy, observer: source } = createPassiveCompilerProxy(directory);
    write(path.join(directory, "proxy-source.json"), {
      source: "editors/vscode/test/fixtures/passive-vize-server.cjs",
      sha256: sha256(source),
      wrapperSha256: sha256(fs.readFileSync(proxy)),
      meaning: "unchanged passive forwarding with an absolute Node shebang",
    });
    selectedRuntime = proxy;
  }
  const config = {
    vue: {
      languageServer: { lint: true, typecheck: true, editor: true, ecosystem: true },
      typeChecker: { corsaPath: selectedRuntime },
    },
  };
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    `export default ${JSON.stringify(config)};\n`,
  );
  write(path.join(directory, "inputs.json"), {
    authority: before.inputs.authority,
    files,
    config,
    tsconfigSha256: sha256(fs.readFileSync(path.join(workspace, "tsconfig.json"))),
  });
  return fs.realpathSync(workspace);
}

async function run(directory: string, runtime: string, vue: string, observed: boolean) {
  fs.mkdirSync(directory, { recursive: true });
  const workspace = prepareWorkspace(directory, runtime, vue, observed);
  const nativeFile = path.join(directory, "native-protocol.jsonl");
  const keys = [
    "VIZE_TEST_OBSERVED_SERVER_PATH",
    "VIZE_TEST_CONFIG_PROTOCOL_PATH",
    "VIZE_TRACE_EDITOR_PREPARATION",
    "RUST_LOG",
    "NO_COLOR",
  ];
  const previous = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  process.env.RUST_LOG = observed ? "info" : "warn";
  process.env.NO_COLOR = "1";
  if (observed) {
    process.env.VIZE_TEST_OBSERVED_SERVER_PATH = runtime;
    process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH = nativeFile;
    process.env.VIZE_TRACE_EDITOR_PREPARATION = "1";
  } else for (const key of keys.slice(0, 3)) delete process.env[key];
  let session: LspSession | undefined;
  let closeProviderSession: (() => void) | undefined;
  let live: NativeAudit | undefined;
  const rows: ObservedRow[] = [];
  const packets: JsonRpcMessage[] = [];
  const notifications: unknown[] = [];
  try {
    closeProviderSession = provider.openSession();
    session = new LspSession({
      repoRoot: root,
      binary: process.env.VIZE_LSP_BIN ?? binaryRelativePath(),
    });
    session.responseObservers.push((packet) => packets.push(packet));
    session.notificationObservers.push((method, params) => notifications.push({ method, params }));
    const initialization = await session.initialize(workspace, {
      lint: true,
      typecheck: true,
      editor: true,
      ecosystem: true,
    });
    write(path.join(directory, "initialization.json"), initialization);
    for (const file of ["src/c/Comp0.vue", "src/c/Comp7.vue", "src/lib/mod0.ts"]) {
      const uri = pathToFileURL(path.join(workspace, file)).href;
      const text = fs.readFileSync(path.join(workspace, file), "utf8");
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri,
          languageId: file.endsWith(".vue") ? "vue" : "typescript",
          version: 1,
          text,
        },
      });
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
        45000,
      );
    }
    const uri = pathToFileURL(path.join(workspace, "src/c/Comp0.vue")).href;
    for (const expected of expectedRows) {
      const params = structuredClone(expected.params);
      assert.equal(
        params.textDocument.uri,
        `file://${before.authority.roots.workspace}/src/c/Comp0.vue`,
      );
      params.textDocument.uri = uri;
      let id: number | undefined;
      const started = performance.now();
      const result = await session.request(expected.method, params, 90000, (requestId) => {
        id = requestId;
        return [];
      });
      const elapsedMs = performance.now() - started;
      assert.deepEqual(
        packets.filter((packet) => packet.id === id),
        [{ jsonrpc: "2.0", id, result }],
      );
      rows.push({
        label: expected.label,
        method: expected.method,
        params,
        response: { jsonrpc: "2.0", id, result },
        elapsedMs,
      });
      write(path.join(directory, "progress.json"), { rows, notifications });
    }
    if (observed) live = await auditNative(nativeFile, workspace, runtime);
    const nativeProject = live?.roots.nativeProject ?? "unused-by-the-13-strict-nonmember-packets";
    const roots = { workspace, nativeProject };
    let member = 0;
    for (const [index, row] of rows.entries()) {
      const packet = {
        label: row.label,
        method: row.method,
        params: row.params,
        response: row.response,
      };
      const expected = expectedRows[index];
      if (row.label.endsWith("member-completion")) {
        if (live)
          assert.deepEqual(
            normalizedMemberPacket(packet, roots, live.main, live.completions[member]),
            normalizedMemberPacket(
              expected,
              before.authority.roots,
              beforeMain,
              beforeQueries[member].params,
            ),
          );
        member++;
      } else
        assert.deepEqual(
          relocate(packet, roots),
          relocate(expected, before.authority.roots),
          row.label,
        );
    }
    assert.equal(member, 2);
    if (observed)
      write(
        path.join(directory, "membership-counters.json"),
        membershipCounters(session.stderrText),
      );
    write(path.join(directory, "results.json"), {
      meaning: observed
        ? "captured and traced source correctness; proxy IO is not a latency admission"
        : "same exact source, direct runtime without native proxy/tracing; whole-request observations after first diagnostics, not universal or globally cold latency",
      comparedWholePackets: observed ? 15 : 13,
      rawMemberTimingRows: observed ? 0 : 2,
      roots,
      rows,
      notifications,
    });
  } finally {
    try {
      if (session) {
        await session.shutdown();
        await session.waitForClosed();
        closeProviderSession?.();
      }
    } finally {
      if (session) fs.writeFileSync(path.join(directory, "stderr.log"), session.stderrText);
      write(path.join(directory, "complete-outer-packets.json"), packets);
      for (const [key, value] of Object.entries(previous)) {
        if (value === undefined) delete process.env[key];
        else process.env[key] = value;
      }
    }
  }
  if (observed && live) {
    const closed = await auditNative(nativeFile, workspace, runtime, live);
    write(path.join(directory, "native-source-audit.json"), closed.summary);
    write(path.join(directory, "native-archive.json"), await archiveNative(nativeFile));
  }
  return rows;
}

test("source original400 preserves full packets and native membership with bounded URI-index work", async (t) => {
  if (!provider.requestedSource) {
    t.skip("source-only observation was not requested; provider readiness grants no source credit");
    return;
  }
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "actual native TypeScript",
    "native TypeScript unavailable",
    provider.dependenciesRequired,
  );
  const vue = requireTypecheckDependency(
    t,
    provider.vue,
    "isolated real Vue3.5.41",
    "run the focused workflow with its frozen Vue install",
    provider.dependenciesRequired,
  );
  if (!runtime || !vue) return;
  assert.equal(
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    "1",
    "source receipt custody is mandatory; published fallback is forbidden",
  );
  assert.equal(execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(), "Version 7.0.2");
  const actualVue = fs.realpathSync(vue);
  const manifest = fs.readFileSync(path.join(actualVue, "package.json"));
  assert.equal(JSON.parse(manifest.toString()).version, "3.5.41");
  assert.equal(sha256(manifest), before.authority.vuePackageJsonSha256);
  fs.mkdirSync(output, { recursive: true });
  const directory = fs.mkdtempSync(path.join(output, "source-"));
  write(path.join(directory, "providers.json"), {
    runtime: {
      path: fs.realpathSync(runtime),
      sha256: sha256(fs.readFileSync(runtime)),
      version: "Version 7.0.2",
    },
    vue: { path: actualVue, version: "3.5.41", packageJsonSha256: sha256(manifest) },
    before: { authority: "published0.567.0 darwin-arm64", source: before.authority.sourceRevision },
    after: {
      authority: "exact receipt-required source build",
      source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
    },
  });
  const captures = snapshotSessionCaptures(root);
  const captured = await run(path.join(directory, "captured"), runtime, actualVue, true);
  const timing = await run(
    path.join(directory, "direct-runtime-timings"),
    runtime,
    actualVue,
    false,
  );
  const verified = verifySessionCaptures(root, captures);
  assert.equal(verified.sessions.length, 2);
  for (const session of verified.sessions)
    assert.deepEqual(session.process, { exitStatus: 0, signal: null, error: null });
  write(path.join(directory, "outer-session-receipts.json"), verified);
  t.diagnostic(
    JSON.stringify({
      scope:
        "source CI observations only; no latency budget, production speedup, or native whole-product admission",
      capturedWholePackets: captured.length,
      directRuntimeWholeRequestMs: timing.map(({ label, elapsedMs }) => ({ label, elapsedMs })),
      timingMemberPackets:
        "two complete raw outer member packets retained; independent native query custody belongs only to the separate captured correctness process",
    }),
  );
});
