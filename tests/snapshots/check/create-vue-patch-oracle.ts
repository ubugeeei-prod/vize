import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  repoRoot,
  symlinkDirectory,
  withPinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  completionLabels,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";

const appPath = "template/bare/typescript/src/App.vue";

test("create-vue clean, broken, and repaired patches agree across check and LSP", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: [appPath] },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write(
        "tsconfig.json",
        `${JSON.stringify(
          {
            compilerOptions: {
              lib: ["ES2022", "DOM", "DOM.Iterable"],
              module: "ESNext",
              moduleResolution: "bundler",
              noEmit: true,
              skipLibCheck: true,
              strict: true,
              target: "ES2022",
            },
            include: ["template/bare/typescript/src/**/*.vue"],
          },
          null,
          2,
        )}\n`,
      );
      fixture.write(
        "vize.config.json",
        `${JSON.stringify(
          {
            lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
            typeChecker: { corsaPath },
          },
          null,
          2,
        )}\n`,
      );

      fixture.applyExactPatch(
        appPath,
        '<script setup lang="ts"></script>',
        `<script setup lang="ts">\nconst count: number = 1\nconst label = 'ready'\n</script>`,
      );
      const cleanSource = fixture.applyExactPatch(
        appPath,
        "  <h1>You did it!</h1>",
        "  <h1>{{ label }}</h1>\n  <p>{{ count }}</p>",
      );
      const appFile = fixture.resolve(appPath);
      const appUri = pathToFileURL(appFile).href;

      assertCleanCheck(runCheck(fixture.workspaceDir, corsaPath));

      const session = new LspSession();
      try {
        await session.initialize(fixture.workspaceDir, {
          completion: true,
          editor: true,
          hover: true,
          lint: false,
          typecheck: true,
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
        });
        const cleanPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, appUri) && params.version === 1,
        )) as PublishDiagnosticsParams;
        assert.deepEqual(cleanPublish.diagnostics, [], JSON.stringify(cleanPublish.diagnostics));

        const completion = await session.request("textDocument/completion", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf("{{ count }}") + "{{ cou".length,
          ),
        });
        const labels = completionLabels(completion);
        assert.ok(labels.includes("count"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: appUri },
          position: offsetToPosition(
            cleanSource,
            cleanSource.indexOf("{{ count }}") + "{{ count".length,
          ),
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /count/);
        assert.match(hoverText, /number/i);

        const brokenSource = fixture.applyExactPatch(
          appPath,
          "const count: number = 1",
          "const count: number = 'broken'",
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 2 },
          contentChanges: [{ text: brokenSource }],
        });
        const brokenPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, appUri) &&
            params.diagnostics.some((diagnostic) =>
              diagnostic.message?.includes("not assignable to type 'number'"),
            ),
        )) as PublishDiagnosticsParams;
        assert.equal(brokenPublish.version, 2, JSON.stringify(brokenPublish));
        const typeError = brokenPublish.diagnostics.find((diagnostic) =>
          diagnostic.message?.includes("not assignable to type 'number'"),
        );
        assert.ok(typeError, JSON.stringify(brokenPublish.diagnostics));
        assert.equal(typeError.source, "vize/types");
        assert.equal(typeError.severity, 1);
        assert.equal(String(typeError.code).replace(/^TS/, ""), "2322");
        assert.match(typeError.message ?? "", /string.*not assignable.*number/i);
        assert.deepEqual(typeError.range?.start, { line: 1, character: 6 });
        assert.deepEqual(typeError.range?.end, { line: 1, character: 11 });

        const brokenCheck = runCheck(fixture.workspaceDir, corsaPath);
        assert.equal(brokenCheck.status, 1, brokenCheck.stderr);
        assert.equal(brokenCheck.report.errorCount, 1, JSON.stringify(brokenCheck.report));
        assert.match(
          brokenCheck.report.files[0]?.diagnostics.join("\n") ?? "",
          /TS2322.*string.*not assignable.*number/i,
        );

        const repairedSource = fixture.applyExactPatch(
          appPath,
          "const count: number = 'broken'",
          "const count: number = 2",
        );
        session.notify("textDocument/didChange", {
          textDocument: { uri: appUri, version: 3 },
          contentChanges: [{ text: repairedSource }],
        });
        const repairedPublish = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            isDiagnosticsForUri(params, appUri) &&
            params.version === 3 &&
            params.diagnostics.every(
              (diagnostic) => !diagnostic.message?.includes("not assignable to type 'number'"),
            ),
        )) as PublishDiagnosticsParams;
        assert.deepEqual(
          repairedPublish.diagnostics,
          [],
          JSON.stringify(repairedPublish.diagnostics),
        );
        assertCleanCheck(runCheck(fixture.workspaceDir, corsaPath));
      } finally {
        await session.shutdown();
      }
    },
  );
});

type CheckReport = {
  errorCount: number;
  fileCount: number;
  files: Array<{ file: string; diagnostics: string[] }>;
};

function assertCleanCheck(result: ReturnType<typeof runCheck>): void {
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.report.fileCount, 1, JSON.stringify(result.report));
  assert.equal(result.report.errorCount, 0, JSON.stringify(result.report));
  assert.deepEqual(result.report.files[0]?.diagnostics, []);
}

function runCheck(
  workspaceDir: string,
  corsaPath: string,
): { status: number | null; stderr: string; report: CheckReport } {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "check", appPath, "--format", "json", "--quiet", "--corsa-path", corsaPath],
    { cwd: workspaceDir, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, report: JSON.parse(result.stdout) };
}

function resolveVizeCommand(): string[] {
  const candidates = [
    process.env.VIZE_TEST_BIN,
    path.join(repoRoot, "target/debug/vize"),
    path.join(repoRoot, "target/ci/vize"),
    path.join(repoRoot, "target/release/vize"),
    "vize",
  ].filter((candidate): candidate is string => Boolean(candidate));
  for (const candidate of candidates) {
    if (spawnSync(candidate, ["--version"], { cwd: repoRoot }).status === 0) return [candidate];
  }
  return ["cargo", "run", "-q", "-p", "vize", "--"];
}

function resolveTsgoBinary(): string {
  const candidates = [
    process.env.VIZE_TEST_TSGO,
    path.join(repoRoot, "../corsa-bind/.cache/tsgo"),
    path.join(repoRoot, "node_modules/.bin/tsgo"),
    path.join(repoRoot, "tests/node_modules/.bin/tsgo"),
  ].filter((candidate): candidate is string => Boolean(candidate));
  const binary = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(binary, "tsgo binary is required for real-world patch oracles");
  return binary;
}

function symlinkVueTypes(workspaceDir: string): void {
  const candidates = [
    path.join(repoRoot, "node_modules/vue"),
    path.join(repoRoot, "tests/node_modules/vue"),
  ];
  const vuePackage = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for real-world patch oracles");
  symlinkDirectory(vuePackage, path.join(workspaceDir, "node_modules/vue"));
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlinkDirectory(vueNamespace, path.join(workspaceDir, "node_modules/@vue"));
  }
}
