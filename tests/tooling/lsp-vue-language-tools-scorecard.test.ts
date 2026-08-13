import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "yaml";

import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
import {
  budgetRegistryPath,
  loadLspIncrementalBudget,
} from "../performance/support/incremental-metrics.ts";
import {
  completionLabels,
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspDiagnostic, LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const scorecardPath = "tests/_fixtures/maestro-vue-language-tools-scorecard.json";

type Evidence = {
  file: string;
  contains: string[];
};

type Oracle = {
  id: string;
  summary: string;
  evidence: Evidence[];
};

type FeatureRow = {
  dimension: string;
  lspMethods: string[];
  mustInclude: Oracle[];
  mustExclude: Oracle[];
};

type EditorRow = {
  editor: string;
  coverage: string;
  ciJob: string;
  task: string;
  mustInclude: string[];
  mustExclude: string[];
  evidence: Evidence[];
};

type LatencyBudgetRow = {
  fixtureId: string;
  suite: string;
  budgetSource: string;
  ciJob: string;
  ciStep: string;
  completionLane: string;
  hoverLane: string;
  diagnosticsToStableLanes: string[];
};

type Scorecard = {
  schemaVersion: number;
  trackingIssue: number;
  baseline: {
    name: string;
    server: string;
    versionEvidence: Evidence[];
  };
  featureMatrix: FeatureRow[];
  editorBreadth: EditorRow[];
  latencyBudgets: LatencyBudgetRow[];
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readScorecard(): Scorecard {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, scorecardPath), "utf8")) as Scorecard;
}

function taskCommand(name: string): string {
  const entry = testAndBenchmarkTasks[name] as { command?: string } | undefined;
  assert.ok(entry?.command, `missing task ${name}`);
  return entry.command;
}

// Evidence anchors name behavior (test titles, identifiers, diagnostic payloads),
// so whitespace is incidental: collapsing runs of whitespace on both sides keeps
// the release gate passing across reformats and line rewraps.
function collapseWhitespace(value: string): string {
  return value.replace(/\s+/g, " ");
}

function assertEvidence(evidence: Evidence[]): void {
  assert.ok(evidence.length > 0, "each oracle must point at executable evidence");
  for (const item of evidence) {
    const absolute = path.join(repoRoot, item.file);
    assert.ok(fs.existsSync(absolute), `missing evidence file ${item.file}`);
    const content = collapseWhitespace(fs.readFileSync(absolute, "utf8"));
    for (const required of item.contains) {
      assert.ok(
        content.includes(collapseWhitespace(required)),
        `${item.file} must contain ${JSON.stringify(required)}`,
      );
    }
  }
}

function normalizeItems(
  response: Array<Record<string, unknown>> | { items?: Array<Record<string, unknown>> } | null,
): Array<Record<string, unknown>> {
  if (response == null) return [];
  return Array.isArray(response) ? response : (response.items ?? []);
}

function rangeFor(source: string, needle: string): LspRange {
  const start = source.indexOf(needle);
  assert.notEqual(start, -1, `missing source anchor ${JSON.stringify(needle)}`);
  return {
    start: offsetToPosition(source, start),
    end: offsetToPosition(source, start + needle.length),
  };
}

function startsForEdits(
  edit: {
    changes?: Record<string, Array<{ range: LspRange; newText: string }>>;
  } | null,
  uri: string,
): Array<{ line: number; character: number }> {
  return (edit?.changes?.[uri] ?? [])
    .map((textEdit) => textEdit.range.start)
    .sort((left, right) => left.line - right.line || left.character - right.character);
}

test("Maestro scorecard fixture covers every Vue Language Server parity dimension", () => {
  const scorecard = readScorecard();
  assert.equal(scorecard.schemaVersion, 1);
  assert.equal(scorecard.trackingIssue, 3224);
  assert.equal(scorecard.baseline.name, "vuejs/language-tools");
  assert.equal(scorecard.baseline.server, "Vue Language Server");
  assertEvidence(scorecard.baseline.versionEvidence);

  const requiredDimensions = [
    "diagnostics",
    "completion",
    "hover",
    "definition",
    "references",
    "rename",
    "code-actions",
    "semantic-tokens",
    "inlay-hints",
    "document-features",
    "file-rename",
    "workspace-symbols",
  ];
  assert.deepEqual(
    scorecard.featureMatrix.map((row) => row.dimension),
    requiredDimensions,
  );

  for (const row of scorecard.featureMatrix) {
    assert.ok(row.lspMethods.length > 0, `${row.dimension} must name LSP methods`);
    assert.ok(row.mustInclude.length > 0, `${row.dimension} needs positive oracles`);
    assert.ok(row.mustExclude.length > 0, `${row.dimension} needs negative oracles`);
    for (const oracle of [...row.mustInclude, ...row.mustExclude]) {
      assert.match(oracle.id, /^[a-z0-9-]+$/);
      assert.ok(oracle.summary.length > 20, `${row.dimension}.${oracle.id} is too vague`);
      assertEvidence(oracle.evidence);
    }
  }
});

