import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
} from "../../_helpers/realworld-typecheck.ts";

// #3322: on the generated 24-file vue-benchmarks corpus Vize reported 94
// diagnostics where vue-tsc reported 18. Every one of the 94 landed on an SFC
// whose script block is plain JavaScript, and every one of vue-tsc's 18 landed
// on a `lang="ts"` SFC — the two sets were disjoint. The inference itself was
// never wrong: with `lang="ts"` both engines produce byte-identical codes and
// ranges. What diverged was *whether a JavaScript SFC is checked at all*.
// TypeScript only checks a `.js` file under `checkJs`, and `@vue/language-core`
// gives a `lang="js"` block a `.js` virtual extension for exactly that reason,
// while Vize always generated `.vue.ts`.
//
// This oracle pins the contract in both directions against the real vue-tsc:
// an unchecked JavaScript SFC reports nothing, and every genuine diagnostic —
// the same code in a TypeScript SFC, a TypeScript module misused *by* the
// JavaScript SFC, a `// @ts-check` opt-in, a `checkJs` project, and a
// script-less SFC — still reports at the exact authored range.

const baseCompilerOptions = {
  esModuleInterop: true,
  isolatedModules: true,
  lib: ["ESNext", "DOM"],
  module: "ESNext",
  moduleResolution: "bundler",
  noEmit: true,
  skipLibCheck: true,
  strict: true,
  target: "ESNext",
  types: [],
};

/// Plain-JavaScript SFC carrying one member of each family from #3322: an
/// implicit-`any` parameter (TS7006) and `never[]` inference from `ref([])`
/// (TS2339). It also misuses a TypeScript module so the negative case has a
/// diagnostic that must survive.
const javascriptApp = `<script setup>
import { ref } from 'vue'
import { needsNumber } from './helper'

const rows = ref([])
function selectRow(row) {
  needsNumber(row)
}
</script>

<template>
  <ul>
    <li v-for="row in rows" :key="row.id" @click="selectRow(row)">{{ row.label }}</li>
  </ul>
</template>
`;

const typescriptApp = javascriptApp.replace("<script setup>", '<script setup lang="ts">');
const pragmaApp = javascriptApp.replace("<script setup>", "<script setup>\n// @ts-check");

/// A TypeScript module with an error of its own, imported by the JavaScript
/// SFC. Suppressing the SFC must never suppress this.
const helperModule = `export function needsNumber(value: number): number {
  return value + 1;
}

const wrong: number = "not-a-number";
export const exported = wrong;
`;

const templateOnlyApp = `<template>
  <button type="button" :disabled="'yes'">go</button>
</template>
`;

const appDiagnostics = (line: number): string[] => [
  `error:${line}:20 [TS7006] Parameter 'row' implicitly has an 'any' type.`,
  `error:${line + 7}:39 [TS2339] Property 'id' does not exist on type 'never'.`,
  `error:${line + 7}:74 [TS2339] Property 'label' does not exist on type 'never'.`,
];

const vueTscAppDiagnostics = (file: string, line: number): string =>
  [
    `${file}(${line},20): error TS7006: Parameter 'row' implicitly has an 'any' type.`,
    `${file}(${line + 7},39): error TS2339: Property 'id' does not exist on type 'never'.`,
    `${file}(${line + 7},74): error TS2339: Property 'label' does not exist on type 'never'.`,
    "",
  ].join("\n");

const helperDiagnostic = "error:5:7 [TS2322] Type 'string' is not assignable to type 'number'.";
// vue-tsc may report files in host-dependent program order. Keep the diagnostic
// text exact, but compare it independent of file traversal order.
const vueTscHelperDiagnostic =
  "helper.ts(5,7): error TS2322: Type 'string' is not assignable to type 'number'.\n";

type Case = {
  compilerOptions?: Record<string, unknown>;
  diagnostics: Record<string, string[]>;
  files: Record<string, string>;
  id: string;
  vueTscOutput: string;
};

