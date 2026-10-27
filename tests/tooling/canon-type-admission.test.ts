import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { admissionFiles } from "./support/type-admission-project.ts";
import {
  check,
  type Diagnostic,
  compareIdentity,
  diagnosticIdentity,
  workspace,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";
import { root } from "./support/lsp/paths.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

// These authored constraints are checked even when no template reads the variable.
// A clean result therefore also proves exact types, absence of any/unknown/never,
// nullability, readonly, and the exposed component's argument/return/state types.
test("template refs preserve exact types and reject every unique mutation before repair", async (t) => {
  const checker = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "type admission backend",
    "TypeScript native backend unavailable",
  );
  if (!checker) return;
  const directory = workspace("type-admission-");
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ES2024",
        lib: ["ES2024", "DOM"],
        types: [],
      },
      // Vue's oracle recognizes configured composable names rather than import
      // symbols. Vize must resolve the authored import alias independently.
      vueCompilerOptions: {
        composables: {
          useTemplateRef: ["useTemplateRef", "templateRef", "r", "useTemplateRe\\u0066"],
        },
      },
      include: ["*.vue", "*.ts"],
    }),
  );
  try {
    for (const [name, stage] of [
      ["valid", "valid"],
      ["invalid", "invalid"],
      ["repaired", "valid"],
      ["proof-liveness", "poisoned"],
      ["proof-repaired", "valid"],
    ] as const) {
      const { files, expected } = admissionFiles(stage);
      for (const [file, source] of Object.entries(files))
        fs.writeFileSync(path.join(directory, file), source);
      const captureName = `template-ref-admission-${name}`;
      const oracleCapture = `${captureName}-vue`;
      fs.mkdirSync(
        path.join(root, "target/differential/fallthrough-listener-inputs", oracleCapture),
        { recursive: true },
      );
      const subjects: [
        PromiseSettledResult<Diagnostic[]>,
        PromiseSettledResult<Pick<Diagnostic, "file" | "line" | "column" | "code">[]>,
      ] = await Promise.allSettled([
        check(directory, ["--corsa-path", checker], {
          name: captureName,
          sources: ["tsconfig.json", ...Object.keys(files)],
        }),
        Promise.resolve().then(() => vueTscDiagnostics(directory, oracleCapture)),
      ]);
      for (const result of subjects) if (result.status === "rejected") throw result.reason;
      const actual: Diagnostic[] = subjects[0].status === "fulfilled" ? subjects[0].value : [];
      const oracle: Pick<Diagnostic, "file" | "line" | "column" | "code">[] =
        subjects[1].status === "fulfilled" ? subjects[1].value : [];
      // Arrays retain duplicate diagnostics. A Set would conceal repeated reports
      // and could conceal an unvisited file behind another file's same message.
      assert.deepEqual(
        actual.map(diagnosticIdentity).sort(compareIdentity),
        [...expected].sort(compareIdentity),
        `${name}: Vize authored evidence`,
      );
      assert.deepEqual(
        oracle.sort(compareIdentity),
        [...expected].sort(compareIdentity),
        `${name}: independent Vue oracle`,
      );
      if (stage !== "valid") assert.equal(actual.length, 30);
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