test("Maestro scorecard gates editor breadth through CI-backed artifacts", () => {
  const scorecard = readScorecard();
  const expectedEditors = ["VS Code", "Zed", "Neovim", "Helix", "Vim", "Emacs"];
  assert.deepEqual(
    scorecard.editorBreadth.map((row) => row.editor),
    expectedEditors,
  );

  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const hostAction = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const hostJob = workflowJobBody(workflow, "editor-host-smoke");
  assert.match(hostJob, /uses: \.\/\.github\/actions\/vscode-host-smoke/);
  const jobs = (parse(workflow) as { jobs: Record<string, { needs?: string[] | string }> }).jobs;
  const reportNeeds = jobs["test-report"]?.needs ?? [];
  assert.ok(
    (Array.isArray(reportNeeds) ? reportNeeds : [reportNeeds]).includes("editor-host-smoke"),
    "test-report must aggregate the editor-host-smoke gate",
  );

  for (const row of scorecard.editorBreadth) {
    assert.equal(row.ciJob, "editor-host-smoke");
    assert.ok(row.mustInclude.length > 0, `${row.editor} must state covered behavior`);
    assert.ok(row.mustExclude.length > 0, `${row.editor} must state forbidden overclaim/leakage`);
    assertEvidence(row.evidence);
    assert.ok(hostAction.includes(row.task), `${row.editor} task is not wired in host CI`);
    assert.ok(taskCommand(row.task).length > 0, `${row.editor} task command must be registered`);
  }

  assert.equal(
    scorecard.editorBreadth.filter((row) => row.coverage.includes("real-server")).length,
    5,
    "five editor integrations have real-server evidence; Emacs is explicitly packaged Eglot evidence",
  );
  assert.equal(
    scorecard.editorBreadth.find((row) => row.editor === "Emacs")?.coverage,
    "packaged-eglot-spec",
  );
});

test("Maestro scorecard names enforced Misskey and Vue Vben Admin latency budgets", () => {
  const scorecard = readScorecard();
  assert.deepEqual(
    scorecard.latencyBudgets.map((row) => row.fixtureId),
    ["misskey", "vue-vben-admin"],
  );

  const workflow = readRepoFile(".github", "actions", "check-vue-parity", "action.yml");
  for (const row of scorecard.latencyBudgets) {
    assert.equal(row.budgetSource, budgetRegistryPath);
    assert.equal(row.ciJob, "vue-parity");
    assert.ok(workflow.includes(row.ciStep), `${row.suite} must be run by vue-parity CI`);
    assert.ok(workflow.includes("test:performance:lsp-incremental"));

    const { fixtureId, budget } = loadLspIncrementalBudget(row.suite);
    assert.equal(fixtureId, row.fixtureId);
    for (const lane of [row.completionLane, row.hoverLane, ...row.diagnosticsToStableLanes]) {
      const budgetMs = budget.laneBudgetsMs[lane];
      assert.ok(
        Number.isSafeInteger(budgetMs) && budgetMs > 0,
        `${row.suite}.${lane} must have a positive enforced latency budget`,
      );
      assert.ok(
        budgetMs <= budget.laneHardTimeoutMs,
        `${row.suite}.${lane} must fit under the hard timeout`,
      );
    }
  }
});

