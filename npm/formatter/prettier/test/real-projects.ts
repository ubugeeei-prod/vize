import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { SpawnSyncReturns } from "node:child_process";
import type { FormatResultNapi } from "@vizejs/native";
import type { Plugin } from "prettier";

interface FilePacket {
  path: string;
  source: string;
  sourceSha256: string;
  sourceBytes: number;
  nativeOutput: FormatResultNapi | null;
  output: string | null;
  outputSha256: string | null;
  outputBytes: number | null;
  repeatedOutput: string | null;
  cliOutput: string | null;
}
interface ProjectPacket {
  root: string;
  manifestSha256: string;
  files: FilePacket[];
}
interface Consumer {
  root: string;
  consumer: string;
  output: string;
  prettier: Pick<typeof import("prettier"), "format">;
  plugin: Plugin;
  native: Pick<typeof import("@vizejs/native"), "formatSfc">;
  execute: (parameters: string[]) => SpawnSyncReturns<string>;
}

const projects = [
  { root: "examples/vite-musea", minimumFiles: 7 },
  { root: "examples/jsx-tsx", minimumFiles: 1 },
];

const digest = (value: string): string => createHash("sha256").update(value).digest("hex");

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
 * Qualifies the installed public adapter on every SFC in two source-owned apps.
 * The direct native preset is the output oracle; no formatter semantics live here.
 * Timings are observations, excluded from the complete cross-runtime output packet.
 */
export async function qualifyRealProjects({
  root,
  consumer,
  output,
  prettier,
  plugin,
  native,
  execute,
}: Consumer) {
  const options = {
    printWidth: 64,
    singleQuote: true,
    quoteProps: "preserve",
    semi: false,
    endOfLine: "lf",
  } as const;
  const config = "real-projects.prettier.config.mjs";
  fs.writeFileSync(
    path.join(consumer, config),
    `export default ${JSON.stringify({ ...options, plugins: ["prettier-plugin-vize"] })};\n`,
  );
  const packet = {
    schema: "vize.prettier.real-projects.v1",
    owner: "formatter/vize_glyph",
    issue: 13019,
    maturity: "experimental",
    scope:
      "Installed adapter API and CLI equivalence to the shared native formatter on application SFCs",
    options,
    projects: [] as ProjectPacket[],
  };
  const timings: { project: string; path: string; nativeMs: number; adapterMs: number }[] = [];
  const copiedFiles: string[] = [];
  let cliMs: number | null = null;
  try {
    for (const project of projects) {
      const projectRoot = path.join(root, project.root);
      const manifest = fs.readFileSync(path.join(projectRoot, "package.json"), "utf8");
      const files = sfcFiles(path.join(projectRoot, "src"));
      assert.ok(
        files.length >= project.minimumFiles,
        `${project.root}: real application SFCs missing`,
      );
      const row: ProjectPacket = {
        root: project.root,
        manifestSha256: digest(manifest),
        files: [],
      };
      packet.projects.push(row);
      for (const file of files) {
        const relative = path.relative(projectRoot, file).split(path.sep).join("/");
        const source = fs.readFileSync(file, "utf8");
        const destination = path.posix.join(
          "src/real-projects",
          path.basename(projectRoot),
          relative,
        );
        const filename = path.join(consumer, destination);
        fs.mkdirSync(path.dirname(filename), { recursive: true });
        fs.writeFileSync(filename, source);
        copiedFiles.push(destination);
        const observation: FilePacket = {
          path: relative,
          source,
          sourceSha256: digest(source),
          sourceBytes: Buffer.byteLength(source),
          nativeOutput: null,
          output: null,
          outputSha256: null,
          outputBytes: null,
          repeatedOutput: null,
          cliOutput: null,
        };
        row.files.push(observation);
        const startNative = performance.now();
        const direct = native.formatSfc(source, {
          preset: "prettier",
          vueVersion: "3",
          ...options,
        });
        const nativeMs = performance.now() - startNative;
        observation.nativeOutput = direct;
        const startAdapter = performance.now();
        const installed = await prettier.format(source, {
          ...options,
          filepath: filename,
          plugins: [plugin],
        });
        const adapterMs = performance.now() - startAdapter;
        observation.output = installed;
        observation.outputSha256 = digest(installed);
        observation.outputBytes = Buffer.byteLength(installed);
        timings.push({ project: project.root, path: relative, nativeMs, adapterMs });
        assert.equal(
          installed,
          direct.code,
          `${project.root}/${relative}: complete native/API output`,
        );
        observation.repeatedOutput = await prettier.format(installed, {
          ...options,
          filepath: filename,
          plugins: [plugin],
        });
        assert.equal(
          observation.repeatedOutput,
          installed,
          `${project.root}/${relative}: installed API idempotence`,
        );
      }
    }
    // A rejected host option must not partially rewrite a real application.
    const rejected = execute([
      "--config",
      config,
      "--write",
      ...copiedFiles,
      "--html-whitespace-sensitivity",
      "strict",
    ]);
    assert.equal(rejected.status, 2, rejected.stderr);
    for (const project of packet.projects) {
      for (const file of project.files) {
        const filename = path.join(
          consumer,
          "src/real-projects",
          path.basename(project.root),
          file.path,
        );
        assert.equal(fs.readFileSync(filename, "utf8"), file.source, `${filename}: rejected write`);
      }
    }
    const startCli = performance.now();
    const written = execute(["--config", config, "--write", ...copiedFiles]);
    cliMs = performance.now() - startCli;
    assert.equal(written.status, 0, written.stderr);
    const checked = execute(["--config", config, "--check", ...copiedFiles]);
    assert.equal(checked.status, 0, checked.stderr);
    for (const project of packet.projects) {
      for (const file of project.files) {
        const filename = path.join(
          consumer,
          "src/real-projects",
          path.basename(project.root),
          file.path,
        );
        file.cliOutput = fs.readFileSync(filename, "utf8");
        assert.equal(file.cliOutput, file.output, `${filename}: complete CLI output`);
        assert.equal(
          fs.readFileSync(path.join(root, project.root, file.path), "utf8"),
          file.source,
          `${project.root}/${file.path}: source application remains intact`,
        );
      }
    }
    return packet;
  } finally {
    fs.writeFileSync(
      path.join(output, "real-projects.json"),
      JSON.stringify(
        { packet, timings, cliMs, timingScope: "Single observed pass; no speedup claim" },
        null,
        2,
      ),
    );
  }
}
