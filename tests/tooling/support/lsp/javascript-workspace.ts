// #8099: the actual source server edits the real package-linked JS SFC, with closed raw capture.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { LspSession } from "./session.ts";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import {
  argumentMessage,
  missingMessage,
  position,
} from "../../../../tools/support/compat/nuxt/javascript-workspace-cli.mjs";
import { save, sha } from "../../../../tools/support/compat/nuxt/javascript-workspace-project.mjs";

type Context = { project: string; artifacts: string; cohort: { nuxt?: string } };
const lintHelp =
  "props, data, computed, methods, setup, and inject share the component instance namespace; give each member a unique name.";
export async function editorProducts(root: string, context: Context, binary: string) {
  const main = path.join(
    context.project,
    "apps",
    context.cohort.nuxt ? "nuxt/src/pages/index.vue" : "vite/src/App.vue",
  );
  const uri = pathToFileURL(main).href;
  const original = fs.readFileSync(main, "utf8");
  const captures = path.join(root, "target/differential/lsp-sessions");
  fs.mkdirSync(captures, { recursive: true });
  for (const native of [false, true]) {
    const before = fs.readdirSync(captures);
    let session: LspSession | undefined;
    let initialization: unknown;
    const rows: unknown[] = [],
      notifications: unknown[] = [],
      responses: unknown[] = [];
    const errors: string[] = [];
    const remember = (error: unknown) =>
      errors.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
    const prefix =
      "<script>\nexport default {\n  computed: { value() { return 1; } },\n  methods: { value() {} }\n};\n</script>\n\n";
    const lintSource = prefix + original;
    const at = position(lintSource, "value() {}");
    const lintDiagnostic = {
      range: { start: at, end: { line: at.line, character: at.character + 5 } },
      severity: 1,
      code: "script/no-dupe-keys",
      codeDescription: { href: "https://eslint.vuejs.org/rules/script/no-dupe-keys.html" },
      source: "vize/lint",
      message: `Duplicated key 'value'\n\nHelp: ${lintHelp}`,
    };
    const nativeCases = [
      { id: "open", source: original },
      {
        id: "wrong-argument",
        source: original.replace("formatTotal(count.value)", 'formatTotal("two")'),
        needle: '"two"',
        code: 2345,
        message: argumentMessage,
      },
      { id: "repair-argument", source: original },
      {
        id: "missing-module",
        source: original.replace("@workspace/ui/BadgeCard.vue", "@workspace/ui/Missing.vue"),
        needle: '"@workspace/ui/Missing.vue"',
        code: 2307,
        message: missingMessage,
      },
      { id: "repair-module", source: original },
    ];
    const lintCases = [
      { id: "open", source: original },
      { id: "duplicate-options-key", source: lintSource },
      { id: "repair", source: original },
    ];
    const cases = native ? nativeCases : lintCases;
    const expectedPublications = [];
    try {
      session = new LspSession({ repoRoot: root, binary });
      session.notificationObservers.push((method, params) =>
        notifications.push({ method, params }),
      );
      session.responseObservers.push((message) => responses.push(message));
      initialization = await session.initialize(context.project, {
        editor: true,
        typecheck: native,
        lint: !native,
      });
      for (const [index, testCase] of cases.entries()) {
        const version = index + 1;
        const changed = testCase as (typeof nativeCases)[number];
        const diagnostic =
          native && changed.code
            ? (() => {
                const start = position(changed.source, changed.needle!);
                return {
                  range: {
                    start,
                    end: { line: start.line, character: start.character + changed.needle!.length },
                  },
                  severity: 1,
                  code: changed.code,
                  source: "vize/types",
                  message: changed.message,
                };
              })()
            : undefined;
        const expected = {
          uri,
          version,
          diagnostics: diagnostic ? [diagnostic] : !native && index === 1 ? [lintDiagnostic] : [],
        };
        const started = performance.now();
        if (index === 0)
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version, text: testCase.source },
          });
        else
          session.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text: testCase.source }],
          });
        const packet = await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) =>
            (params as { uri?: string; version?: number }).uri === uri &&
            (params as { version?: number }).version === version,
        );
        rows.push({
          id: testCase.id,
          native,
          source: testCase.source,
          expected,
          actual: packet,
          wallMs: performance.now() - started,
        });
        expectedPublications.push(expected);
        save(context.artifacts, `lsp-${native ? "native" : "lint"}-live.json`, {
          initialization,
          rows,
          notifications,
          responses,
          stderr: session.stderrText,
        });
        assert.deepEqual(packet, expected);
      }
      session.notify("textDocument/didClose", { textDocument: { uri } });
      const closed = await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) =>
          (params as { uri?: string }).uri === uri && !Object.hasOwn(params as object, "version"),
      );
      const expectedClosed = { uri, diagnostics: [] };
      expectedPublications.push(expectedClosed);
      assert.deepEqual(closed, expectedClosed);
    } catch (error) {
      remember(error);
    } finally {
      try {
        await session?.shutdown();
      } catch (error) {
        remember(error);
      }
      save(context.artifacts, `lsp-${native ? "native" : "lint"}.json`, {
        initialization,
        rows,
        notifications,
        responses,
        errors,
        expectedPublications,
        stderr: session?.stderrText ?? null,
      });
    }
    const added = fs.readdirSync(captures).filter((name) => !before.includes(name));
    assert.equal(added.length, 1, "exactly one owned real stdio session");
    const directory = path.join(captures, added[0]);
    let observation;
    for (let attempt = 0; attempt < 100; attempt++) {
      observation = JSON.parse(fs.readFileSync(path.join(directory, "observation.json"), "utf8"));
      if (observation.state === "process-closed") break;
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    const wire: Record<string, unknown[]> = {};
    for (const side of ["client", "server", "stderr"]) {
      const bytes = fs.readFileSync(path.join(directory, `${side}.bin`));
      assert.equal(sha(bytes), observation.streams[side].sha256);
      assert.equal(bytes.length, observation.streams[side].observedBytes);
      assert.equal(observation.streams[side].truncated, false);
      if (side !== "stderr") wire[side] = decodeFrames(bytes).messages;
    }
    fs.cpSync(directory, path.join(context.artifacts, `lsp-${native ? "native" : "lint"}-raw`), {
      recursive: true,
    });
    save(context.artifacts, `lsp-${native ? "native" : "lint"}-decoded.json`, wire);
    assert.equal(observation.state, "process-closed");
    assert.deepEqual(observation.process, { exitStatus: 0, signal: null, error: null });
    assert.equal(observation.sourceRevision, process.env.GITHUB_SHA);
    const server = wire.server as Array<{
      method?: string;
      params?: unknown;
      id?: number;
      result?: unknown;
      error?: unknown;
    }>;
    assert.deepEqual(
      server
        .filter((row) => row.method === "textDocument/publishDiagnostics")
        .map((row) => row.params),
      expectedPublications,
    );
    assert.deepEqual(
      server
        .filter((row) => row.method != null)
        .map((row) => ({ method: row.method, params: row.params })),
      notifications,
    );
    assert.deepEqual(
      server.filter((row) => row.id != null && row.method == null),
      responses,
    );
    assert.equal(responses.length, 2);
    assert.equal((responses[1] as { result: unknown }).result, null);
    assert.deepEqual(errors, []);
    assert.equal(rows.length, cases.length);
  }
  assert.equal(
    fs.readFileSync(main, "utf8"),
    original,
    "unsaved editor changes never mutate authored disk source",
  );
}
