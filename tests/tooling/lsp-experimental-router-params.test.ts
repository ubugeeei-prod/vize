import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import os from "node:os";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";
import {
  createExperimentalRouterWorkspace,
  experimentalRouterDocumentTransport,
  normalizeExperimentalRouterPackets,
  type RouterGeneratedDocument,
} from "./support/router-experimental-transport.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const custody = path.join(
  root,
  "tests/_fixtures/differential/lsp/experimental-router-params-10518",
);
const output = path.join(root, "target/differential/experimental-router-params");
const relativePage = "src/pages/products/[id=int].vue";
const options = {
  completion: true,
  editor: true,
  ecosystem: true,
  hover: true,
  lint: false,
  typecheck: true,
};
const methods = [
  "textDocument/hover",
  "textDocument/completion",
  "textDocument/definition",
  "textDocument/references",
];
type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type Golden = {
  source: string;
  diagnostics: Json;
  requests: Array<{ method: string; params: Json; response: Json }>;
  resolved: Json[];
  invalidDiagnostics: Json;
  generated: Record<"valid" | "invalid", { file: string; sha256: string }>;
};

function golden(variant: string): Golden {
  return JSON.parse(fs.readFileSync(path.join(custody, `${variant}.expected.json`), "utf8"));
}

// Transport addresses move only in path fields and compiler-owned references.
function normalize(
  value: unknown,
  directory: string,
  generated?: RouterGeneratedDocument | null,
): Json {
  const canonical = fs.realpathSync(directory);
  return normalizeExperimentalRouterPackets(value, [directory, canonical], canonical, generated);
}

function generatedGolden(expected: Golden, kind: "valid" | "invalid"): string {
  const entry = expected.generated[kind];
  assert.match(entry.file, /^(?:lf|crlf|explicit)\.(?:valid|invalid)\.virtual\.ts\.txt$/u);
  const bytes = fs.readFileSync(path.join(custody, entry.file));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.sha256);
  const historicalReceipt = fs.readFileSync(path.join(custody, "integrated-current-11964.json"));
  assert.equal(
    createHash("sha256").update(historicalReceipt).digest("hex"),
    "5ed5d324fe566f4d5de43b31bd0e4695b1e6ce2fb359ee493e03d40b56fd54be",
  );
  const successor = JSON.parse(historicalReceipt.toString("utf8"));
  assert.equal(successor.schema, "vize.experimental-router-generated-transition.v1");
  assert.equal(successor.qualificationCredit, false);
  const transition = successor.entries.find((row: { file: string }) => row.file === entry.file);
  assert.ok(transition);
  const sharedHelperReceipt = fs.readFileSync(
    path.join(custody, "shared-helper-preamble-20261106.json"),
  );
  assert.equal(
    createHash("sha256").update(sharedHelperReceipt).digest("hex"),
    "f79c2fe318ad3db71ef2643cf31c0a95225ccad3b287c24b9263cf22f39d5c79",
  );
  const sharedHelper = JSON.parse(sharedHelperReceipt.toString("utf8"));
  assert.equal(sharedHelper.schema, "vize.experimental-router-shared-helper-transition.v1");
  assert.equal(sharedHelper.qualificationCredit, false);
  assert.equal(sharedHelper.previous.path, "integrated-current-11964.json");
  assert.equal(
    sharedHelper.previous.sha256,
    createHash("sha256").update(historicalReceipt).digest("hex"),
  );
  const currentTransition = sharedHelper.entries.find(
    (row: { file: string }) => row.file === entry.file,
  );
  assert.ok(currentTransition);
  assert.equal(currentTransition.previousSha256, transition.currentSha256);
  const strictExtraReceipt = fs.readFileSync(path.join(custody, "strict-extra-keys-20261107.json"));
  assert.equal(
    createHash("sha256").update(strictExtraReceipt).digest("hex"),
    "0122a13467baad68f8f6415ab8eccedfce9e17c33217bb7c73c7fc3b93695347",
  );
  const strictExtra = JSON.parse(strictExtraReceipt.toString("utf8"));
  assert.equal(strictExtra.schema, "vize.experimental-router-strict-extra-keys-transition.v1");
  assert.equal(strictExtra.qualificationCredit, false);
  assert.equal(strictExtra.previous.path, "shared-helper-preamble-20261106.json");
  assert.equal(
    strictExtra.previous.sha256,
    createHash("sha256").update(sharedHelperReceipt).digest("hex"),
  );
  const strictTransition = strictExtra.entries.find(
    (row: { file: string }) => row.file === entry.file,
  );
  assert.ok(strictTransition);
  assert.equal(strictTransition.previousSha256, currentTransition.currentSha256);
  assert.equal(strictTransition.currentSha256, entry.sha256);
  const ownedReceipt = fs.readFileSync(path.join(custody, successor.ownedConstructor.path));
  assert.equal(
    createHash("sha256").update(ownedReceipt).digest("hex"),
    successor.ownedConstructor.sha256,
  );
  const owned = JSON.parse(ownedReceipt.toString("utf8")).entries.find(
    (row: { file: string }) => row.file === entry.file,
  );
  assert.ok(owned);
  assert.equal(owned.beforeSha256, transition.baselineSha256);
  assert.equal(owned.afterSha256, transition.ownedConstructorSha256);
  const integratedSource = fs.readFileSync(path.resolve(custody, successor.integratedSource.path));
  assert.equal(
    createHash("sha256").update(integratedSource).digest("hex"),
    successor.integratedSource.sha256,
  );
  const current = bytes.toString("utf8");
  assert.equal(current.includes(strictExtra.edit.before), false);
  assert.equal(current.split(strictExtra.edit.after).length, 2);
  assert.equal(current.includes(successor.currentEdit.before), false);
  assert.equal(current.split(successor.currentEdit.after).length, 2);
  return current;
}

