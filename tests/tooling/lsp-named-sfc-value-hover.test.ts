import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolveVuePackagePath } from "../../tools/support/compat/editor-e2e/real-vue-workspace.ts";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

type Location = {
  uri: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
};
type Request = { id: string; method: string; params: unknown; result: unknown };
type Phase = { type: string; publication: unknown; requests: Request[]; childSource: string };

const fixture = path.join(root, "tests/_fixtures/differential/lsp/named-sfc-value-hover-11953");
const reference = JSON.parse(fs.readFileSync(path.join(fixture, "reference.json"), "utf8")) as {
  schema: string;
  inputs: Record<string, string>;
  files: Record<string, string>;
};

function compareLocations(a: Location, b: Location): number {
  return (
    (a.uri < b.uri ? -1 : a.uri > b.uri ? 1 : 0) ||
    a.range.start.line - b.range.start.line ||
    a.range.start.character - b.range.start.character ||
    a.range.end.line - b.range.end.line ||
    a.range.end.character - b.range.end.character
  );
}

/** Normalize only the authored workspace prefix and reference-set order. */
function normalize(value: unknown, workspace: string): unknown {
  if (typeof value === "string" && value.startsWith("file://")) {
    const actual = fs.realpathSync(fileURLToPath(value));
    const relative = path.relative(fs.realpathSync(workspace), actual);
    assert.equal(path.isAbsolute(relative), false, "native URI stays in the authored workspace");
    assert.notEqual(relative.split(path.sep)[0], "..", "native URI never leaks a mirror path");
    return `file:///workspace/${relative.split(path.sep).join("/")}`;
  }
  if (Array.isArray(value)) return value.map((item) => normalize(item, workspace));
  if (value !== null && typeof value === "object") {
    const result = Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, normalize(item, workspace)]),
    );
    if (result.method === "textDocument/references" && Array.isArray(result.result)) {
      // References are an unordered protocol set. Sorting retains every field
      // and duplicate row; the raw packet is independently preserved below.
      result.result = (result.result as Location[]).toSorted(compareLocations);
    }
    return result;
  }
  return value;
}

