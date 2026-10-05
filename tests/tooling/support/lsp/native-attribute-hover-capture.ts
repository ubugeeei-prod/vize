import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveVuePackagePath } from "../../../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { readPinnedArtifact, sha256 } from "../../../differential/harness.mjs";
import { errorText, hasCapabilities } from "../../../differential/lsp-types.ts";
import { LspWire } from "../../../differential/lsp-wire.ts";
import { resolveTypecheckRuntime } from "../typecheck-dependency.ts";
import type { HoverCapture, HoverCorpus, HoverSession } from "./native-attribute-hover-contract.ts";

function fileIdentity(file: string) {
  const resolved = fs.realpathSync(file);
  const bytes = fs.readFileSync(resolved);
  return {
    path: file,
    resolvedPath: resolved,
    bytes: bytes.length,
    sha256: sha256(bytes) as string,
  };
}

export function hoverProviders(root: string) {
  const runtime = resolveTypecheckRuntime(root);
  assert.ok(runtime, "native TypeScript is mandatory for the original hover corpus");
  const runtimeProbe = spawnSync(runtime, ["--version"], {
    timeout: 30_000,
    maxBuffer: 1024 * 1024,
  });
  const vuePath = resolveVuePackagePath();
  const vueManifest = fs.readFileSync(path.join(vuePath, "package.json"));
  return {
    runtime: fileIdentity(runtime),
    runtimeProbe: {
      status: runtimeProbe.status,
      signal: runtimeProbe.signal,
      stdoutBase64: runtimeProbe.stdout?.toString("base64") ?? "",
      stderrBase64: runtimeProbe.stderr?.toString("base64") ?? "",
      error: runtimeProbe.error?.message ?? null,
    },
    vue: {
      path: vuePath,
      manifestSha256: sha256(vueManifest) as string,
      manifest: JSON.parse(vueManifest.toString("utf8")) as Record<string, unknown>,
    },
  };
}

export async function captureNativeAttributeSession(
  binary: string,
  fixtureRoot: string,
  corpus: HoverCorpus,
  session: HoverSession,
  providers: ReturnType<typeof hoverProviders>,
): Promise<HoverCapture & Record<string, unknown>> {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-attribute-7993-"));
  const rootUri = pathToFileURL(workspace).href;
  const uri = pathToFileURL(path.join(workspace, "Field.vue")).href;
  const files = {
    "Field.vue": readPinnedArtifact(fixtureRoot, session.file),
    "tsconfig.json": readPinnedArtifact(fixtureRoot, corpus.config),
    "package.json": Buffer.from('{"private":true,"type":"module"}\n'),
    "vize.config.json": Buffer.from(
      `${JSON.stringify({
        lsp: corpus.initialization.options,
        typeChecker: { corsaPath: providers.runtime.path },
      })}\n`,
    ),
  };
  const responses: HoverCapture["responses"] = [];
  const failures: string[] = [];
  let wire: LspWire | undefined;
  let initialization: unknown = null;
  let diagnostics: unknown = null;
  try {
    for (const [name, bytes] of Object.entries(files))
      fs.writeFileSync(path.join(workspace, name), bytes);
    fs.mkdirSync(path.join(workspace, "node_modules"));
    fs.symlinkSync(providers.vue.path, path.join(workspace, "node_modules/vue"), "junction");
    const vueNamespace = path.join(path.dirname(providers.vue.path), "@vue");
    if (fs.existsSync(vueNamespace))
      fs.symlinkSync(vueNamespace, path.join(workspace, "node_modules/@vue"), "junction");
    wire = new LspWire(binary, ["lsp"], workspace);
    initialization = await wire.request("initialize", {
      processId: process.pid,
      rootUri,
      capabilities: corpus.initialization.capabilities,
      initializationOptions: corpus.initialization.options,
    });
    assert.ok(hasCapabilities(initialization as Parameters<typeof hasCapabilities>[0]));
    wire.notify("initialized", {});
    wire.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        text: files["Field.vue"].toString("utf8"),
        version: 1,
      },
    });
    diagnostics = await wire.waitFor(
      (message) =>
        message.method === "textDocument/publishDiagnostics" &&
        message.params?.uri === uri &&
        message.params?.version === 1,
      120_000,
    );
    for (const request of session.requests) {
      try {
        const response = await wire.request("textDocument/hover", {
          textDocument: { uri },
          position: request.position,
        });
        responses.push({ role: request.role, response, error: null });
      } catch (error) {
        responses.push({ role: request.role, response: null, error: errorText(error) });
      }
    }
    await wire.finish();
  } catch (error) {
    failures.push(errorText(error));
  } finally {
    try {
      await wire?.stop();
    } catch (error) {
      failures.push(errorText(error));
    }
    for (const [name, bytes] of Object.entries(files)) {
      try {
        assert.deepEqual(
          fs.readFileSync(path.join(workspace, name)),
          bytes,
          `${name}: unchanged physical input`,
        );
      } catch (error) {
        failures.push(errorText(error));
      }
    }
    fs.rmSync(workspace, { recursive: true, force: true });
  }
  for (const request of session.requests.slice(responses.length)) {
    responses.push({
      role: request.role,
      response: null,
      error: "session failed before this request",
    });
  }
  return {
    id: session.id,
    workspaceUri: uri,
    rootUri,
    failure: failures.length ? failures.join("\n") : null,
    responses,
    initialization,
    diagnostics,
    serverProcessId: wire?.child.pid ?? null,
    physicalFiles: Object.entries(files).map(([name, bytes]) => ({
      name,
      bytes: bytes.length,
      sha256: sha256(bytes) as string,
      bytesBase64: bytes.toString("base64"),
    })),
    wire: wire?.observation() ?? {
      clientWireBase64: "",
      serverWireBase64: "",
      stderrBase64: "",
      exitStatus: null,
      signal: null,
      processError: "server was not started",
    },
  };
}
