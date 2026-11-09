import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

const vueGlobs = [
  "tests/fixtures/**/*.vue",
  "tests/_fixtures/_projects/**/*.vue",
  "playground/src/**/*.vue",
  "playground/e2e/**/*.vue",
] as const;

const folioGlobs = [
  "crates/vize_croquis/tests/fixtures/**/*.folio",
  "davinci/vize_l0/tests/fixtures/**/*.folio",
  "davinci/vize_l2/tests/fixtures/**/*.folio",
] as const;

export const corpusTargets = [
  "sfc_parse",
  "template_lexer",
  "js_ts_expression",
  "l1_program",
  "css_parse",
  "template_compile",
  "l1_lowering",
  "l2_to_l3_lowering",
  "folio_parse",
] as const;
type CorpusTarget = (typeof corpusTargets)[number];

function isRepositoryRoot(directory: string): boolean {
  return (
    fs.existsSync(path.join(directory, "Cargo.toml")) &&
    fs.existsSync(path.join(directory, "pnpm-workspace.yaml"))
  );
}

function repositoryRoot(): string {
  const configured = process.env.VIZE_REPO_ROOT;
  if (configured !== undefined) {
    if (!isRepositoryRoot(configured)) {
      throw new Error(`VIZE_REPO_ROOT=${configured} is not a Vize repository root`);
    }
    return fs.realpathSync.native(configured);
  }
  for (const candidate of [path.dirname(fileURLToPath(import.meta.url)), process.cwd()]) {
    let directory = path.resolve(candidate);
    for (;;) {
      if (isRepositoryRoot(directory)) return fs.realpathSync.native(directory);
      const parent = path.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }
  throw new Error(`cannot find Vize repository root from ${process.cwd()}`);
}

function globFiles(root: string, patterns: readonly string[]): string[] {
  return patterns
    .flatMap((pattern) => fs.globSync(pattern, { cwd: root }))
    .map((file) => path.join(root, file))
    .sort();
}

function resetCorpus(root: string, target: CorpusTarget): string {
  const directory = path.join(root, target);
  fs.rmSync(directory, { force: true, recursive: true });
  fs.mkdirSync(directory, { recursive: true });
  return directory;
}

function seedHash(contents: Uint8Array): string {
  return createHash("sha1").update(contents).digest("hex").slice(0, 16);
}

function writeSeed(directory: string, contents: Uint8Array): void {
  if (contents.byteLength > 0) fs.writeFileSync(path.join(directory, seedHash(contents)), contents);
}

function writeProgramSeeds(directory: string, source: Uint8Array): void {
  for (let selector = 0; selector < 8; selector += 1) {
    writeSeed(directory, Buffer.concat([Buffer.from([selector]), source]));
  }
}

export function extractTemplateBlock(source: string): string | undefined {
  const lower = source.toLowerCase();
  const openStart = lower.indexOf("<template");
  if (openStart < 0) return undefined;
  const openEnd = source.indexOf(">", openStart + "<template".length);
  if (openEnd < 0) return undefined;
  const close = lower.indexOf("</template>", openEnd + 1);
  return close < 0 ? undefined : source.slice(openEnd + 1, close);
}

export function seedFuzzCorpus(arguments_: readonly string[]): number {
  if (arguments_.length > 0) {
    throw new Error(
      `Usage: node tools/commands/ci/fuzz/seed-corpus.ts; unexpected argument ${arguments_[0]}`,
    );
  }
  const root = repositoryRoot();
  const corpusRoot = path.join(root, "tests/fuzz/corpus");
  const directories = Object.fromEntries(
    corpusTargets.map((target) => [target, resetCorpus(corpusRoot, target)]),
  ) as Record<CorpusTarget, string>;

  const folioFiles = globFiles(root, folioGlobs);
  for (const file of folioFiles) writeSeed(directories.folio_parse, fs.readFileSync(file));

  const vueFiles = globFiles(root, vueGlobs);
  const scriptBlocks = /<script\b[^>]*>(.*?)<\/script>/gis;
  const styleBlocks = /<style\b[^>]*>(.*?)<\/style>/gis;
  const interpolations = /\{\{(.*?)\}\}/gs;

  let sfcCount = 0;
  for (const file of globFiles(root, ["tests/fuzz/regressions/sfc_parse/*"])) {
    writeSeed(directories.sfc_parse, fs.readFileSync(file));
    sfcCount += 1;
  }
  let loweringRegressionCount = 0;
  for (const file of globFiles(root, ["tests/fuzz/regressions/l1_lowering/*"])) {
    writeSeed(directories.l1_lowering, fs.readFileSync(file));
    loweringRegressionCount += 1;
  }
  let expressionCount = 0;
  for (const file of globFiles(root, ["tests/fuzz/regressions/js_ts_expression/*"])) {
    writeSeed(directories.js_ts_expression, fs.readFileSync(file));
    expressionCount += 1;
  }
  let programCount = 0;
  for (const file of globFiles(root, ["tests/fuzz/regressions/l1_program/*"])) {
    writeProgramSeeds(directories.l1_program, fs.readFileSync(file));
    programCount += 1;
  }
  let styleCount = 0;
  for (const file of globFiles(root, ["crates/vize_atelier_sfc/tests/fixtures/css-engine/*.css"])) {
    writeSeed(directories.css_parse, fs.readFileSync(file));
    styleCount += 1;
  }

  let templateCount = 0;
  for (const file of vueFiles) {
    const contents = fs.readFileSync(file, "utf8");
    writeSeed(directories.sfc_parse, Buffer.from(contents));
    sfcCount += 1;

    const template = extractTemplateBlock(contents);
    if (template !== undefined) {
      const bytes = Buffer.from(template);
      for (const target of [
        "template_lexer",
        "template_compile",
        "l1_lowering",
        "l2_to_l3_lowering",
      ] as const) {
        writeSeed(directories[target], bytes);
      }
      for (const match of template.matchAll(interpolations)) {
        writeSeed(directories.js_ts_expression, Buffer.from(match[1]!.trim()));
        expressionCount += 1;
      }
      templateCount += 1;
    }
    for (const match of contents.matchAll(scriptBlocks)) {
      const bytes = Buffer.from(match[1]!);
      writeSeed(directories.js_ts_expression, bytes);
      writeProgramSeeds(directories.l1_program, bytes);
      expressionCount += 1;
      programCount += 1;
    }
    for (const match of contents.matchAll(styleBlocks)) {
      writeSeed(directories.css_parse, Buffer.from(match[1]!));
      styleCount += 1;
    }
  }

  console.log(
    `Seeded ${sfcCount} sfc_parse entries, ${templateCount} template entries (template_lexer/template_compile/l1_lowering/l2_to_l3_lowering), ${expressionCount} JS/TS expression entries, ${programCount} native Program sources in eight explicit profiles, ${styleCount} CSS entries, ${folioFiles.length} folio pages from ${vueFiles.length + folioFiles.length} fixtures, and ${loweringRegressionCount} retained l1_lowering regression entries.`,
  );
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => seedFuzzCorpus(process.argv.slice(2)));
}