await test(
  "named SFC values retain complete native types, navigation and negative diagnostics through unsaved edits",
  { timeout: 180_000 },
  async (t) => {
    assert.equal(reference.schema, "vize.named-sfc-value-hover.reference.v1");
    for (const [file, hash] of Object.entries(reference.files)) {
      assert.equal(
        createHash("sha256")
          .update(fs.readFileSync(path.join(fixture, file)))
          .digest("hex"),
        hash,
      );
    }
    assert.deepEqual(Object.keys(reference.inputs).sort(), ["App.vue", "Child.vue", "barrel.ts"]);
    const expected = JSON.parse(fs.readFileSync(path.join(fixture, "expected.json.txt"), "utf8"));
    const registered = loadLspManifest(
      path.join(path.dirname(fixture), "manifest.json"),
    ).cases.find(({ id }) => id === "lsp/regression/named-sfc-value-hover-11953");
    assert.ok(registered, "the complete native witness also registers immutable LSP inputs");
    assert.ok(registered.data.provenance.witness);
    assert.deepEqual(
      JSON.parse(readPinnedArtifact(fixture, registered.data.provenance.witness).toString("utf8")),
      reference,
      "the registered case pins the complete public/native reference custody",
    );
    assert.deepEqual(
      Object.fromEntries(
        registered.files
          .filter(({ runtimePath }) => Object.hasOwn(reference.inputs, runtimePath))
          .map(({ runtimePath, bytes }) => [runtimePath, bytes.toString("utf8")]),
      ),
      reference.inputs,
      "the registered authored sources retain the original direct-import reproducer",
    );
    assert.deepEqual(
      registered.requests,
      expected[0].requests
        .filter(({ id }: Request) => id === "direct" || id === "alias")
        .map(({ params, result }: { params: { position: unknown }; result: unknown }) => ({
          params: { position: params.position },
          result,
        })),
      "registered whole hovers retain the same native control oracle",
    );
    const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
    let sourceIdentity: unknown;
    resolveVizeLaunchCommand(undefined, binary, {
      required: true,
      onVerifiedLaunch: (launch) => {
        sourceIdentity = launch;
      },
    });
    const corsaPath = resolveTypecheckRuntime(root);
    assert.ok(corsaPath, "the native TypeScript provider is mandatory for this P0 oracle");
    const runtimePackage = path.dirname(path.dirname(fs.realpathSync(corsaPath)));
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-named-sfc-value-"));
    const phases: Phase[] = [];
    const report: Record<string, unknown> = {
      schema: "vize.named-sfc-value-hover.observation.v1",
      reference,
      inputs: reference.inputs,
      sourceIdentity,
      corsaPath,
      typeScriptProvider: {
        metadata: JSON.parse(fs.readFileSync(path.join(runtimePackage, "package.json"), "utf8")),
        sha256: createHash("sha256").update(fs.readFileSync(corsaPath)).digest("hex"),
      },
      workspace,
      phases,
    };
    const output = path.join(root, "target/differential/named-sfc-value-hover-11953/report.json");
    fs.mkdirSync(path.dirname(output), { recursive: true });
    const persist = () => fs.writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`);
    let session: LspSession | undefined;
    try {
      for (const [file, source] of Object.entries(reference.inputs)) {
        fs.writeFileSync(path.join(workspace, file), source);
      }
      const vuePackage = resolveVuePackagePath();
      report.vuePackage = {
        path: vuePackage,
        metadata: JSON.parse(fs.readFileSync(path.join(vuePackage, "package.json"), "utf8")),
      };
      fs.mkdirSync(path.join(workspace, "node_modules"));
      fs.symlinkSync(vuePackage, path.join(workspace, "node_modules/vue"), "junction");
      const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
      if (fs.existsSync(vueNamespace)) {
        fs.symlinkSync(vueNamespace, path.join(workspace, "node_modules/@vue"), "junction");
      }
      fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}');
      fs.writeFileSync(
        path.join(workspace, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "bundler",
            strict: true,
            skipLibCheck: true,
            noEmit: true,
          },
          include: ["*.vue", "*.ts"],
        }),
      );
      const config = writeVueConfigFixture(workspace, {
        lsp: { editor: true, hover: true, lint: false, typecheck: true },
        typeChecker: { corsaPath },
      });
      report.config = fs.readFileSync(config, "utf8");
      session = new LspSession({ repoRoot: root, binary });
      report.serverProcessId = session.processId;
      await session.initialize(workspace, {
        editor: true,
        hover: true,
        lint: false,
        typecheck: true,
      });
      const app = reference.inputs["App.vue"];
      const child = reference.inputs["Child.vue"];
      const uri = pathToFileURL(path.join(workspace, "App.vue")).href;
      const childUri = pathToFileURL(path.join(workspace, "Child.vue")).href;
      for (const [index, type] of ["number", "string", "number"].entries()) {
        const childSource =
          type === "number" ? child : child.replace("number = 1", 'string = "changed"');
        if (index > 0) {
          session.notify(
            index === 1 ? "textDocument/didOpen" : "textDocument/didChange",
            index === 1
              ? {
                  textDocument: { uri: childUri, languageId: "vue", version: 1, text: childSource },
                }
              : {
                  textDocument: { uri: childUri, version: 2 },
                  contentChanges: [{ text: childSource }],
                },
          );
        }
        session.notify(
          index === 0 ? "textDocument/didOpen" : "textDocument/didChange",
          index === 0
            ? { textDocument: { uri, languageId: "vue", version: 1, text: app } }
            : { textDocument: { uri, version: index + 1 }, contentChanges: [{ text: app }] },
        );
        const publication = await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === index + 1,
          120_000,
        );
        const phase: Phase = { type, publication, requests: [], childSource };
        phases.push(phase);
        persist();
        for (const [id, offset] of [
          ["direct", app.indexOf("= childValue") + 2],
          ["alias", app.indexOf("observedValue")],
          ["alias-use", app.lastIndexOf("observedValue;")],
          ["barrel", app.indexOf("= barrelValue") + 2],
          ["default", app.lastIndexOf("<Child") + 1],
          ["default-as", app.lastIndexOf("<DirectChild") + 1],
          ["barrel-component", app.lastIndexOf("<BarrelChild") + 1],
        ] as const) {
          const params = { textDocument: { uri }, position: offsetToPosition(app, offset) };
          const result = await session.request("textDocument/hover", params, 120_000);
          phase.requests.push({ id, method: "textDocument/hover", params, result });
          persist();
        }
        for (const method of ["textDocument/definition", "textDocument/references"]) {
          const params = {
            textDocument: { uri },
            position: offsetToPosition(app, app.indexOf("= childValue") + 2),
            ...(method === "textDocument/references"
              ? { context: { includeDeclaration: true } }
              : {}),
          };
          phase.requests.push({
            id: method,
            method,
            params,
            result: await session.request(method, params, 120_000),
          });
          persist();
        }
      }
      // Retain every complete native packet before any semantic verdict.
      report.normalizedPhases = normalize(phases, workspace);
      persist();
      assert.deepEqual(report.normalizedPhases, expected);
      for (const [file, source] of Object.entries(reference.inputs)) {
        assert.equal(
          fs.readFileSync(path.join(workspace, file), "utf8"),
          source,
          "unsaved dependency edits leave all authored files unchanged",
        );
      }
      assert.deepEqual(
        normalize(phases[2].requests, workspace),
        normalize(phases[0].requests, workspace),
      );
      t.diagnostic(`complete source-bound native response packet: ${output}`);
    } finally {
      try {
        await session?.shutdown();
        report.exitedGracefully = session?.exitedGracefully;
        report.stderr = session?.stderrText;
        if (session) assert.equal(session.exitedGracefully, true);
      } finally {
        persist();
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }
  },
);
