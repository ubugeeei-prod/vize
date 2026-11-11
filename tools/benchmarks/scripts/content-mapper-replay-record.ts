/**
 * Record the content-mapper request stream `tsc` sends for each real corpus,
 * for content-mapper-replay.ts.
 *
 *   node tools/benchmarks/scripts/content-mapper-replay-record.ts \
 *     --bin /tmp/vize-base --tsc tools/benchmarks/typecheck-hosts/node_modules/.bin/tsc \
 *     --corpora misskey,n8n,reka-ui,nuxt-ui --work /tmp/replay
 *
 * Each corpus is laid out as the type-check corpus pair lays it out (sources,
 * tsconfig, the pinned corpus packages, the workspace's helper declarations
 * as `node_modules/vize`), and the `vize` package's mapper command runs
 * content-mapper-replay-proxy.ts, which copies every byte tsc writes to
 * `<work>/<corpus>-recording.<pid>.bin` before `--bin` answers it. The largest recording per corpus becomes `<work>/<corpus>.bin`,
 * and the script prints one `--stream <corpus>=<file>@<project>` argument per
 * corpus. Replays must run while the projects exist: the mapper reads
 * imported type sources from them.
 */

import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { CORPORA as PAIR_CORPORA } from "./typecheck-corpus-pair.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptsDir, "..", "..", "..");
const CORPUS_PACKAGES = resolve(scriptsDir, "..", "typecheck-corpora", "node_modules");
const PROXY = join(scriptsDir, "content-mapper-replay-proxy.ts");

type ReplayCorpus = {
  id: string;
  source: string;
  copy: readonly string[];
  tsconfig: unknown;
};

/**
 * Library corpora beside the pair's application corpora, with the type-check
 * configuration each project runs (`vue-tsc --noEmit` on nuxt/ui `src` with
 * the options `nuxt-module-build prepare` writes; reka-ui `packages/core`
 * `tsconfig.check.json`).
 */