function replaceExpected(value: Json, key: "version" | "revision", expected: number): Json {
  if (Array.isArray(value)) return value.map((entry) => replaceExpected(entry, key, expected));
  if (value !== null && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([name, entry]) => [
        name,
        name === key ? expected : replaceExpected(entry, key, expected),
      ]),
    );
  return value;
}

function position(text: string, offset: number): { line: number; character: number } {
  assert.ok(offset >= 0);
  const prefix = text.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    character: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
}

function open(session: LspSession, directory: string, source: string): string {
  const uri = pathToFileURL(path.join(directory, relativePage)).href;
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
  });
  return uri;
}

async function diagnostics(session: LspSession, uri: string, version: number): Promise<unknown> {
  return session.waitForNotification("textDocument/publishDiagnostics", (value) => {
    const packet = value as { uri: string; version: number };
    return packet.uri === uri && packet.version === version;
  });
}

async function queries(session: LspSession, uri: string, source: string) {
  const requests = [];
  const resolved = [];
  let generated: { file: string; sha256: string; source: string } | null = null;
  for (const method of methods) {
    const offset =
      source.indexOf("route.params.page") + (method === "textDocument/completion" ? 13 : 14);
    const params = {
      textDocument: { uri },
      position: position(source, offset),
      ...(method === "textDocument/references" ? { context: { includeDeclaration: true } } : {}),
    };
    const started = performance.now();
    const response = await session.request(method, params);
    requests.push({ method, params, response, elapsedMs: performance.now() - started });
    if (method === "textDocument/completion") {
      assert.ok(Array.isArray(response));
      const candidate = response[0] as
        | { data?: { vizeCompletion?: { item?: { data?: { fileName?: unknown } } } } }
        | undefined;
      const file = candidate?.data?.vizeCompletion?.item?.data?.fileName;
      if (typeof file === "string" && fs.existsSync(file)) {
        const bytes = fs.readFileSync(file);
        generated = {
          file,
          sha256: createHash("sha256").update(bytes).digest("hex"),
          source: bytes.toString("utf8"),
        };
      }
      for (const item of response)
        resolved.push(await session.request("completionItem/resolve", item));
    }
  }
  return { requests, resolved, generated };
}

function assertQueries(
  actual: Awaited<ReturnType<typeof queries>>,
  expected: Golden,
  directory: string,
  revision: number,
  kind: "valid" | "invalid",
  compare: (actual: unknown, expected: unknown, message?: string) => void,
) {
  assert.ok(actual.generated, "the exact generated native document must be retained");
  compare(
    experimentalRouterDocumentTransport(actual.generated.source, fs.realpathSync(directory))
      .normalizedSource,
    generatedGolden(expected, kind),
  );
  const packets = actual.requests.map(({ elapsedMs: _, ...packet }) => packet);
  compare(
    normalize(packets, directory, actual.generated),
    replaceExpected(expected.requests as unknown as Json, "revision", revision),
  );
  compare(
    normalize(actual.resolved, directory, actual.generated),
    replaceExpected(expected.resolved, "revision", revision),
  );
}

