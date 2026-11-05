import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";

import {
  assertStableProcessTreeSample,
  linuxProcessBirthTicks,
  processTreeRss,
} from "../performance/support/process-metrics.ts";
import { wordRanges } from "../performance/support/dual-lsp-fixture.ts";
import { statistics } from "../performance/support/current-baseline/wire.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
  WorkspaceEdit,
} from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  snapshotSessionCaptures,
  verifySessionCaptures,
} from "./support/lsp/verify-session-captures.ts";
import {
  anchoredSymbolRange,
  locations,
  sortLocations,
} from "./support/real-project-lsp-authored-utils.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

// One complete authored packet, repeated without editing the queried file.
// Timings and boundary RSS are observations; neither changes the semantic oracle.
const CHAIN_LENGTH = 96;
const CYCLES = 5;
const RENAME_TO = "renamedChildValue";
const A = `<script setup lang="ts">
import Child, { childValue } from './Child.vue'
const marker = 'é😀'; const requiredNumber: number = childValue; const observedValue = childValue
</script>
<template><Child />{{ requiredNumber }}{{ marker }}</template>
`;
const CHILD = `<script lang="ts">
export const childValue: number = 1
</script>
<script setup lang="ts">
import Next from './Chain000.vue'
</script>
<template><Next /></template>
`;
const CHILD_CHANGED = CHILD.replace("childValue: number = 1", "childValue: string = 'changed'");
const B = `<script setup lang="ts">
const unrelated: number = 1
</script>
<template>{{ unrelated }}</template>
`;
const B_BROKEN = B.replace("unrelated: number = 1", "unrelated: number = 'broken'");
const INITIALIZATION = {
  editor: true,
  lint: false,
  typecheck: true,
  hover: true,
  definition: true,
  references: true,
  rename: true,
};
const hash = (body: string | Buffer): string => createHash("sha256").update(body).digest("hex");
type Packet = Record<"hover" | "definition" | "references" | "rename", unknown>;

function mismatch(source: string, name: string): LspDiagnostic[] {
  return [
    {
      code: 2322,
      message: "Type 'string' is not assignable to type 'number'.",
      range: anchoredSymbolRange(source, `const ${name}: number`, name, name),
      severity: 1,
      source: "vize/types",
    },
  ];
}