const LIBRARY_CORPORA: ReplayCorpus[] = [
  {
    id: "reka-ui",
    source: "tests/_fixtures/_git/reka-ui/packages/core",
    copy: ["src", "env.d.ts"],
    tsconfig: {
      compilerOptions: {
        noEmit: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        resolveJsonModule: true,
        allowImportingTsExtensions: true,
        moduleDetection: "force",
        jsx: "preserve",
        jsxImportSource: "vue",
        noImplicitThis: true,
        strict: true,
        verbatimModuleSyntax: true,
        target: "ESNext",
        useDefineForClassFields: true,
        esModuleInterop: true,
        forceConsistentCasingInFileNames: true,
        skipLibCheck: true,
        incremental: false,
        lib: ["ESNext", "DOM", "DOM.Iterable"],
        types: [],
        paths: { "@/*": ["./src/*"] },
      },
      include: ["env.d.ts", "src/**/*", "src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
      exclude: ["**/*.test.ts", "src/**/*.story.vue", "src/**/story/*", "src/**/stories/*"],
    },
  },
  {
    id: "nuxt-ui",
    source: "tests/_fixtures/_git/nuxt-ui",
    copy: ["src"],
    tsconfig: {
      compilerOptions: {
        esModuleInterop: true,
        skipLibCheck: true,
        target: "ESNext",
        allowJs: true,
        allowImportingTsExtensions: true,
        resolveJsonModule: true,
        moduleDetection: "force",
        isolatedModules: true,
        verbatimModuleSyntax: true,
        allowArbitraryExtensions: true,
        strict: true,
        noUncheckedIndexedAccess: true,
        forceConsistentCasingInFileNames: true,
        noImplicitOverride: true,
        module: "preserve",
        noEmit: true,
        incremental: false,
        lib: ["ESNext", "dom", "dom.iterable", "webworker"],
        jsx: "preserve",
        jsxImportSource: "vue",
        types: [],
        moduleResolution: "Bundler",
        useDefineForClassFields: true,
        noImplicitThis: true,
        allowSyntheticDefaultImports: true,
      },
      include: ["./src/**/*"],
    },
  },
];

export const REPLAY_CORPORA: ReplayCorpus[] = [
  ...PAIR_CORPORA,
  ...LIBRARY_CORPORA.filter((corpus) => !PAIR_CORPORA.some((pair) => pair.id === corpus.id)),
];

/** A corpus project whose `vize` mapper tees tsc's bytes to `streamPrefix.<pid>.bin`. */
export function prepareRecordingProject(
  dir: string,
  corpus: ReplayCorpus,
  bin: string,
  streamPrefix: string,
): void {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  for (const entry of corpus.copy) {
    cpSync(join(repoRoot, corpus.source, entry), join(dir, entry), {
      recursive: true,
      verbatimSymlinks: true,
    });
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(corpus.tsconfig, null, 2)}\n`);
  writeFileSync(
    join(dir, "tsconfig.vize.json"),
    `${JSON.stringify({ extends: "./tsconfig.json", contentMappers: [{ package: "vize", extensions: [".vue"] }] })}\n`,
  );
  cpSync(CORPUS_PACKAGES, join(dir, "node_modules"), { recursive: true, verbatimSymlinks: true });
  const cli = join(repoRoot, "npm", "cli");
  const pkg = join(dir, "node_modules", "vize");
  mkdirSync(pkg, { recursive: true });
  for (const name of readdirSync(cli)) {
    if (/\.d\.(ts|mts|cts)$/.test(name)) symlinkSync(join(cli, name), join(pkg, name));
  }
  const manifest = JSON.parse(readFileSync(join(cli, "package.json"), "utf8")) as {
    version: string;
    type: string;
    exports: unknown;
    typescript: { contentMapper: Record<string, unknown> };
  };
  // The recording proxy copies every request byte, then the binary answers.
  const exec = [
    process.execPath,
    PROXY,
    `${streamPrefix}.%p.bin`,
    "-",
    "--",
    resolve(bin),
    "content-mapper",
  ];
  writeFileSync(
    join(pkg, "package.json"),
    `${JSON.stringify({
      name: "vize",
      version: manifest.version,
      type: manifest.type,
      exports: manifest.exports,
      typescript: { contentMapper: { ...manifest.typescript.contentMapper, exec } },
    })}\n`,
  );
}

function main(): void {
  const { values } = parseArgs({
    options: {
      bin: { type: "string" },
      tsc: { type: "string" },
      corpora: { type: "string", default: REPLAY_CORPORA.map((corpus) => corpus.id).join(",") },
      work: { type: "string" },
    },
  });
  if (!values.bin || !values.tsc || !values.work) {
    throw new Error("usage: --bin <vize> --tsc <tsc> --work <dir> [--corpora a,b]");
  }
  const work = resolve(values.work);
  mkdirSync(work, { recursive: true });
  const specs: string[] = [];
  for (const id of values.corpora.split(",").map((value) => value.trim())) {
    const corpus = REPLAY_CORPORA.find((candidate) => candidate.id === id);
    if (!corpus) throw new Error(`unknown corpus: ${id}`);
    const project = join(work, id);
    const prefix = join(work, `${id}-recording`);
    prepareRecordingProject(project, corpus, values.bin, prefix);
    const result = spawnSync(
      resolve(values.tsc),
      ["-p", "tsconfig.vize.json", "--noEmit", "--runExternalCode", "--pretty", "false"],
      { cwd: project, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
    );
    // Type errors exit non-zero; only a missing mapper session is fatal.
    const recordings = readdirSync(work)
      .filter((name) => name.startsWith(`${id}-recording.`) && name.endsWith(".bin"))
      .map((name) => join(work, name))
      .sort((a, b) => statSync(b).size - statSync(a).size);
    const largest = recordings[0];
    if (!largest)
      throw new Error(`${id}: tsc started no mapper (status ${result.status}): ${result.stderr}`);
    const stream = join(work, `${id}.bin`);
    copyFileSync(largest, stream);
    for (const recording of recordings) rmSync(recording);
    const diagnostics = (result.stdout.match(/error TS\d+/g) ?? []).length;
    console.error(
      `${id}: ${recordings.length} mapper session(s), ${statSync(stream).size} request bytes, ${diagnostics} diagnostics`,
    );
    specs.push(`--stream`, `${id}=${stream}@${project}`);
  }
  process.stdout.write(`${specs.join(" ")}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