function setOptIn(directory: string, enabled: boolean) {
  const file = path.join(directory, "tsconfig.json");
  const document = JSON.parse(fs.readFileSync(file, "utf8"));
  document.vueCompilerOptions = {
    ...document.vueCompilerOptions,
    plugins: enabled ? ["vue-router/volar/sfc-typed-router"] : [],
  };
  fs.writeFileSync(file, `${JSON.stringify(document, null, 2)}\n`);
}

function copyProject(
  providerRoot: string,
  directory: string,
  source: string,
  runtime: string,
  enabled: boolean,
) {
  fs.cpSync(providerRoot, directory, {
    recursive: true,
    filter: (file) => !file.split(path.sep).includes("node_modules"),
  });
  fs.symlinkSync(
    path.join(providerRoot, "node_modules"),
    path.join(directory, "node_modules"),
    "junction",
  );
  fs.writeFileSync(path.join(directory, relativePage), source);
  setOptIn(directory, enabled);
  writeVueConfigFixture(directory, { lsp: options, typeChecker: { corsaPath: runtime } });
}

const structuralCompletion = [
  { detail: "Vue Router file route param: string", kind: 10, label: "id", sortText: "0" },
];
function structuralDiagnostics(uri: string, version: number) {
  return {
    diagnostics: [
      ["page", 12, 34, 38],
      ["tags", 13, 48, 52],
      ["at", 14, 42, 44],
      ["heading", 15, 49, 56],
      ["page", 17, 63, 67],
    ].map(([name, line, start, end]) => ({
      code: "ecosystem/vue-router-route-param",
      message: `Route param \`${name}\` is not defined by this page file. Available params: id`,
      range: { start: { line, character: start }, end: { line, character: end } },
      severity: 2,
      source: "vize/ecosystem",
    })),
    uri,
    version,
  };
}

