import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import {
  BUILD_RECIPE,
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";
import {
  archiveNative,
  beforeAuthority,
  fixture,
  normalizedMemberPacket,
  object,
  read,
  relocate,
  sha256,
} from "../project-uri-membership-10622.ts";
import { createPassiveCompilerProxy } from "../project-uri-membership-position-10622.ts";
import { prepareProjectUriProvider } from "../project-uri-membership-provider-10622.ts";
import { resolveTypecheckRuntime } from "../typecheck-dependency.ts";
import { isDiagnosticsForUri } from "./assertions.ts";
import { root } from "./paths.ts";
import { LspSession } from "./session.ts";
import { snapshotSessionCaptures, verifySessionCaptures } from "./verify-session-captures.ts";
import { auditSourceControlNative } from "./project-uri-source-control-native-10622.ts";

const QUALIFIER = "6573b92e74c93883303e34e9f12869868f567207";
const SOURCES = {
  control: "225b884276cdc066a692bf2cd1b51a15e952e97b",
  candidate: QUALIFIER,
};
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const git = (directory: string, args: string[]) =>
  execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trim();

function argumentsForRun(args: string[]): {
  role: keyof typeof SOURCES;
  sourceRevision: string;
  producerRoot: string;
  trial: string;
  output: string;
} {
  assert.equal(args.length, 5, "role, producer source/root, trial and output are explicit");
  const [role, sourceRevision, producerRoot, trial, output] = args;
  assert.ok(role === "control" || role === "candidate");
  assert.equal(sourceRevision, SOURCES[role]);
  assert.ok(trial === "1" || trial === "2");
  assert.ok(path.isAbsolute(producerRoot) && path.isAbsolute(output));
  assert.equal(fs.realpathSync(producerRoot), producerRoot);
  assert.equal(git(producerRoot, ["rev-parse", "HEAD"]), sourceRevision);
  assert.equal(git(producerRoot, ["status", "--porcelain", "--untracked-files=no"]), "");
  const relative = path.relative(
    path.join(root, "target/vize-tests/project-uri-source-control-10622"),
    output,
  );
  assert.ok(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  assert.equal(fs.existsSync(output), false, "each trial output starts absent");
  return { role, sourceRevision, producerRoot, trial, output };
}

export async function observeSourceControl(args = process.argv.slice(2)) {
  const selected = argumentsForRun(args);
  const { role, sourceRevision, producerRoot, trial, output } = selected;
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(git(root, ["merge-base", QUALIFIER, "HEAD"]), QUALIFIER);
  const common = [
    "tests/differential/build-receipt.ts",
    "tests/tooling/support/lsp/session.ts",
    "tests/tooling/support/lsp/session-process.ts",
    "tests/tooling/support/lsp/launch.ts",
    "tests/tooling/support/lsp/published-launch.ts",
    "tests/tooling/support/lsp/verify-session-captures.ts",
    "tests/tooling/support/project-uri-membership-10622.ts",
    "tests/tooling/support/project-uri-membership-position-10622.ts",
    "tests/tooling/support/project-uri-membership-provider-10622.ts",
    "editors/vscode/test/fixtures/passive-vize-server.cjs",
  ];
  const qualifierFiles = Object.fromEntries(
    common.map((file) => {
      const expected = execFileSync("git", ["show", `${QUALIFIER}:${file}`], { cwd: root });
      const current = fs.readFileSync(path.join(root, file));
      assert.deepEqual(current, expected, `unchanged shared qualifier ${file}`);
      return [file, sha256(current)];
    }),
  );
  const identity = expectedBuildIdentity(producerRoot);
  const receiptFile = path.join(producerRoot, `${binaryRelativePath()}.differential-build.json`);
  const receipt = JSON.parse(fs.readFileSync(receiptFile, "utf8"));
  validateBuildReceipt(receipt, identity, BUILD_RECIPE);
  assert.equal(identity.sourceRevision, sourceRevision);
  const producer = JSON.parse(
    fs.readFileSync(
      path.join(path.dirname(path.dirname(output)), "producers", `${role}.json`),
      "utf8",
    ),
  );
  assert.equal(producer.state, "built");
  assert.equal(producer.role, role);
  assert.equal(producer.sourceRevision, sourceRevision);
  assert.equal(producer.sourceRoot, producerRoot);
  assert.equal(producer.targetInitiallyAbsent, true);
  assert.deepEqual(producer.receipt, receipt);
  const provider = prepareProjectUriProvider();
  const runtime = resolveTypecheckRuntime(root);
  assert.ok(runtime && provider.vue, "the pinned native runtime and Vue are mandatory");
  assert.equal(execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(), "Version 7.0.2");
  const before = beforeAuthority();
  const vueManifest = fs.readFileSync(path.join(provider.vue, "package.json"));
  assert.equal(sha256(vueManifest), before.authority.vuePackageJsonSha256);
  fs.mkdirSync(output, { recursive: true });
  const workspace = path.join(
    producerRoot,
    "target/uri-source-control-workspaces",
    `${role}-${trial}`,
  );
  assert.equal(fs.existsSync(workspace), false, "each source workspace starts absent");
  fs.mkdirSync(workspace, { recursive: true });
  const historical = path.join(root, "tests/_fixtures/differential/lsp/warm-type-backed-requests");
  for (const file of ["gen.mjs", "package.json", "tsconfig.json"]) {
    assert.equal(
      sha256(fs.readFileSync(path.join(historical, file))),
      before.inputs.authority[file],
    );
    fs.copyFileSync(path.join(historical, file), path.join(workspace, file));
  }
  execFileSync(process.execPath, [path.join(workspace, "gen.mjs"), "400"]);
  const files = ["c", "lib"].flatMap((part) =>
    fs
      .readdirSync(path.join(workspace, "src", part))
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
      .map((name) => {
        const relative = `src/${part}/${name}`;
        const bytes = fs.readFileSync(path.join(workspace, relative));
        return { path: relative, bytes: bytes.length, sha256: sha256(bytes) };
      }),
  );
  assert.deepEqual(files, before.inputs.files, "all 534 original authored source bytes");
  fs.mkdirSync(path.join(workspace, "node_modules"));
  fs.symlinkSync(provider.vue, path.join(workspace, "node_modules/vue"), "dir");
  fs.symlinkSync(
    fs.realpathSync(path.join(provider.vue, "../@vue")),
    path.join(workspace, "node_modules/@vue"),
    "dir",
  );
  const { proxy, observer } = createPassiveCompilerProxy(output);
  const config = {
    vue: {
      languageServer: { lint: true, typecheck: true, editor: true, ecosystem: true },
      typeChecker: { corsaPath: proxy },
    },
  };
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    `export default ${JSON.stringify(config)};\n`,
  );
  const native = path.join(output, "native-protocol.jsonl");
  process.env.VIZE_TEST_OBSERVED_SERVER_PATH = runtime;
  process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH = native;
  process.env.VIZE_TRACE_EDITOR_PREPARATION = "1";
  process.env.RUST_LOG = "info";
  process.env.NO_COLOR = "1";
  write(path.join(output, "identities.json"), {
    role,
    trial,
    producer,
    qualifierRevision: QUALIFIER,
    experimentRevision: git(root, ["rev-parse", "HEAD"]),
    qualifierFiles,
    observer: {
      file: fileURLToPath(import.meta.url),
      sha256: sha256(fs.readFileSync(fileURLToPath(import.meta.url))),
      passiveSha256: sha256(observer),
    },
    runtime: {
      path: fs.realpathSync(runtime),
      sha256: sha256(fs.readFileSync(runtime)),
      version: "Version 7.0.2",
    },
    vue: {
      path: fs.realpathSync(provider.vue),
      packageJsonSha256: sha256(vueManifest),
      version: "3.5.41",
    },
    workspace,
    inputs: files,
    config,
    meaning:
      "Controlled lifecycle observation. Native owner counts are measurements; the original single-owner gate remains unchanged. No timing or speed claim.",
  });
  const snapshot = snapshotSessionCaptures(producerRoot);
  const closeProvider = provider.openSession();
  const session = new LspSession({ repoRoot: producerRoot, binary: binaryRelativePath() });
  const packets: unknown[] = [];
  const notifications: unknown[] = [];
  const timeline: unknown[] = [];
  const rows: Array<Record<string, unknown>> = [];
  const checkpoint = (name: string) =>
    timeline.push({
      name,
      hrtimeNs: process.hrtime.bigint().toString(),
      nativeBytes: fs.existsSync(native) ? fs.statSync(native).size : 0,
    });
  session.responseObservers.push((packet) => packets.push(packet));
  session.notificationObservers.push((method, params) => notifications.push({ method, params }));
  try {
    write(
      path.join(output, "initialization.json"),
      await session.initialize(workspace, {
        lint: true,
        typecheck: true,
        editor: true,
        ecosystem: true,
      }),
    );
    for (const file of ["src/c/Comp0.vue", "src/c/Comp7.vue", "src/lib/mod0.ts"]) {
      const uri = pathToFileURL(path.join(workspace, file)).href;
      checkpoint(`before didOpen ${file}`);
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri,
          languageId: file.endsWith(".vue") ? "vue" : "typescript",
          version: 1,
          text: fs.readFileSync(path.join(workspace, file), "utf8"),
        },
      });
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
        45000,
      );
      checkpoint(`after diagnostics ${file}`);
      write(path.join(output, "progress.json"), { timeline, rows, notifications });
    }
    const uri = pathToFileURL(path.join(workspace, "src/c/Comp0.vue")).href;
    for (const expected of before.packets.rows) {
      const params = structuredClone(expected.params);
      assert.equal(
        params.textDocument.uri,
        `file://${before.authority.roots.workspace}/src/c/Comp0.vue`,
      );
      params.textDocument.uri = uri;
      let id: number | undefined;
      checkpoint(`before ${expected.label}`);
      const result = await session.request(expected.method, params, 90000, (requestId) => {
        id = requestId;
        return [];
      });
      assert.deepEqual(
        packets.filter(
          (packet) => packet && typeof packet === "object" && "id" in packet && packet.id === id,
        ),
        [{ jsonrpc: "2.0", id, result }],
      );
      rows.push({
        label: expected.label,
        method: expected.method,
        params,
        response: { jsonrpc: "2.0", id, result },
      });
      checkpoint(`after ${expected.label}`);
      write(path.join(output, "progress.json"), { timeline, rows, notifications });
    }
  } finally {
    try {
      await session.shutdown();
      await session.waitForClosed();
      closeProvider();
    } finally {
      fs.writeFileSync(path.join(output, "stderr.log"), session.stderrText);
      write(path.join(output, "complete-outer-packets.json"), packets);
      write(path.join(output, "progress.json"), { timeline, rows, notifications });
    }
  }
  assert.deepEqual(
    expectedBuildIdentity(producerRoot),
    identity,
    "producer executable/source remain exact",
  );
  const closed = verifySessionCaptures(producerRoot, snapshot);
  assert.equal(closed.sessions.length, 1);
  assert.deepEqual(closed.sessions[0].process, { exitStatus: 0, signal: null, error: null });
  write(path.join(output, "outer-custody.json"), closed);
  const audit = await auditSourceControlNative(native, workspace, runtime, {
    role,
    sourceRevision,
    cliReceiptRoot: producerRoot,
  });
  const beforeMain = {
    uri: `file://${before.authority.roots.nativeProject}/src/c/Comp0.vue.ts`,
    text: gunzipSync(
      fs.readFileSync(path.join(fixture, "public-0.567.0.native-main.ts.gz")),
    ).toString(),
    languageId: "typescript",
    version: 1,
  };
  const beforeQueries = read("public-0.567.0.native-completion-queries.json");
  assert.equal(rows.length, 15);
  let member = 0;
  for (const [index, row] of rows.entries()) {
    const expected = before.packets.rows[index];
    if (String(row.label).endsWith("member-completion")) {
      const exchange = audit.completionExchanges[member];
      assert.equal(exchange.response.id, exchange.request.id);
      const outerItems = object(row.response).result;
      const nativeItems = object(exchange.response.result).items;
      assert.ok(Array.isArray(outerItems) && Array.isArray(nativeItems));
      assert.deepEqual(
        outerItems.map((item) => object(object(object(item).data).vizeCompletion).item),
        nativeItems,
        "every native completion item remains complete in its outer response",
      );
      assert.deepEqual(
        normalizedMemberPacket(row, exchange.roots, exchange.document, exchange.request.params),
        normalizedMemberPacket(
          expected,
          before.authority.roots,
          beforeMain,
          beforeQueries[member].params,
        ),
      );
      member++;
    } else assert.deepEqual(relocate(row, audit.roots), relocate(expected, before.authority.roots));
  }
  assert.equal(member, 2);
  write(path.join(output, "native-audit.json"), audit.summary);
  write(path.join(output, "native-archive.json"), await archiveNative(native));
  write(path.join(output, "result.json"), {
    state: "closed source lifecycle observation; not original single-owner qualification",
    role,
    sourceRevision,
    trial,
    wholePackets: rows.length,
    sourceCode: audit.sources,
    native: audit.summary,
    timeline,
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await observeSourceControl();
}