const cases: Case[] = [
  {
    id: "a JavaScript SFC is unchecked without checkJs",
    files: { "JsApp.vue": javascriptApp, "TsApp.vue": typescriptApp, "helper.ts": helperModule },
    diagnostics: {
      "JsApp.vue": [],
      "TsApp.vue": appDiagnostics(6),
      "helper.ts": [helperDiagnostic],
    },
    vueTscOutput: vueTscAppDiagnostics("TsApp.vue", 6) + vueTscHelperDiagnostic,
  },
  {
    id: "checkJs opts the same JavaScript SFC back in",
    compilerOptions: { allowJs: true, checkJs: true },
    files: { "JsApp.vue": javascriptApp, "TsApp.vue": typescriptApp, "helper.ts": helperModule },
    diagnostics: {
      "JsApp.vue": appDiagnostics(6),
      "TsApp.vue": appDiagnostics(6),
      "helper.ts": [helperDiagnostic],
    },
    vueTscOutput:
      vueTscAppDiagnostics("JsApp.vue", 6) +
      vueTscAppDiagnostics("TsApp.vue", 6) +
      vueTscHelperDiagnostic,
  },
  {
    id: "a leading @ts-check pragma opts a single block back in",
    files: { "JsApp.vue": pragmaApp, "helper.ts": helperModule },
    diagnostics: { "JsApp.vue": appDiagnostics(7), "helper.ts": [helperDiagnostic] },
    vueTscOutput: vueTscAppDiagnostics("JsApp.vue", 7) + vueTscHelperDiagnostic,
  },
  {
    id: "a script-less SFC stays checked",
    files: { "TemplateOnly.vue": templateOnlyApp },
    diagnostics: {
      "TemplateOnly.vue": [
        `error:2:26 [TS2322] Type '"yes"' is not assignable to type 'Booleanish | undefined'.`,
      ],
    },
    vueTscOutput:
      `TemplateOnly.vue(2,26): error TS2322: Type '"yes"' is not assignable to type ` +
      "'Booleanish | undefined'.\n",
  },
];

test("vize check matches vue-tsc on JavaScript SFCs under checkJs", async (t) => {
  assert.equal(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "the checkJs oracle must run with VIZE_TEST_REQUIRE_TSGO=1",
  );
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  const outputRoot = path.join(repoRoot, "target/vize-tests/javascript-sfc-checkjs");
  fs.mkdirSync(outputRoot, { recursive: true });

  for (const testCase of cases) {
    await t.test(testCase.id, () => {
      const workspaceDir = fs.mkdtempSync(path.join(outputRoot, "case-"));
      try {
        writeProject(workspaceDir, testCase);
        symlinkVueTypes(workspaceDir);

        const first = runVizeCheck(workspaceDir, corsaPath, []);
        assert.deepEqual(
          omitProgramEvidence(first.report),
          expectedReport(testCase.diagnostics),
          JSON.stringify(first.report),
        );
        const second = runVizeCheck(workspaceDir, corsaPath, []);
        assert.equal(second.stdout, first.stdout, "diagnostics must be byte-stable");

        const vueTsc = runVueTsc(workspaceDir, vueTscPath);
        assert.deepEqual(
          diagnosticLines(vueTsc.stdout),
          diagnosticLines(testCase.vueTscOutput),
          vueTsc.stderr,
        );
      } finally {
        fs.rmSync(workspaceDir, { recursive: true, force: true });
      }
    });
  }
});

function writeProject(workspaceDir: string, testCase: Case): void {
  for (const [relativePath, source] of Object.entries(testCase.files)) {
    fs.writeFileSync(path.join(workspaceDir, relativePath), source, "utf8");
  }
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: { ...baseCompilerOptions, ...testCase.compilerOptions },
        vueCompilerOptions: { strictTemplates: true },
        include: ["**/*.vue", "**/*.ts"],
      },
      null,
      2,
    ),
  );
  fs.writeFileSync(
    path.join(workspaceDir, "package.json"),
    JSON.stringify({ name: "vize-javascript-sfc-checkjs", private: true, type: "module" }),
  );
}

function diagnosticLines(output: string): string[] {
  return output
    .trimEnd()
    .split("\n")
    .filter((line) => line.length > 0)
    .sort();
}

function expectedReport(diagnostics: Record<string, string[]>): unknown {
  const entries = Object.keys(diagnostics)
    .sort()
    .map((file) => ({ file, diagnostics: diagnostics[file] ?? [] }));
  return {
    files: entries,
    errorCount: entries.reduce((count, entry) => count + entry.diagnostics.length, 0),
    warningCount: 0,
    fileCount: entries.length,
  };
}