test(
  "unrelated overlays preserve complete answers while imported overlays invalidate them",
  { timeout: 240_000 },
  async (t) => {
    if (
      process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1" ||
      !process.env.VIZE_EDITOR_CONFIG_VUE_ROOT
    ) {
      t.skip("exact-source overlay graph qualification runs in the native LSP lane");
      return;
    }
    const runtime = resolveTypecheckRuntime(root);
    assert.ok(runtime, "the locked native TypeScript runtime is required");
    const vue = process.env.VIZE_EDITOR_CONFIG_VUE_ROOT;
    assert.ok(vue, "the native LSP lane's pinned Vue provider is required");
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-overlay-graph-"));
    const outputRoot = path.resolve(
      process.env.VIZE_LSP_OVERLAY_GRAPH_CAPTURE ??
        path.join(root, "target/differential/lsp-overlay-graph-reuse"),
    );
    fs.mkdirSync(outputRoot, { recursive: true });
    const output = fs.mkdtempSync(path.join(outputRoot, "run-"));
    const report: Record<string, unknown> = {
      status: "running",
      chainLength: CHAIN_LENGTH,
      cycles: CYCLES,
      scope:
        "fresh process and warm repeated packets; filesystem caches uncontrolled; RSS only at named boundaries, no peak or PSS claim",
      testSource: {
        path: path.relative(root, fileURLToPath(import.meta.url)),
        sha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))),
      },
      timings: [],
      rss: [],
      packets: [],
      publications: [],
    };
    const persist = () =>
      fs.writeFileSync(path.join(output, "report.json"), JSON.stringify(report, null, 2) + "\n");
    persist();
    const inputs: Record<string, string> = {
      "src/A.vue": A,
      "src/Child.vue": CHILD,
      "src/B.vue": B,
      "package.json": JSON.stringify({ private: true, type: "module" }),
      "tsconfig.json": JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          skipLibCheck: true,
        },
        include: ["src/**/*.vue"],
      }),
    };
    for (let index = 0; index < CHAIN_LENGTH; index += 1) {
      const name = `src/Chain${String(index).padStart(3, "0")}.vue`;
      inputs[name] =
        index + 1 === CHAIN_LENGTH
          ? "<template><span>terminal</span></template>\n"
          : `<script setup lang="ts">\nimport Next from './Chain${String(index + 1).padStart(3, "0")}.vue'\n</script>\n<template><Next /></template>\n`;
    }
    const sessions = new Set<LspSession>();
    let primaryFailure: unknown;
    let captureBefore: ReturnType<typeof snapshotSessionCaptures> | undefined;
    try {
      for (const [file, source] of Object.entries(inputs)) {
        fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
        fs.writeFileSync(path.join(directory, file), source);
      }
      fs.mkdirSync(path.join(directory, "node_modules"));
      fs.symlinkSync(
        fs.realpathSync(vue),
        path.join(directory, "node_modules/vue"),
        process.platform === "win32" ? "junction" : "dir",
      );
      writeVueConfigFixture(directory, {
        lsp: INITIALIZATION,
        typeChecker: { corsaPath: fs.realpathSync(runtime) },
      });
      inputs["vite.config.mjs"] = fs.readFileSync(path.join(directory, "vite.config.mjs"), "utf8");
      report.inputs = inputs;
      report.overlayInputs = { unrelatedBroken: B_BROKEN, childChanged: CHILD_CHANGED };
      report.provider = {
        runtime: fs.realpathSync(runtime),
        runtimeSha256: hash(fs.readFileSync(runtime)),
        runtimeVersion: execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(),
        vue: fs.realpathSync(vue),
        vuePackage: JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8")),
      };
      captureBefore = snapshotSessionCaptures(root);
      report.source = captureBefore.binary;
      report.rawCaptureBefore = captureBefore;
      persist();
      const uris = Object.fromEntries(
        ["A", "Child", "B"].map((name) => [
          name,
          pathToFileURL(path.join(directory, `src/${name}.vue`)).href,
        ]),
      );
      const position = anchoredSymbolRange(
        A,
        "number = childValue",
        "childValue",
        "A imported value",
      ).start;
      const hoverPosition = anchoredSymbolRange(
        A,
        "const observedValue =",
        "observedValue",
        "inferred value hover",
      ).start;
      const expectedSites = sortLocations([
        ...wordRanges(A, "childValue").map((range) => ({ uri: uris.A, range })),
        ...wordRanges(CHILD, "childValue").map((range) => ({ uri: uris.Child, range })),
      ]);

      async function measure<T>(session: string, label: string, run: () => Promise<T>): Promise<T> {
        const started = performance.now();
        try {
          const result = await run();
          (report.timings as unknown[]).push({
            session,
            label,
            elapsedMs: performance.now() - started,
            status: "completed",
          });
          persist();
          return result;
        } catch (error) {
          (report.timings as unknown[]).push({
            session,
            label,
            elapsedMs: performance.now() - started,
            status: "failed",
            error: String(error),
          });
          persist();
          throw error;
        }
      }
      function observe(session: LspSession, label: string) {
        const history: PublishDiagnosticsParams[] = [];
        session.notificationObservers.push((method, params) => {
          if (
            method === "textDocument/publishDiagnostics" &&
            typeof params === "object" &&
            params !== null
          ) {
            history.push(params as PublishDiagnosticsParams);
            (report.publications as unknown[]).push({
              session: label,
              ordinal: history.length - 1,
              params,
            });
            persist();
          }
        });
        const rootTicks = linuxProcessBirthTicks(session.processId);
        const births = new Map<number, string>();
        return {
          history,
          async publication(
            uri: string,
            version: number,
            diagnostics: LspDiagnostic[],
            after: number,
          ) {
            const result = await session.waitForNotification(
              "textDocument/publishDiagnostics",
              (params) =>
                isDiagnosticsForUri(params, uri) &&
                params.version === version &&
                history.indexOf(params) >= after &&
                isDeepStrictEqual(params.diagnostics, diagnostics),
              120_000,
            );
            assert.deepEqual(result, { uri, version, diagnostics });
            return result;
          },
          rss(boundary: string) {
            const sample = processTreeRss(session.processId, {
              stableIdentity: true,
              expectedRootBirthTicks: rootTicks,
            });
            if (process.platform === "linux")
              assert.ok(sample, "a complete stable process-tree RSS observation is required");
            if (sample) assertStableProcessTreeSample(session.processId, sample, births);
            (report.rss as unknown[]).push({
              session: label,
              boundary,
              pid: session.processId,
              sample,
            });
            persist();
          },
        };
      }
      function open(session: LspSession, name: "A" | "Child" | "B", text: string, version = 1) {
        session.notify("textDocument/didOpen", {
          textDocument: { uri: uris[name], languageId: "vue", version, text },
        });
      }
      function change(session: LspSession, name: "B" | "Child", text: string, version: number) {
        session.notify("textDocument/didChange", {
          textDocument: { uri: uris[name], version },
          contentChanges: [{ text }],
        });
      }
      async function packet(session: LspSession, label: string): Promise<Packet> {
        const common = { textDocument: { uri: uris.A }, position };
        const result: Packet = {
          hover: await measure(label, "hover", () =>
            session.request("textDocument/hover", {
              textDocument: { uri: uris.A },
              position: hoverPosition,
            }),
          ),
          definition: await measure(label, "definition", () =>
            session.request("textDocument/definition", common),
          ),
          references: await measure(label, "references", () =>
            session.request("textDocument/references", {
              ...common,
              context: { includeDeclaration: true },
            }),
          ),
          rename: await measure(label, "rename", () =>
            session.request("textDocument/rename", { ...common, newName: RENAME_TO }),
          ),
        };
        (report.packets as unknown[]).push({ label, result });
        persist();
        assert.ok(
          typeof result.hover === "object" && result.hover !== null,
          "typed hover must answer",
        );
        assert.deepEqual(locations(result.definition, "imported definition must answer"), [
          { uri: uris.Child, range: wordRanges(CHILD, "childValue")[0] },
        ]);
        assert.deepEqual(
          sortLocations(locations(result.references, "imported references must answer")),
          expectedSites,
        );
        const rename = result.rename as WorkspaceEdit | null;
        assert.ok(
          rename && rename.changes,
          "rename must retain the complete authored changes vector",
        );
        assert.deepEqual(Object.keys(rename).sort(), ["changes"]);
        assert.deepEqual(rename, {
          changes: {
            [uris.A]: wordRanges(A, "childValue").map((range, index) => ({
              newText: index === 0 ? `childValue as ${RENAME_TO}` : RENAME_TO,
              range,
            })),
          },
        });
        return result;
      }

      const spawned = performance.now();
      const session = new LspSession();
      sessions.add(session);
      const observed = observe(session, "warm");
      await measure("warm", "initialize", () => session.initialize(directory, INITIALIZATION));
      open(session, "Child", CHILD);
      open(session, "B", B);
      open(session, "A", A);
      for (const name of ["Child", "B", "A"]) await observed.publication(uris[name], 1, [], 0);
      (report.timings as unknown[]).push({
        session: "warm",
        label: "fresh-process-through-open-diagnostics",
        elapsedMs: performance.now() - spawned,
        status: "passed",
      });
      const baseline = await packet(session, "before-unrelated-edits");
      observed.rss("cold-prepared");
      let bVersion = 1;
      for (let cycle = 0; cycle < CYCLES; cycle += 1) {
        for (const [phase, source, diagnostics] of [
          ["break", B_BROKEN, mismatch(B_BROKEN, "unrelated")],
          ["repair", B, []],
        ] as const) {
          const cursor = observed.history.length;
          await measure("warm", `B-${cycle}-${phase}-through-diagnostics`, async () => {
            change(session, "B", source, ++bVersion);
            await observed.publication(uris.B, bVersion, [...diagnostics], cursor);
          });
          assert.deepEqual(await packet(session, `B-${cycle}-${phase}`), baseline);
          for (const publication of observed.history
            .slice(cursor)
            .filter((value) => value.uri === uris.A)) {
            assert.deepEqual(publication, { uri: uris.A, version: 1, diagnostics: [] });
          }
        }
        observed.rss(`unrelated-cycle-${cycle}`);
      }

      const childCursor = observed.history.length;
      await measure("warm", "imported-child-edit-through-A-diagnostics", async () => {
        change(session, "Child", CHILD_CHANGED, 2);
        await observed.publication(uris.Child, 2, [], childCursor);
        await observed.publication(uris.A, 1, mismatch(A, "requiredNumber"), childCursor);
      });
      const invalidated = await packet(session, "after-imported-child-edit");
      assert.notDeepEqual(
        invalidated.hover,
        baseline.hover,
        "an imported type edit must change A's complete hover",
      );
      observed.rss("imported-edit");
      await session.shutdown();
      await session.waitForClosed();
      assert.equal(session.exitedGracefully, true);
      sessions.delete(session);

      // Same authored bytes and unsaved overlays, independent fresh editor state.
      // Whole answers must agree; a stale but stable warm answer cannot pass.
      const coldStarted = performance.now();
      const cold = new LspSession();
      sessions.add(cold);
      const independent = observe(cold, "independent-changed-overlay");
      await measure("independent-changed-overlay", "initialize", () =>
        cold.initialize(directory, INITIALIZATION),
      );
      open(cold, "Child", CHILD_CHANGED, 2);
      open(cold, "B", B, bVersion);
      open(cold, "A", A);
      await independent.publication(uris.Child, 2, [], 0);
      await independent.publication(uris.B, bVersion, [], 0);
      await independent.publication(uris.A, 1, mismatch(A, "requiredNumber"), 0);
      (report.timings as unknown[]).push({
        session: "independent-changed-overlay",
        label: "fresh-process-through-open-diagnostics",
        elapsedMs: performance.now() - coldStarted,
        status: "passed",
      });
      assert.deepEqual(await packet(cold, "independent-changed-overlay"), invalidated);
      independent.rss("cold-changed-overlay");
      await cold.shutdown();
      await cold.waitForClosed();
      assert.equal(cold.exitedGracefully, true);
      sessions.delete(cold);
      for (const [file, source] of Object.entries(inputs))
        assert.equal(
          fs.readFileSync(path.join(directory, file), "utf8"),
          source,
          `unsaved edits changed disk: ${file}`,
        );
      const timings = report.timings as Array<{
        session: string;
        label: string;
        elapsedMs: number;
      }>;
      report.qualifiedWarm = Object.fromEntries(
        ["hover", "definition", "references", "rename"].map((method) => [
          method,
          statistics(
            timings
              .filter((row) => row.session.startsWith("B-") && row.label === method)
              .map((row) => row.elapsedMs),
          ),
        ]),
      );
      report.status = "passed";
    } catch (error) {
      primaryFailure = error;
      report.status = "failed";
      report.error =
        error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
    } finally {
      const cleanupErrors: string[] = [];
      for (const session of sessions) {
        try {
          await session.shutdown();
          await session.waitForClosed();
        } catch (error) {
          cleanupErrors.push(String(error));
          await session.kill().catch((killError) => cleanupErrors.push(String(killError)));
        }
      }
      report.cleanupErrors = cleanupErrors;
      try {
        assert.ok(captureBefore, "the raw source-bound wire snapshot must precede the sessions");
        report.rawCapture = verifySessionCaptures(root, captureBefore);
        assert.deepEqual(cleanupErrors, []);
      } catch (error) {
        primaryFailure ??= error;
        report.status = "failed";
        report.captureError = String(error);
      }
      persist();
      fs.rmSync(directory, { recursive: true, force: true });
      t.diagnostic(`complete overlay graph report: ${output}`);
    }
    if (primaryFailure) throw primaryFailure;
  },
);
