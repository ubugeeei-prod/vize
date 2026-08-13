import fs from "node:fs";
import path from "node:path";

export type MutationDiagnosticMode = "match" | "missing" | "mismatch";

export function writeVize(
  pathname: string,
  options: { baselineDiagnostics: string[]; mutation: MutationDiagnosticMode },
) {
  const sourcePath = "src/App.vue";
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node
import fs from "node:fs";
if (process.argv.includes("--version")) { console.log("vize 0.0.0-test"); process.exit(0); }
const sourcePath = ${JSON.stringify(sourcePath)};
const source = fs.readFileSync(sourcePath, "utf8");
const diagnostics = ${JSON.stringify(options.baselineDiagnostics)};
${mutationScript()}
const mutation = mutationDiagnostic(source, ${JSON.stringify(options.mutation)}, "vize");
if (mutation != null) diagnostics.push(mutation);
const report = {
  errorCount: diagnostics.filter((entry) => entry.startsWith("error:")).length,
  warningCount: diagnostics.filter((entry) => entry.startsWith("warning:")).length,
  fileCount: 1,
  files: [{ file: sourcePath, diagnostics }],
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
    files: string[];
    fixtureRoot: string;
    mutation: MutationDiagnosticMode;
  },
  invocationPath: string,
) {
  const sourcePath = path.join(options.fixtureRoot, "src", "App.vue");
  const files = options.files.map((file) => `${path.join(options.fixtureRoot, file)}\n`).join("");
  const runBody = `
const source = fs.readFileSync(${JSON.stringify(sourcePath)}, "utf8");
let output = ${JSON.stringify(options.baselineOutput)};
${mutationScript()}
const mutation = mutationDiagnostic(source, ${JSON.stringify(options.mutation)}, "vue-tsc");
if (mutation != null) output += mutation;
output += ${JSON.stringify(files)};
process.stdout.write(output);
process.exit(2);
`;
  writeVueTsc(pathname, runBody, invocationPath);
}

function mutationScript() {
  return `
function mutationDiagnostic(source, mode, tool) {
  if (!source.includes("__vize_typecheck_mutation_probe: string = 1")) return null;
  if (mode === "missing") return null;
  const line = source.slice(0, source.indexOf("__vize_typecheck_mutation_probe")).split(/\\r?\\n/).length;
  const message = mode === "mismatch"
    ? "Type 'number' is not assignable to type 'boolean'."
    : "Type 'number' is not assignable to type 'string'.";
  if (tool === "vize") return \`error:\${line}:1 [TS2322] \${message}\`;
  return \`src/App.vue(\${line},1): error TS2322: \${message}\\n\`;
}
`;
}