// This test installs the exact public provider and invokes its real generator.
// The small manifest hover witness is custody, not an experimental-router oracle.
test(
  "configured experimental route params own default editor diagnostics, complete lazy resolution and navigation",
  { timeout: 600_000 },
  async () => {
    const runtime = resolveTypecheckRuntime(root);
    assert.ok(runtime, "the real native TypeScript runtime is required");
    fs.mkdirSync(output, { recursive: true });
    const provider = await prepareExperimentalRouterProject(path.join(output, "provider"));
    const workspaceParent = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-editor-")),
    );
    const receipts: unknown[] = [];
    const failures: string[] = [];
    const compare = (actual: unknown, expected: unknown, message?: string) => {
      try {
        assert.deepEqual(actual, expected, message);
      } catch (error) {
        if (!(error instanceof assert.AssertionError)) throw error;
        failures.push(error.message);
      }
    };
    try {
      for (const variant of ["lf", "crlf"]) {
        const expected = golden(variant);
        const directory = createExperimentalRouterWorkspace(workspaceParent, variant);
        copyProject(provider.root, directory, expected.source, runtime, true);
        const declaration = fs.readFileSync(path.join(directory, "typed-router.d.ts"));
        const session = new LspSession();
        const raw: unknown[] = [];
        session.responseObservers.push((packet) => raw.push(packet));
        session.notificationObservers.push((method, params) => raw.push({ method, params }));
        const receipt = {
          variant,
          source: expected.source,
          raw,
          stages: [] as unknown[],
          stderr: "",
        };
        receipts.push(receipt);
        try {
          await session.initialize(directory, options);
          const uri = open(session, directory, expected.source);
          for (const [version, source] of [
            [1, expected.source],
            [
              2,
              expected.source.replace(
                "route.params.page.toFixed()",
                "route.params.page.toUpperCase()",
              ),
            ],
            [3, expected.source],
          ] as const) {
            if (version !== 1)
              session.notify("textDocument/didChange", {
                textDocument: { uri, version },
                contentChanges: [{ text: source }],
              });
            const published = await diagnostics(session, uri, version);
            const expectedDiagnostics =
              version === 2
                ? expected.invalidDiagnostics
                : replaceExpected(expected.diagnostics, "version", version);
            compare(normalize(published, directory), expectedDiagnostics);
            const actual = await queries(session, uri, source);
            receipt.stages.push({ version, published, ...actual });
            assertQueries(
              actual,
              expected,
              directory,
              version,
              version === 2 ? "invalid" : "valid",
              compare,
            );
          }
          compare(
            fs.readFileSync(path.join(directory, "typed-router.d.ts")),
            declaration,
            "editor requests never regenerate provider declarations",
          );
        } finally {
          await session.shutdown();
          receipt.stderr = session.stderrText;
          fs.writeFileSync(
            path.join(output, "observation.json"),
            `${JSON.stringify({ dependencies: provider.dependencies, failures, receipts }, null, 2)}\n`,
          );
          fs.rmSync(directory, { recursive: true, force: true });
        }
      }

      const expected = golden("explicit");
      const enabled = createExperimentalRouterWorkspace(workspaceParent, "enabled");
      const disabled = createExperimentalRouterWorkspace(workspaceParent, "disabled");
      copyProject(provider.root, enabled, expected.source, runtime, true);
      copyProject(provider.root, disabled, expected.source, runtime, false);
      const session = new LspSession();
      const raw: unknown[] = [];
      session.responseObservers.push((packet) => raw.push(packet));
      session.notificationObservers.push((method, params) => raw.push({ method, params }));
      const receipt = {
        variant: "multi-project-and-config-update",
        raw,
        stages: [] as unknown[],
        stderr: "",
      };
      receipts.push(receipt);
      try {
        await session.initialize([enabled, disabled], options);
        const enabledUri = open(session, enabled, expected.source);
        const nativeQueries = async (label: string, revision: number) => {
          const actual = await queries(session, enabledUri, expected.source);
          receipt.stages.push({ label, revision, ...actual });
          assertQueries(actual, expected, enabled, revision, "valid", compare);
        };
        compare(
          normalize(await diagnostics(session, enabledUri, 1), enabled),
          expected.diagnostics,
        );
        await nativeQueries("before-disabled-sibling", 1);
        const disabledUri = open(session, disabled, expected.source);
        compare(await diagnostics(session, disabledUri, 1), structuralDiagnostics(disabledUri, 1));
        const completion = (uri: string) =>
          session.request("textDocument/completion", {
            textDocument: { uri },
            position: position(expected.source, expected.source.indexOf("route.params.page") + 13),
          });
        compare(await completion(disabledUri), structuralCompletion);
        // The same spelled page/route in another project must not borrow authority.
        await nativeQueries("after-disabled-sibling", 1);
        for (const [version, admitted] of [
          [2, false],
          [3, true],
        ] as const) {
          setOptIn(enabled, admitted);
          session.notify("workspace/didChangeWatchedFiles", {
            changes: [{ uri: pathToFileURL(path.join(enabled, "tsconfig.json")).href, type: 2 }],
          });
          session.notify("textDocument/didChange", {
            textDocument: { uri: enabledUri, version },
            contentChanges: [{ text: expected.source }],
          });
          const published = await diagnostics(session, enabledUri, version);
          if (admitted) {
            compare(
              normalize(published, enabled),
              replaceExpected(expected.diagnostics, "version", version),
            );
            await nativeQueries("restored-after-config-toggle", 4);
          } else {
            compare(published, structuralDiagnostics(enabledUri, version));
            compare(await completion(enabledUri), structuralCompletion);
          }
          compare(await completion(disabledUri), structuralCompletion);
        }
      } finally {
        await session.shutdown();
        receipt.stderr = session.stderrText;
        fs.writeFileSync(
          path.join(output, "observation.json"),
          `${JSON.stringify({ dependencies: provider.dependencies, failures, receipts }, null, 2)}\n`,
        );
        fs.rmSync(enabled, { recursive: true, force: true });
        fs.rmSync(disabled, { recursive: true, force: true });
      }
    } finally {
      provider.cleanup();
      fs.rmSync(workspaceParent, { recursive: true, force: true });
    }
    assert.deepEqual(failures, [], "every independent whole protocol comparison must pass");
  },
);
