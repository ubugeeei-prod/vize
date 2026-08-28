import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  loadProjectionMatrix,
  validateProjectionMatrix,
  type ProjectionFixture,
} from "./support/davinci-ts40-projection.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

test("vize lsp republishes TS-40 local-import prop diagnostics after child edits", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the TS-40 LSP projection gate",
    "TypeScript 7/Corsa runtime not found; skipping TS-40 LSP projection diagnostic test",
  );
  if (corsaPath == null) return;

  const matrix = loadProjectionMatrix(root);
  validateProjectionMatrix(root, matrix);
  const parentFixture = fixture(matrix.fixtures, "parent-local-import");
  const childFixture = fixture(matrix.fixtures, "child-local-import");
  assert.deepEqual(parentFixture.coverage, ["local-vue-import", "props", "navigation-ranges"]);
  assert.ok(childFixture.coverage.includes("props"));

  const parentSource = fs.readFileSync(path.join(root, parentFixture.file), "utf8");
  const initialChild = fs.readFileSync(path.join(root, childFixture.file), "utf8");
  const changedChild = initialChild.replace("message: string", "message: number");
  assert.notEqual(changedChild, initialChild);

  const testRootDir = path.join(testOutputRoot, "lsp-davinci-ts40-projection-diagnostics");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  const diagnostics = observeDiagnostics(session);

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
    linkVuePackage(workspaceDir);
    writeProjectConfig(workspaceDir, corsaPath);

    const parentPath = path.join(sourceDir, path.basename(parentFixture.file));
    const childPath = path.join(sourceDir, path.basename(childFixture.file));
    const parentUri = pathToFileURL(parentPath).href;
    const childUri = pathToFileURL(childPath).href;
    fs.writeFileSync(parentPath, parentSource, "utf8");
    fs.writeFileSync(childPath, initialChild, "utf8");

    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });

    let marker = diagnostics.mark();
    session.notify("textDocument/didOpen", {
      textDocument: { uri: childUri, languageId: "vue", version: 1, text: initialChild },
    });
    const initialChildPublish = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      childUri,
    );
    assert.equal(initialChildPublish.version, 1);
    assert.deepEqual(initialChildPublish.diagnostics, []);

    marker = diagnostics.mark();
    session.notify("textDocument/didOpen", {
      textDocument: { uri: parentUri, languageId: "vue", version: 1, text: parentSource },
    });
    const initialParentPublish = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      parentUri,
    );
    assert.equal(initialParentPublish.version, 1);
    assert.deepEqual(initialParentPublish.diagnostics, []);

    marker = diagnostics.mark();
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 2 },
      contentChanges: [{ text: changedChild }],
    });
    const changedParent = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      parentUri,
      (params) =>
        params.diagnostics.some((diagnostic) =>
          diagnostic.message?.includes("not assignable to type 'number'"),
        ),
    );
    assert.equal(changedParent.version, 1);
    assert.deepEqual(changedParent.diagnostics, [
      expectedMessageDiagnostic(parentSource, "string", "number"),
    ]);
    const changedChildPublish = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      childUri,
      (params) => params.version === 2,
    );
    assert.deepEqual(changedChildPublish.diagnostics, []);

    marker = diagnostics.mark();
    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 3 },
      contentChanges: [{ text: initialChild }],
    });
    const repairedParent = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      parentUri,
      (params) => params.version === 1 && params.diagnostics.length === 0,
    );
    assert.deepEqual(repairedParent.diagnostics, []);
    const repairedChildPublish = await waitForDiagnosticsAfter(
      session,
      diagnostics,
      marker,
      childUri,
      (params) => params.version === 3,
    );
    assert.deepEqual(repairedChildPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function fixture(fixtures: ProjectionFixture[], id: string): ProjectionFixture {
  const found = fixtures.find((fixture) => fixture.id === id);
  assert.ok(found, `TS-40 fixture ${id} must exist`);
  return found;
}

function observeDiagnostics(session: LspSession) {
  const events: unknown[] = [];
  session.notificationObservers.push((method, params) => {
    if (method === "textDocument/publishDiagnostics") events.push(params);
  });
  return {
    mark: () => events.length,
    isAfter: (marker: number, params: unknown) => events.indexOf(params) >= marker,
  };
}

function waitForDiagnosticsAfter(
  session: LspSession,
  diagnostics: ReturnType<typeof observeDiagnostics>,
  marker: number,
  uri: string,
  predicate: (params: PublishDiagnosticsParams) => boolean = () => true,
): Promise<PublishDiagnosticsParams> {
  return session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      diagnostics.isAfter(marker, params) && isDiagnosticsForUri(params, uri) && predicate(params),
  ) as Promise<PublishDiagnosticsParams>;
}

function expectedMessageDiagnostic(source: string, actualType: string, expectedType: string) {
  const bindingOffset = source.indexOf(":message");
  assert.notEqual(bindingOffset, -1);
  const start = offsetToPosition(source, bindingOffset + ":".length);
  return {
    code: 2322,
    message: `Type '${actualType}' is not assignable to type '${expectedType}'.`,
    range: {
      start,
      end: { line: start.line, character: start.character + "message".length },
    },
    severity: 1,
    source: "vize/types",
  };
}

function writeProjectConfig(workspaceDir: string, corsaPath: string): void {
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    JSON.stringify({ lsp: { lint: false, typecheck: true }, typeChecker: { corsaPath } }),
    "utf8",
  );
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        strict: true,
        target: "ES2022",
      },
      include: ["src/**/*"],
    }),
    "utf8",
  );
}

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = path.join(root, "node_modules/vue");
  if (!fs.existsSync(vuePackage)) {
    writeVueShim(workspaceDir);
    return;
  }
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), "dir");
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), "dir");
  }
}

function writeVueShim(workspaceDir: string): void {
  fs.writeFileSync(
    path.join(workspaceDir, "src/vue-shim.d.ts"),
    `declare module "vue" {
  export interface Ref<T = any> {
    value: T;
  }

  export interface ShallowRef<T = any> {
    value: T;
  }

  export type DefineComponent<Props = {}> = new () => { $props: Props };

  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>;
    $slots: Record<string, unknown>;
    $refs: Record<string, unknown>;
    $emit: (...args: any[]) => void;
  }

  export interface GlobalComponents {}
}
`,
    "utf8",
  );
}
