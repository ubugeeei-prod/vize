import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { ESLint, Linter } from "eslint";

type Native = Pick<typeof import("@vizejs/native"), "lintPatinaSfc">;
interface NativeCall {
  source: string;
  options: Parameters<Native["lintPatinaSfc"]>[1];
  result: unknown;
}
interface ProjectFile {
  path: string;
  source: string;
  sourceSha256: string;
  installed: ESLint.LintResult[];
  reference: ESLint.LintResult[];
  installedCalls: NativeCall[];
  referenceCalls: NativeCall[];
  repeated: ESLint.LintResult[];
}
interface Consumer {
  ESLint: typeof import("eslint").ESLint;
  plugin: ESLint.Plugin & { configs: { recommended: Linter.Config } };
  bridge: ESLint.Plugin;
  native: Native;
  projectRoot: string;
}

const projects = [
  { root: "examples/vite-musea", minimumFiles: 7 },
  { root: "examples/jsx-tsx", minimumFiles: 1 },
];
const digest = (source: string): string => createHash("sha256").update(source).digest("hex");
const packet = (value: unknown): unknown => JSON.parse(JSON.stringify(value));

function sfcFiles(directory: string): string[] {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const file = path.join(directory, entry.name);
      return entry.isDirectory() ? sfcFiles(file) : entry.name.endsWith(".vue") ? [file] : [];
    })
    .sort();
}

/**
 * Compares complete installed-adapter and shared-bridge packets on application sources.
 * Native calls, diagnostics, ranges, suggestions and fixes retain their original order.
 * This observes the same rule objects; it implements no lint or offset semantics.
 */
export async function qualifyRealProjects({
  ESLint,
  plugin,
  bridge,
  native,
  projectRoot,
}: Consumer) {
  assert.equal(plugin.rules, bridge.rules, "the installed adapter must retain the shared rules");
  const configuration = (selected: ESLint.Plugin): Linter.Config[] => [
    {
      ...plugin.configs.recommended,
      plugins: { vize: selected },
      settings: { vize: { helpLevel: "none", locale: "en", preset: "incremental" } },
    },
  ];
  const installed = new ESLint({
    overrideConfigFile: true,
    overrideConfig: configuration(plugin),
    fix: true,
  });
  const reference = new ESLint({
    overrideConfigFile: true,
    overrideConfig: configuration(bridge),
    fix: true,
  });
  const report = {
    schema: "vize.eslint.real-projects.v1",
    owner: "linter/vize_patina",
    issue: 13019,
    maturity: "experimental",
    projects: [] as { root: string; manifestSha256: string; files: ProjectFile[] }[],
  };
  const original = native.lintPatinaSfc;
  const calls: NativeCall[] = [];
  native.lintPatinaSfc = (source, options) => {
    const result: unknown = original(source, options);
    calls.push({ source, options: structuredClone(options), result: structuredClone(result) });
    return result;
  };
  try {
    for (const project of projects) {
      const directory = path.join(projectRoot, project.root);
      const manifest = fs.readFileSync(path.join(directory, "package.json"), "utf8");
      const files = sfcFiles(path.join(directory, "src"));
      assert.ok(files.length >= project.minimumFiles, `${project.root}: application SFCs missing`);
      const projectPacket = {
        root: project.root,
        manifestSha256: digest(manifest),
        files: [] as ProjectFile[],
      };
      report.projects.push(projectPacket);
      for (const file of files) {
        const relative = path.relative(directory, file).split(path.sep).join("/");
        const source = fs.readFileSync(file, "utf8");
        const filePath = path.resolve("application sources 雪", path.basename(directory), relative);
        const row: ProjectFile = {
          path: relative,
          source,
          sourceSha256: digest(source),
          installed: [],
          reference: [],
          installedCalls: [],
          referenceCalls: [],
          repeated: [],
        };
        projectPacket.files.push(row);
        let start = calls.length;
        row.installed = await installed.lintText(source, { filePath });
        row.installedCalls = calls.slice(start);
        start = calls.length;
        row.reference = await reference.lintText(source, { filePath });
        row.referenceCalls = calls.slice(start);
        assert.ok(row.installedCalls.length > 0, `${file}: actual native rule execution required`);
        assert.equal(
          row.installedCalls[0]!.source,
          source,
          `${file}: original application source reaches native`,
        );
        assert.deepEqual(
          packet(row.installedCalls),
          packet(row.referenceCalls),
          `${file}: complete native calls`,
        );
        assert.deepEqual(
          packet(row.installed),
          packet(row.reference),
          `${file}: complete ESLint diagnostics and fixes`,
        );
        assert.equal(row.installed.length, 1, file);
        assert.equal(
          row.installed[0]!.fatalErrorCount,
          0,
          `${file}: real host parser must accept the source`,
        );
        row.repeated = await installed.lintText(row.installed[0]!.output ?? source, { filePath });
        assert.equal(
          row.repeated[0]!.output,
          undefined,
          `${file}: application autofix must reach a fixed point`,
        );
        assert.deepEqual(
          row.repeated[0]!.messages,
          row.installed[0]!.messages,
          `${file}: repeated complete diagnostics`,
        );
        assert.equal(fs.readFileSync(file, "utf8"), source, `${file}: original source custody`);
      }
    }
    return report;
  } finally {
    native.lintPatinaSfc = original;
    if (process.env.VIZE_ESLINT_PROJECT_EVIDENCE) {
      fs.writeFileSync(
        process.env.VIZE_ESLINT_PROJECT_EVIDENCE,
        JSON.stringify(report, null, 2) + "\n",
      );
    }
  }
}
