import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

// This reduced input retains the registry-pinned Volt binding declaration.
// Its real Vue ref type comes from a physical declaration file, not an import
// inserted into the SFC or a guessed hover signature.
const fixture = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/lsp-nuxt-auto-import-hover.json"), "utf8"),
) as { source: string; ambient: string };
const { source, ambient } = fixture;

for (const ownDeclaration of [true, false]) {
  test(
    ownDeclaration
      ? "Nuxt auto-import hover uses the owning app declaration and Vue ref type"
      : "Nuxt auto-import hover refuses a sibling app declaration",
    async (t) => {
      const corsaPath = requireTypecheckDependency(
        t,
        resolveTypecheckRuntime(root),
        "TypeScript 7/Corsa runtime for Nuxt auto-import hover",
        "TypeScript 7/Corsa runtime not found; skipping Nuxt auto-import hover test",
      );
      if (corsaPath == null) return;
      const parent = path.join(testOutputRoot, "lsp-nuxt-auto-import-hover");
      fs.mkdirSync(parent, { recursive: true });
      const workspace = fs.mkdtempSync(path.join(parent, "workspace-"));
      const app = path.join(workspace, "apps/volt");
      const sibling = path.join(workspace, "apps/sibling");
      const session = new LspSession();
      let initialized = false;
      try {
        for (const directory of [app, sibling]) {
          fs.mkdirSync(path.join(directory, ".nuxt"), { recursive: true });
          fs.mkdirSync(path.join(directory, "src"), { recursive: true });
          linkVuePackage(directory);
          fs.writeFileSync(
            path.join(directory, "tsconfig.json"),
            JSON.stringify({ extends: "./.nuxt/tsconfig.json" }),
          );
          fs.writeFileSync(
            path.join(directory, ".nuxt/tsconfig.json"),
            JSON.stringify({
              compilerOptions: {
                strict: true,
                skipLibCheck: true,
                module: "ESNext",
                moduleResolution: "bundler",
                target: "ES2022",
                lib: ["ES2022", "DOM"],
              },
              include: ["../src/**/*.vue", "./imports.d.ts"],
            }),
          );
        }
        // This declaration exists in both cases. It must not give the queried
        // app a ref binding when its own config has no such declaration.
        fs.writeFileSync(path.join(sibling, ".nuxt/imports.d.ts"), ambient);
        if (ownDeclaration) fs.writeFileSync(path.join(app, ".nuxt/imports.d.ts"), ambient);
        fs.writeFileSync(
          path.join(workspace, "vize.config.json"),
          JSON.stringify({
            lsp: { hover: true, lint: false, typecheck: true },
            typeChecker: { corsaPath },
          }),
        );
        const file = path.join(app, "src/FilesCard.vue");
        const uri = pathToFileURL(file).href;
        fs.writeFileSync(file, source);
        await session.initialize(workspace, {
          editor: true,
          hover: true,
          lint: false,
          typecheck: true,
        });
        initialized = true;
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        const notification = await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
          120_000,
        );
        const diagnostics = (
          notification as {
            diagnostics: Array<{ code?: string | number; message: string; range: Range }>;
          }
        ).diagnostics;
        const unresolvedRef = diagnostics.filter(
          (item) => Number(item.code) === 2304 && /Cannot find name 'ref'/.test(item.message),
        );
        if (ownDeclaration) {
          assert.deepEqual(
            diagnostics,
            [],
            "the owning app must typecheck against its own declarations",
          );
          assert.deepEqual(unresolvedRef, []);
          await assertHover(
            session,
            uri,
            rangeFor("tags", source.indexOf("const tags")),
            /const tags: Ref<string\[\]/,
          );
          await assertHover(
            session,
            uri,
            rangeFor("tags", source.indexOf("{{ tags")),
            /const tags: string\[\]/,
          );
        } else {
          assert.equal(diagnostics.length, 1, "retain the complete missing-ref diagnostic vector");
          assert.equal(unresolvedRef.length, 1);
          assert.deepEqual(unresolvedRef[0]?.range, rangeFor("ref", source.indexOf("ref(")));
          const hover = await session.request(
            "textDocument/hover",
            { textDocument: { uri }, position: rangeFor("tags", source.indexOf("{{ tags")).start },
            120_000,
          );
          assert.doesNotMatch(
            hoverToText(hover as Parameters<typeof hoverToText>[0]),
            /const tags: string\[\]|Ref<string\[\]/,
            "a sibling declaration must not mint the missing ref's Vue type",
          );
        }
        session.notify("textDocument/didClose", { textDocument: { uri } });
      } finally {
        if (initialized) await session.shutdown();
        else await session.kill().catch(() => undefined);
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    },
  );
}

type Range = {
  start: { line: number; character: number };
  end: { line: number; character: number };
};
function rangeFor(symbol: string, near: number): Range {
  const start = source.indexOf(symbol, near);
  assert.ok(start >= 0);
  return {
    start: offsetToPosition(source, start),
    end: offsetToPosition(source, start + symbol.length),
  };
}
async function assertHover(session: LspSession, uri: string, range: Range, expected: RegExp) {
  const result = await session.request(
    "textDocument/hover",
    { textDocument: { uri }, position: range.start },
    120_000,
  );
  assert.deepEqual((result as { range?: Range } | null)?.range, range);
  const text = hoverToText(result as Parameters<typeof hoverToText>[0]);
  assert.match(text, /^\x60\x60\x60typescript\n/);
  assert.match(text, expected);
  assert.doesNotMatch(text, /\bany\b|Ref<unknown>|MaybeRef<unknown>/);
}
function linkVuePackage(directory: string) {
  const vue = [path.join(root, "node_modules/vue"), path.join(root, "tests/node_modules/vue")].find(
    (p) => fs.existsSync(p),
  );
  assert.ok(vue, "actual Vue package is required");
  const modules = path.join(directory, "node_modules");
  fs.mkdirSync(modules, { recursive: true });
  fs.symlinkSync(vue, path.join(modules, "vue"), process.platform === "win32" ? "junction" : "dir");
  const namespace = path.join(path.dirname(vue), "@vue");
  if (fs.existsSync(namespace))
    fs.symlinkSync(
      namespace,
      path.join(modules, "@vue"),
      process.platform === "win32" ? "junction" : "dir",
    );
}
