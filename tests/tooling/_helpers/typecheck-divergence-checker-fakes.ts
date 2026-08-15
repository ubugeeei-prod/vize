import fs from "node:fs";
import path from "node:path";

export type MutationDiagnosticMode = "match" | "missing" | "mismatch";

export function writeVize(
  pathname: string,
  options: { baselineDiagnostics: string[]; mutation: MutationDiagnosticMode },
) {
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node
import fs from "node:fs";
if (process.argv.includes("--version")) { console.log("vize 0.0.0-test"); process.exit(0); }
const baselineDiagnostics = new Map([["src/App.vue", ${JSON.stringify(options.baselineDiagnostics)}]]);
${mutationScript()}
const files = listVueFiles("src").sort();
const entries = files.map((sourcePath) => {
  const source = fs.readFileSync(sourcePath, "utf8");
  const diagnostics = [...(baselineDiagnostics.get(sourcePath) ?? [])];
  const mutation = mutationDiagnostic(source, ${JSON.stringify(options.mutation)}, "vize", sourcePath);
  if (mutation != null) diagnostics.push(mutation);
  return { file: sourcePath, diagnostics };
});
const report = {
  errorCount: entries.flatMap((entry) => entry.diagnostics).filter((entry) => entry.startsWith("error:")).length,
  warningCount: entries.flatMap((entry) => entry.diagnostics).filter((entry) => entry.startsWith("warning:")).length,
  fileCount: entries.length,
  files: entries,
};
process.stdout.write(JSON.stringify(report));
process.exit(report.errorCount > 0 ? 1 : 0);
`,
  );
  fs.chmodSync(pathname, 0o755);
}

export function writeVueTsc(pathname: string, runBody: string, invocationPath?: string) {
  const recordInvocation =
    invocationPath == null
      ? ""
      : `fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), args: process.argv.slice(2) }));`;
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node
import fs from "node:fs";
if (process.argv.includes("--version")) { console.log("3.3.4"); process.exit(0); }
${recordInvocation}
${runBody}
`,
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
  },
  invocationPath: string,
) {
  const sourceFiles = options.files.map((file) => ({
    file,
    sourcePath: path.join(options.fixtureRoot, file),
  }));
  const files = options.files.map((file) => `${path.join(options.fixtureRoot, file)}\n`).join("");
  const runBody = `
if (process.argv.includes("--listFilesOnly")) {
  process.${options.coverageOutputStream === "stderr" ? "stderr" : "stdout"}.write(${JSON.stringify(files)});
  process.exit(${JSON.stringify(options.coverageExitCode ?? 0)});
}
let output = ${JSON.stringify(options.baselineOutput)};
${mutationScript()}
for (const { file, sourcePath } of ${JSON.stringify(sourceFiles)}) {
  if (!fs.existsSync(sourcePath)) continue;
  const source = fs.readFileSync(sourcePath, "utf8");
  const mutation = mutationDiagnostic(source, ${JSON.stringify(options.mutation)}, "vue-tsc", file);
  if (mutation != null) output += mutation;
}
process.${options.baselineOutputStream === "stderr" ? "stderr" : "stdout"}.write(output);
process.exit(${JSON.stringify(options.exitCode ?? 2)});
`;
  writeVueTsc(pathname, runBody, invocationPath);
}

function mutationScript() {
  return `
function listVueFiles(root) {
  if (!fs.existsSync(root)) return [];
  const entries = [];
  for (const name of fs.readdirSync(root, { withFileTypes: true })) {
    const child = \`\${root}/\${name.name}\`;
    if (name.isDirectory()) entries.push(...listVueFiles(child));
    else if (name.isFile() && child.endsWith(".vue")) entries.push(child);
  }
  return entries;
}

function mutationDiagnostic(source, mode, tool, file) {
  if (!source.includes("__vize_typecheck_mutation_probe: string = 1")) return null;
  if (source.includes("vize-mutation-invisible")) return null;
  if (mode === "missing") return null;
  const line = source.slice(0, source.indexOf("__vize_typecheck_mutation_probe")).split(/\\r?\\n/).length;
  const message = mode === "mismatch"
    ? "Type 'number' is not assignable to type 'boolean'."
    : "Type 'number' is not assignable to type 'string'.";
  if (tool === "vize") return \`error:\${line}:1 [TS2322] \${message}\`;
  return \`\${file}(\${line},1): error TS2322: \${message}\\n\`;
}
`;
}