test("Maestro scorecard executes representative must-include and must-exclude LSP oracles", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-vue-language-tools-scorecard");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let failure: unknown;

  try {
    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><button>{{ label }}</button></template>
`,
      "utf8",
    );
    fs.writeFileSync(path.join(workspaceDir, "useThing.mjs"), "export const useThing = () => 1\n");

    const source = `<script setup lang="ts">
import Child from './Child.vue'
import { computed, ref } from 'vue'
import { useThing } from './useThing'

const count = ref(0)
const doubled = computed(() => count.value * 2)
const message = ref('hello')
const items = [1, 2]

function submitMessage() {
  return useThing() + count.value
}
</script>

<template>
  <Child  :label="message"
    @click="
      () => {
        cou
      }
    "
  />
  <button :class="$style.primary">{{ message }}</button>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>

<style module>
.primary {}
</style>
`;
    const filePath = path.join(workspaceDir, "Scorecard.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    await session.initialize(workspaceDir, {
      codeActions: true,
      editor: true,
      fileRename: true,
      formatting: true,
      lint: true,
      typecheck: false,
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.diagnostics.some(
          (diagnostic) =>
            diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
        ) &&
        params.diagnostics.some(
          (diagnostic) =>
            diagnostic.source === "vize/lint" && diagnostic.code === "vue/no-multi-spaces",
        ),
    )) as PublishDiagnosticsParams;

    await t.test("diagnostics publish lint findings on authored template ranges", () => {
      const keyDiagnostic = publish.diagnostics.find(
        (diagnostic) =>
          diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
      );
      assert.ok(keyDiagnostic, JSON.stringify(publish.diagnostics));
      assert.deepEqual(keyDiagnostic.range, rangeFor(source, 'v-for="item in items"'));
    });

    await t.test("completion includes ranked bindings and excludes context leakage", async () => {
      const response = (await session.request("textDocument/completion", {
        textDocument: { uri },
        position: offsetToPosition(source, source.indexOf("        cou") + "        cou".length),
      })) as Array<Record<string, unknown>> | { items?: Array<Record<string, unknown>> } | null;
      const labels = completionLabels(response as never);
      assert.ok(labels.includes("count"), labels.join(", "));
      assert.ok(labels.includes("message"), labels.join(", "));
      for (const forbidden of ["v-if", "class", "@click"]) {
        assert.equal(
          labels.includes(forbidden),
          false,
          `${forbidden} leaked into ${labels.join(", ")}`,
        );
      }
      const count = normalizeItems(response).find((item) => item.label === "count");
      assert.equal(count?.sortText, "0count");
    });

    await t.test(
      "hover, definition, references, and rename stay on authored bindings",
      async () => {
        const messageUsageOffset = source.lastIndexOf("message }}</button>") + "message".length;
        const messageUsagePosition = offsetToPosition(source, messageUsageOffset);
        const declarationPosition = offsetToPosition(source, source.indexOf("message = ref"));

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri },
          position: messageUsagePosition,
        })) as { contents?: unknown } | null;
        const hoverText = hoverToText(hover);
        assert.match(hoverText, /message/);
        assert.match(hoverText, /Ref<string>|Template binding from script/);
        assert.doesNotMatch(hoverText, /Vue event listener/);

        const definition = await session.request("textDocument/definition", {
          textDocument: { uri },
          position: messageUsagePosition,
        });
        const location = firstLocation(definition as never);
        assert.equal(location.uri, uri);
        assert.deepEqual(location.range.start, declarationPosition);

        const references = (await session.request("textDocument/references", {
          textDocument: { uri },
          position: messageUsagePosition,
          context: { includeDeclaration: true },
        })) as Array<{ uri: string; range: LspRange }>;
        const referenceStarts = references.map((reference) => reference.range.start);
        assert.ok(
          referenceStarts.some(
            (start) =>
              start.line === declarationPosition.line &&
              start.character === declarationPosition.character,
          ),
          JSON.stringify(references),
        );
        assert.ok(
          referenceStarts.some(
            (start) =>
              start.line === messageUsagePosition.line &&
              start.character === messageUsagePosition.character - "message".length,
          ),
          JSON.stringify(references),
        );

        const directiveRename = await session.request("textDocument/prepareRename", {
          textDocument: { uri },
          position: offsetToPosition(source, source.indexOf("v-for") + 2),
        });
        assert.equal(directiveRename, null);

        const edit = (await session.request("textDocument/rename", {
          textDocument: { uri },
          position: messageUsagePosition,
          newName: "title",
        })) as { changes?: Record<string, Array<{ range: LspRange; newText: string }>> } | null;
        assert.deepEqual(startsForEdits(edit, uri), [
          declarationPosition,
          offsetToPosition(source, source.indexOf(':label="message"') + ':label="'.length),
          offsetToPosition(source, source.lastIndexOf("message }}</button>")),
        ]);
        assert.ok((edit?.changes?.[uri] ?? []).every((item) => item.newText === "title"));
      },
    );

    await t.test(
      "code actions include quick fixes and exclude unrelated requested kinds",
      async () => {
        const diagnostic = publish.diagnostics.find(
          (item): item is LspDiagnostic =>
            item.source === "vize/lint" && item.code === "vue/no-multi-spaces",
        );
        assert.ok(diagnostic, JSON.stringify(publish.diagnostics));
        const actions = (await session.request("textDocument/codeAction", {
          textDocument: { uri },
          range: diagnostic.range,
          context: { diagnostics: [diagnostic], only: ["quickfix"] },
        })) as Array<{ title?: string; kind?: string; isPreferred?: boolean }> | null;
        const titles = (actions ?? []).map((action) => action.title);
        assert.deepEqual(titles, [
          "Fix: Replace multiple spaces with single space",
          "Suppress with @vize:forget (vue/no-multi-spaces)",
        ]);
        assert.equal(actions?.[0]?.kind, "quickfix");
        assert.equal(actions?.[0]?.isPreferred, true);

        const refactors = await session.request("textDocument/codeAction", {
          textDocument: { uri },
          range: diagnostic.range,
          context: { diagnostics: [diagnostic], only: ["refactor", "source"] },
        });
        assert.equal(refactors, null);
      },
    );

    await t.test(
      "semantic tokens and inlay hints include signal without plain-text leakage",
      async () => {
        const semanticTokens = (await session.request("textDocument/semanticTokens/full", {
          textDocument: { uri },
        })) as { data?: number[] } | null;
        assert.ok(Array.isArray(semanticTokens?.data), JSON.stringify(semanticTokens));
        assert.equal(semanticTokens.data.length % 5, 0);
        assert.ok(semanticTokens.data.length > 0);

        const hints = (await session.request("textDocument/inlayHint", {
          textDocument: { uri },
          range: { start: { line: 0, character: 0 }, end: { line: 1000, character: 0 } },
        })) as Array<{ label: string | Array<{ value: string }> }> | null;
        const hintLabels = (hints ?? []).map((hint) =>
          typeof hint.label === "string"
            ? hint.label
            : hint.label.map((part) => part.value).join(""),
        );
        assert.ok(hintLabels.includes(": Ref<number>"), hintLabels.join(", "));
        assert.ok(hintLabels.includes(": ComputedRef<number>"), hintLabels.join(", "));
        assert.equal(hintLabels.includes(": Ref<boolean>"), false, hintLabels.join(", "));

        const noiseSource = `<template>
  <p>email dev@example.com and plain text v-if @click :class</p>
</template>
`;
        const noisePath = path.join(workspaceDir, "Noise.vue");
        const noiseUri = pathToFileURL(noisePath).href;
        fs.writeFileSync(noisePath, noiseSource, "utf8");
        session.notify("textDocument/didOpen", {
          textDocument: { uri: noiseUri, languageId: "vue", version: 1, text: noiseSource },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, noiseUri),
        );
        const noiseTokens = (await session.request("textDocument/semanticTokens/range", {
          textDocument: { uri: noiseUri },
          range: { start: { line: 1, character: 0 }, end: { line: 2, character: 0 } },
        })) as { data?: number[] } | null;
        assert.deepEqual(noiseTokens?.data, []);
      },
    );

    await t.test(
      "document features, file rename, and workspace symbols include and exclude targets",
      async () => {
        const symbols = (await session.request("textDocument/documentSymbol", {
          textDocument: { uri },
        })) as Array<{ name: string }> | null;
        const symbolNames = (symbols ?? []).map((symbol) => symbol.name);
        assert.deepEqual(symbolNames, ["template", "script setup", "style module=$style"]);

        const folding = await session.request("textDocument/foldingRange", {
          textDocument: { uri },
        });
        assert.ok(Array.isArray(folding), JSON.stringify(folding));

        const links = (await session.request("textDocument/documentLink", {
          textDocument: { uri },
        })) as Array<{ target?: string }> | null;
        const linkTargets = (links ?? []).map((link) =>
          path.basename(decodeURIComponent(new URL(link.target ?? "").pathname)),
        );
        assert.ok(linkTargets.includes("Child.vue"), linkTargets.join(", "));
        assert.ok(linkTargets.includes("useThing.mjs"), linkTargets.join(", "));
        assert.equal(linkTargets.includes("Missing.vue"), false, linkTargets.join(", "));

        const renamedChild = pathToFileURL(path.join(workspaceDir, "RenamedChild.vue")).href;
        const renameEdit = (await session.request("workspace/willRenameFiles", {
          files: [
            {
              oldUri: pathToFileURL(path.join(workspaceDir, "Child.vue")).href,
              newUri: renamedChild,
            },
          ],
        })) as { changes?: Record<string, Array<{ newText: string }>> } | null;
        const fileRenameTexts = (renameEdit?.changes?.[uri] ?? []).map((edit) => edit.newText);
        assert.ok(fileRenameTexts.includes("./RenamedChild.vue"), JSON.stringify(renameEdit));
        assert.equal(fileRenameTexts.includes("./useThing"), false, JSON.stringify(renameEdit));

        const workspaceSymbols = (await session.request("workspace/symbol", {
          query: "submitMessage",
        })) as Array<{ name: string; location: { uri: string } }> | null;
        assert.ok(
          workspaceSymbols?.some(
            (symbol) => symbol.name === "submitMessage" && symbol.location.uri === uri,
          ),
          JSON.stringify(workspaceSymbols),
        );
        assert.equal(
          await session.request("workspace/symbol", { query: "missingScorecardSymbol" }),
          null,
        );
      },
    );
  } catch (error) {
    failure = error;
  } finally {
    try {
      await session.shutdown();
    } catch (error) {
      if (failure == null) {
        failure = error;
      } else {
        await session.kill().catch(() => undefined);
      }
    }
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
  if (failure != null) throw failure;
});
