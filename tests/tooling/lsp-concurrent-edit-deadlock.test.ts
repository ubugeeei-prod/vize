import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

// Concurrent-edit publish deadlock (#3315).
//
// tower-lsp polls up to four queued messages concurrently on the LSP's single
// executor thread. The Corsa diagnostics collector used to hold a `DashMap`
// shard read guard on the open-document store (`documents.get`) across its
// `.await` points. Whenever one of those awaits actually yielded — the
// background workspace `*.d.ts` declaration scan does, on a cold or
// invalidated cache — the next queued `didChange` ran on the same thread,
// took the shard *write* lock in `apply_changes`, and parked forever: the
// suspended reader can only release its guard by being polled on the very
// thread that is now parked. The server went permanently silent, publishing
// nothing and answering nothing, with normal memory use.
//
// Both windows below are exercised because they are independently reachable:
// the cold scan at the first diagnostics pass, and a mid-session rescan after
// a watched declaration file changes (the sustained-churn shape).

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

const VUE_SHIM = `declare module "vue" {
  export interface Ref<T = any> {
    value: T;
  }

  export interface ShallowRef<T = any> {
    value: T;
  }

  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>;
    $slots: Record<string, unknown>;
    $refs: Record<string, unknown>;
    $emit: (...args: any[]) => void;
  }

  export interface GlobalComponents {}
}
`;

/**
 * Each revision references one unique undeclared identifier, so the published
 * `vize/types` diagnostic identifies which revision produced it. A stale
 * republish therefore cannot masquerade as the awaited version's result.
 */
function appSource(marker: string): string {
  return `<script setup lang="ts">
const count: number = ${marker}
</script>

<template>
  <p>{{ count }}</p>
</template>
`;
}

function markersOf(publish: PublishDiagnosticsParams): string[] {
  return publish.diagnostics
    .filter((diagnostic) => diagnostic.source === "vize/types")
    .map((diagnostic) => /Cannot find name '(\w+)'/.exec(diagnostic.message ?? "")?.[1] ?? "")
    .filter((marker) => marker !== "");
}

test("vize lsp keeps publishing when edits race an in-flight diagnostics pass", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the concurrent-edit deadlock gate",
    "TypeScript 7/Corsa runtime not found; skipping concurrent-edit deadlock test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-concurrent-edit-deadlock");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  const declarationPath = path.join(workspaceDir, "src/vue-shim.d.ts");

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
    fs.writeFileSync(declarationPath, VUE_SHIM, "utf8");
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
    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });

    const filePath = path.join(workspaceDir, "src/App.vue");
    const uri = pathToFileURL(filePath).href;
    const publishes: Array<number | null> = [];
    session.notificationObservers.push((method, params) => {
      if (method !== "textDocument/publishDiagnostics") return;
      publishes.push((params as PublishDiagnosticsParams).version ?? null);
    });

    let version = 0;
    const edit = (marker: string, notification: "didOpen" | "didChange"): void => {
      version += 1;
      const text = appSource(marker);
      fs.writeFileSync(filePath, text, "utf8");
      session.notify(
        `textDocument/${notification}`,
        notification === "didOpen"
          ? { textDocument: { uri, languageId: "vue", version, text } }
          : { textDocument: { uri, version }, contentChanges: [{ text }] },
      );
    };

    /**
     * Awaits the publish for the newest source marker. The marker is unique per
     * revision, so it still identifies the final diagnostics even when a server
     * or client lane omits the optional publishDiagnostics version field.
     */
    const awaitFinalPublish = async (marker: string, label: string): Promise<void> => {
      const publish = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) =>
          isDiagnosticsForUri(params, uri) &&
          markersOf(params as PublishDiagnosticsParams).includes(marker),
        45_000,
      )) as PublishDiagnosticsParams;
      assert.deepEqual(
        markersOf(publish),
        [marker],
        `${label}: version ${version} must publish exactly its own revision's diagnostic`,
      );
      if (publish.version != null) {
        assert.equal(
          publish.version,
          version,
          `${label}: versioned publish must carry the latest document version`,
        );
      }
      t.diagnostic(`${label}: published v${version} (${publishes.length} publishes so far)`);
    };

    // Window 1: the very first diagnostics pass runs the cold declaration
    // scan, and the edit lands while it is suspended.
    edit("missingOne", "didOpen");
    edit("missingTwo", "didChange");
    await awaitFinalPublish("missingTwo", "cold scan window");

    // Window 2: the same suspension point mid-session. Changing a watched
    // declaration file invalidates the cached scan, so the next diagnostics
    // pass yields again while two further edits are already queued — the
    // sustained-churn shape from the churn-stress suite.
    for (const round of [1, 2, 3]) {
      fs.writeFileSync(declarationPath, `${VUE_SHIM}// round ${round}\n`, "utf8");
      session.notify("workspace/didChangeWatchedFiles", {
        changes: [{ uri: pathToFileURL(declarationPath).href, type: 2 }],
      });
      edit(`churnBroken${round}`, "didChange");
      edit(`churnFinal${round}`, "didChange");
      await awaitFinalPublish(`churnFinal${round}`, `churn round ${round}`);
    }

    // A wedged executor thread also stops answering requests, so a successful
    // round-trip proves the thread is still running the message loop.
    const symbols = await session.request("textDocument/documentSymbol", {
      textDocument: { uri },
    });
    assert.ok(
      Array.isArray(symbols),
      `server must still answer requests: ${JSON.stringify(symbols)}`,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});
