import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";
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
};

function golden(variant: string): Golden {
  return JSON.parse(fs.readFileSync(path.join(custody, `${variant}.expected.json`), "utf8"));
}

// Only allocated process and workspace addresses vary. Every protocol field,
// range, revision and generated-file offset remains part of the comparison.
function normalize(value: unknown, directory: string): Json {
  const roots = new Set([directory, fs.realpathSync(directory)]);
  if (directory.startsWith("/private/")) roots.add(directory.slice(8));
  else if (directory.startsWith("/var/")) roots.add(`/private${directory}`);
  const visit = (entry: unknown): Json => {
    if (typeof entry === "string") {
      let result = entry.replace(
        /^(file:\/\/)?\/[^\n]*?\/vize-canon\/editor\/sessions\/session-[^/]+\/projects\/[0-9a-f]{64}(?=\/)/,
        (_, scheme: string | undefined) => `${scheme ?? ""}$VIRTUAL`,
      );
      for (const address of [...roots].sort((a, b) => b.length - a.length))
        result = result.replaceAll(address, "$PROJECT");
      return result;
    }
    if (Array.isArray(entry)) return entry.map(visit);
    if (entry !== null && typeof entry === "object")
      return Object.fromEntries(Object.entries(entry).map(([key, item]) => [key, visit(item)]));
    assert.ok(entry === null || typeof entry === "boolean" || typeof entry === "number");
    return entry;
  };
  return visit(value);
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
  compare: (actual: unknown, expected: unknown, message?: string) => void,
) {
  const packets = actual.requests.map(({ elapsedMs: _, ...packet }) => packet);
  compare(
    normalize(packets, directory),
    replaceExpected(expected.requests as unknown as Json, "revision", revision),
  );
  compare(
    normalize(actual.resolved, directory),
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
        const directory = path.join(output, variant);
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
            assertQueries(actual, expected, directory, version, compare);
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
      const enabled = path.join(output, "enabled");
      const disabled = path.join(output, "disabled");
      copyProject(provider.root, enabled, expected.source, runtime, true);
      copyProject(provider.root, disabled, expected.source, runtime, false);
      const session = new LspSession();
      const raw: unknown[] = [];
      session.responseObservers.push((packet) => raw.push(packet));
      session.notificationObservers.push((method, params) => raw.push({ method, params }));
      const receipt = { variant: "multi-project-and-config-update", raw, stderr: "" };
      receipts.push(receipt);
      try {
        await session.initialize([enabled, disabled], options);
        const enabledUri = open(session, enabled, expected.source);
        compare(
          normalize(await diagnostics(session, enabledUri, 1), enabled),
          expected.diagnostics,
        );
        assertQueries(
          await queries(session, enabledUri, expected.source),
          expected,
          enabled,
          1,
          compare,
        );
        const disabledUri = open(session, disabled, expected.source);
        compare(await diagnostics(session, disabledUri, 1), structuralDiagnostics(disabledUri, 1));
        const completion = (uri: string) =>
          session.request("textDocument/completion", {
            textDocument: { uri },
            position: position(expected.source, expected.source.indexOf("route.params.page") + 13),
          });
        compare(await completion(disabledUri), structuralCompletion);
        // The same spelled page/route in another project must not borrow authority.
        assertQueries(
          await queries(session, enabledUri, expected.source),
          expected,
          enabled,
          1,
          compare,
        );
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
            assertQueries(
              await queries(session, enabledUri, expected.source),
              expected,
              enabled,
              4,
              compare,
            );
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
    }
    assert.deepEqual(failures, [], "every independent whole protocol comparison must pass");
  },
);
