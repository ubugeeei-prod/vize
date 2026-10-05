import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { resolveVuePackagePath } from "../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

type Position = { line: number; character: number };
type Request = {
  component: string;
  role: string;
  params: { position: Position };
  result: unknown;
};

await test("component hovers wrap complete long types and preserve exact short contracts", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native TypeScript runtime for complete component hovers",
    "native TypeScript runtime unavailable",
  );
  if (!corsaPath) return;
  const fixtureRoot = path.join(root, "tests/_fixtures/differential/lsp");
  const fixture = loadLspManifest(path.join(fixtureRoot, "manifest.json")).cases.find(
    (row) => row.id === "lsp/regression/component-hover-readability",
  );
  assert.ok(fixture);
  const caseRoot = path.join(fixtureRoot, "component-hover-readability");
  const provenance = JSON.parse(
    readPinnedArtifact(caseRoot, fixture.data.provenance.witness!).toString("utf8"),
  );
  const reference = provenance.references.find(
    (row: { source: string }) => row.source === "requests.expected.json",
  );
  assert.ok(reference);
  const requests = JSON.parse(
    readPinnedArtifact(caseRoot, { path: reference.source, sha256: reference.sha256 }).toString(
      "utf8",
    ),
  ) as Request[];
  assert.deepEqual(
    requests.map(({ component, role }) => [component, role]),
    ["LongComponent", "ShortComponent"].flatMap((name) =>
      ["import", "script", "tag"].map((role) => [name, role]),
    ),
  );
  // PR tooling already builds this executable and its mandatory source receipt.
  // Published-payload replay reuses the committed oracle through a strict launcher.
  const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const previousBinary = process.env.VIZE_LSP_BIN;
  const previousRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-hover-"));
  let session: LspSession | undefined;
  try {
    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
    for (const file of fixture.files) {
      fs.writeFileSync(path.join(workspace, file.runtimePath), file.bytes);
    }
    const nodeModules = path.join(workspace, "node_modules");
    fs.mkdirSync(nodeModules);
    const vuePackage = resolveVuePackagePath();
    fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), "junction");
    const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
    if (fs.existsSync(vueNamespace)) {
      fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), "junction");
    }
    fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}');
    fs.writeFileSync(
      path.join(workspace, "vize.config.json"),
      JSON.stringify({
        lsp: { hover: true, lint: false, typecheck: true },
        typeChecker: { corsaPath },
      }),
    );
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
        },
        include: ["*.vue"],
      }),
    );
    const source = fixture.files
      .find((file) => file.runtimePath === "App.vue")!
      .bytes.toString("utf8");
    const uri = pathToFileURL(path.join(workspace, "App.vue")).href;
    session = new LspSession();
    await session.initialize(workspace, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", text: source, version: 1 },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );
    const observations = [];
    for (const row of requests) {
      const started = performance.now();
      const result = await session.request(
        "textDocument/hover",
        { ...row.params, textDocument: { uri } },
        120_000,
      );
      const elapsedMs = performance.now() - started;
      observations.push({ ...row, actual: result, elapsedMs });
    }
    const output = path.join(root, "target/differential/component-hover-readability.json");
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(
      output,
      `${JSON.stringify(
        {
          schema: "vize.component-hover-readability.observation",
          version: 1,
          sourceAuthority: fixture.data.provenance.witness,
          binary: process.env.VIZE_LSP_BIN,
          vuePackage: {
            path: vuePackage,
            identity: JSON.parse(fs.readFileSync(path.join(vuePackage, "package.json"), "utf8")),
          },
          serverProcessId: session.processId,
          scope: "six whole public hovers; bounded elapsed observations, no speedup claim",
          observations,
        },
        null,
        2,
      )}\n`,
    );
    t.diagnostic(`complete hover and bounded latency observation: ${output}`);
    for (const row of observations) {
      assert.deepEqual(row.actual, row.result, `${row.component} ${row.role}: complete hover`);
    }
    for (const file of fixture.files) {
      assert.deepEqual(fs.readFileSync(path.join(workspace, file.runtimePath)), file.bytes);
    }
  } finally {
    try {
      await session?.shutdown();
    } finally {
      try {
        fs.rmSync(workspace, { recursive: true, force: true });
      } finally {
        if (previousBinary === undefined) delete process.env.VIZE_LSP_BIN;
        else process.env.VIZE_LSP_BIN = previousBinary;
        if (previousRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
        else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = previousRequired;
      }
    }
  }
});
