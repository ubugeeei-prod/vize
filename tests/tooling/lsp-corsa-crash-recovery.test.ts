import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

// Corsa mid-session crash recovery (#3240, audit item 6 of #2971): the type
// checker backend (tsgo) is an external child process that can die at any
// time (OOM kill, crash, operator kill). When that happens mid-session the
// server must (a) not crash itself, (b) respawn the backend and republish
// correct typecheck diagnostics for the next edit within a bounded time, and
// (c) keep publishing monotonically-versioned, non-stale diagnostics. Two
// kill+recover cycles guard against one-shot recovery paths.

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

/**
 * PIDs of live descendants of `rootPid` whose command mentions tsgo/corsa.
 *
 * The backend may be reached through wrapper layers (`sh` shim, `node`
 * launcher, native binary), so the whole descendant tree is scanned rather
 * than only direct children. `ps -axo` output is BSD/procps compatible and
 * works on both macOS and Linux CI runners.
 */
function findCorsaDescendants(rootPid: number): number[] {
  const table = execFileSync("ps", ["-axo", "pid=,ppid=,command="], { encoding: "utf8" })
    .split("\n")
    .map((line) => line.trim().match(/^(\d+)\s+(\d+)\s+(.*)$/))
    .filter((match): match is RegExpMatchArray => match != null)
    .map((match) => ({ pid: Number(match[1]), ppid: Number(match[2]), command: match[3] }));

  const childrenByParent = new Map<number, Array<{ pid: number; command: string }>>();
  for (const entry of table) {
    const children = childrenByParent.get(entry.ppid) ?? [];
    children.push(entry);
    childrenByParent.set(entry.ppid, children);
  }

  const matches: number[] = [];
  const queue = [rootPid];
  while (queue.length > 0) {
    const pid = queue.shift() as number;
    for (const child of childrenByParent.get(pid) ?? []) {
      queue.push(child.pid);
      if (/tsgo|corsa/i.test(child.command)) {
        matches.push(child.pid);
      }
    }
  }
  return matches;
}

async function waitForCorsaDescendants(rootPid: number, timeoutMs: number): Promise<number[]> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const pids = findCorsaDescendants(rootPid);
    if (pids.length > 0) {
      return pids;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  return [];
}

function isProcessAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/**
 * A SIGKILLed child counts as dead once it is gone or a zombie: the server
 * only reaps the corpse when it retires the dead session, so `kill(pid, 0)`
 * alone would keep reporting the zombie as alive.
 */
function isProcessDead(pid: number): boolean {
  try {
    const state = execFileSync("ps", ["-o", "state=", "-p", String(pid)], {
      encoding: "utf8",
    }).trim();
    return state === "" || state.startsWith("Z");
  } catch {
    return true;
  }
}

async function killAndAwaitExit(pids: number[]): Promise<void> {
  for (const pid of pids) {
    try {
      process.kill(pid, "SIGKILL");
    } catch {
      // Already gone; the assertion below still verifies it exited.
    }
  }
  const deadline = Date.now() + 5000;
  while (!pids.every(isProcessDead) && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(
    pids.every(isProcessDead),
    `corsa backend processes should exit after SIGKILL: ${pids.join(", ")}`,
  );
}

/**
 * Each document revision references one unique undeclared identifier, so the
 * expected `vize/types` diagnostic ("Cannot find name '<marker>'") is
 * distinguishable per revision — a stale republish of the previous revision's
 * diagnostics cannot satisfy the next cycle's predicate.
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

function typeErrorFor(publish: PublishDiagnosticsParams, marker: string): boolean {
  return publish.diagnostics.some(
    (diagnostic) =>
      diagnostic.source === "vize/types" &&
      diagnostic.message?.includes(`Cannot find name '${marker}'`),
  );
}

test("vize lsp recovers typecheck diagnostics after the Corsa backend is killed", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the Corsa crash-recovery gate",
    "TypeScript 7/Corsa runtime not found; skipping Corsa crash-recovery test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-corsa-crash-recovery");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
    // `vue` module shim so the virtual TS preamble resolves without linking a
    // real node_modules into the fixture (mirrors the fallback shim used by
    // lsp-typecheck-template.test.ts and keeps every publish noise-free).
    fs.writeFileSync(
      path.join(workspaceDir, "src/vue-shim.d.ts"),
      `declare module "vue" {
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
`,
      "utf8",
    );
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
    const initialSource = appSource("missingOne");
    fs.writeFileSync(filePath, initialSource, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: initialSource },
    });
    const initialPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && typeErrorFor(params, "missingOne"),
    )) as PublishDiagnosticsParams;
    assert.equal(initialPublish.version, 1);

    // Two full kill+recover cycles: recovery must not be a one-shot affair.
    let previousMarker = "missingOne";
    let previousBackendPids: number[] = [];
    for (const [cycle, marker] of [
      [2, "missingTwo"],
      [3, "missingThree"],
    ] as const) {
      const backendPids = await waitForCorsaDescendants(session.processId, 15000);
      assert.ok(
        backendPids.length > 0,
        `cycle ${cycle}: expected a live tsgo/corsa descendant of the vize lsp process`,
      );
      if (previousBackendPids.length > 0) {
        assert.notDeepEqual(
          backendPids,
          previousBackendPids,
          `cycle ${cycle}: recovery must respawn a fresh backend process`,
        );
      }
      previousBackendPids = backendPids;

      await killAndAwaitExit(backendPids);

      const changedSource = appSource(marker);
      const editedAt = Date.now();
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: cycle },
        contentChanges: [{ text: changedSource }],
      });

      const recoveredPublish = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) =>
          isDiagnosticsForUri(params, uri) &&
          params.version === cycle &&
          typeErrorFor(params, marker),
      )) as PublishDiagnosticsParams;
      t.diagnostic(
        `cycle ${cycle}: recovered typecheck diagnostics ${Date.now() - editedAt}ms after the edit`,
      );

      // The recovered publish must be fresh: correct version, no diagnostics
      // left over from the pre-kill document state.
      assert.equal(recoveredPublish.version, cycle);
      assert.ok(
        !typeErrorFor(recoveredPublish, previousMarker),
        `cycle ${cycle}: stale "${previousMarker}" diagnostic republished: ${JSON.stringify(
          recoveredPublish.diagnostics,
        )}`,
      );
      previousMarker = marker;

      assert.ok(
        isProcessAlive(session.processId),
        `cycle ${cycle}: vize lsp server must survive the backend crash`,
      );
    }

    // The recovered backend also serves interactive requests again: hover
    // over the template interpolation resolves through the respawned Corsa
    // session (same coordinates the sibling typecheck suite exercises).
    const finalSource = appSource(previousMarker);
    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(finalSource, finalSource.lastIndexOf("count") + "count".length),
    })) as { contents?: unknown } | null;
    assert.match(
      hoverToText(hover),
      /count/,
      "hover should work against the respawned Corsa backend",
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
