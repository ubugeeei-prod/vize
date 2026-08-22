import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";
import { testOutputRoot } from "../../tooling/support/lsp/paths.ts";

test("Vue JSX intrinsic globals stay present while component props remain strict", () => {
  const corsaPath = resolveTsgoBinary();
  const testRootDir = path.join(testOutputRoot, "jsx-intrinsic-globals-oracle");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));

  try {
    const srcDir = path.join(workspaceDir, "src");
    fs.mkdirSync(srcDir, { recursive: true });
    symlinkVueTypes(workspaceDir);
    writeJson(path.join(workspaceDir, "vize.config.json"), {
      typeChecker: { corsaPath, jsxTypecheck: true },
    });
    writeJson(path.join(workspaceDir, "tsconfig.json"), {
      compilerOptions: {
        allowJs: true,
        checkJs: true,
        jsx: "preserve",
        jsxImportSource: "vue",
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        strict: true,
        target: "ES2022",
      },
      include: ["src/**/*"],
    });
    fs.writeFileSync(
      path.join(srcDir, "Counter.vue"),
      `<script setup lang="ts">
defineProps<{ count: number }>();
</script>

<template>
  <button>{{ count }}</button>
</template>
`,
      "utf8",
    );
    fs.writeFileSync(
      path.join(srcDir, "Intrinsic.tsx"),
      `export const view = <main><h1>Dashboard</h1><button disabled>Save</button></main>;
`,
      "utf8",
    );
    const consumerPath = path.join(srcDir, "Consumer.tsx");
    const brokenConsumer = `import Counter from "./Counter.vue";

export const view = <section><Counter count="wrong" /></section>;
`;
    fs.writeFileSync(consumerPath, brokenConsumer, "utf8");

    const broken = runVizeCheck(workspaceDir, corsaPath, []);
    assertNoIntrinsicElementDiagnostic(broken);
    assert.deepEqual(diagnosticsFor(broken, "src/Intrinsic.tsx"), []);
    assert.deepEqual(diagnosticsFor(broken, "src/Consumer.tsx"), [
      "error:3:39 [TS2322] Type 'string' is not assignable to type 'number'.",
    ]);

    fs.writeFileSync(consumerPath, brokenConsumer.replace('"wrong"', "{1}"), "utf8");
    const repaired = runVizeCheck(workspaceDir, corsaPath, []);
    assertNoIntrinsicElementDiagnostic(repaired);
    assert.equal(repaired.report.errorCount, 0, JSON.stringify(repaired.report, null, 2));
  } finally {
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

function writeJson(filePath: string, value: unknown): void {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function diagnosticsFor(result: VizeCheckResult, file: string): string[] {
  return result.report.files.find((entry) => entry.file === file)?.diagnostics ?? [];
}

function assertNoIntrinsicElementDiagnostic(result: VizeCheckResult): void {
  const diagnostics = result.report.files.flatMap((entry) => entry.diagnostics);
  assert.ok(
    diagnostics.every((diagnostic) => !diagnostic.includes("JSX.IntrinsicElements")),
    diagnostics.join("\n"),
  );
  assert.ok(
    diagnostics.every((diagnostic) => !diagnostic.includes("[TS7026]")),
    diagnostics.join("\n"),
  );
}
