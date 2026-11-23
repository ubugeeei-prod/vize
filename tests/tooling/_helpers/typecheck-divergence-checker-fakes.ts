import fs from "node:fs";
import path from "node:path";
import { readToolingProgram } from "../support/program-path.ts";

const mutationDiagnostics = new URL(
  "../programs/commands/typecheck-divergence/mutation-diagnostics.js",
  import.meta.url,
);

export type MutationDiagnosticMode = "match" | "missing" | "mismatch" | "shifted";

export function writeVize(
  pathname: string,
  options: { baselineDiagnostics: string[]; mutation: MutationDiagnosticMode },
) {
  fs.writeFileSync(
    pathname,
    readToolingProgram("commands/typecheck-divergence/vize.js.in", {
      BASELINE_DIAGNOSTICS: JSON.stringify(options.baselineDiagnostics),
      DIAGNOSTICS_MODULE: JSON.stringify(mutationDiagnostics.href),
      MUTATION_MODE: JSON.stringify(options.mutation),
    }),
  );
  fs.chmodSync(pathname, 0o755);
}

export type PreSemanticProbe = {
  output?: string;
  exitCode?: number;
  appendNewlineToConfig?: "generated" | { path: string };
};

export function writeVueTsc(
  pathname: string,
  runBody: string,
  invocationPath?: string,
  preSemantic: PreSemanticProbe = {},
) {
  fs.writeFileSync(
    pathname,
    readToolingProgram("commands/typecheck-divergence/vue-tsc.js.in", {
      INVOCATION_PATH: JSON.stringify(invocationPath ?? null),
      CONFIG_TO_CHANGE: JSON.stringify(preSemantic.appendNewlineToConfig ?? null),
      PRE_SEMANTIC_OUTPUT: JSON.stringify(preSemantic.output ?? ""),
      PRE_SEMANTIC_EXIT_CODE: JSON.stringify(preSemantic.exitCode ?? 0),
      RUN: runBody,
    }),
  );
  fs.chmodSync(pathname, 0o755);
}

export function writeVueTscFixture(
  pathname: string,
  options: {
    baselineOutput: string;
    baselineOutputStream?: "stdout" | "stderr";
    coverageExitCode?: number;
    coverageOutputStream?: "stdout" | "stderr";
    exitCode?: number;
    files: string[];
    fixtureRoot: string;
    mutation: MutationDiagnosticMode;
    /**
     * Non-Vue program entries the same `--listFilesOnly` run emitted, relative
     * to the fixture or absolute. This is how a contaminated ambient environment
     * looks from the outside: the `.vue` corpus is identical and the
     * extra entries are packages the fixture never owned.
     */
    programFiles?: string[];
  },
  invocationPath: string,
) {
  const sourceFiles = options.files.map((file) => ({
    file,
    sourcePath: path.join(options.fixtureRoot, file),
  }));
  const files = [...options.files, ...(options.programFiles ?? [])]
    .map((file) => `${path.resolve(options.fixtureRoot, file)}\n`)
    .join("");
  const runBody = readToolingProgram("commands/typecheck-divergence/vue-tsc-report.js.in", {
    COVERAGE_STREAM: options.coverageOutputStream === "stderr" ? "stderr" : "stdout",
    FILES: JSON.stringify(files),
    COVERAGE_EXIT_CODE: JSON.stringify(options.coverageExitCode ?? 0),
    BASELINE_OUTPUT: JSON.stringify(options.baselineOutput),
    DIAGNOSTICS_MODULE: JSON.stringify(mutationDiagnostics.href),
    SOURCE_FILES: JSON.stringify(sourceFiles),
    MUTATION_MODE: JSON.stringify(options.mutation),
    BASELINE_STREAM: options.baselineOutputStream === "stderr" ? "stderr" : "stdout",
    EXIT_CODE: JSON.stringify(options.exitCode ?? null),
  });
  writeVueTsc(pathname, runBody, invocationPath);
}
