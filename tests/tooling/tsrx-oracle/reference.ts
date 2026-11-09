// Pinned reference compiler for the TSRX Vue profile (#6858). The decision is
// docs/davinci/decisions/2026-11-09-6858-tsrx-vue-profile.md. Products never
// import this package; it records ground truth for differential tests.
//
// Re-record after a deliberate version bump:
//   node tests/tooling/tsrx-oracle/reference.ts --write
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative, sep } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { compile } from "@tsrx/vue";

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

export const repoRoot = join(here, "..", "..", "..");
export const corpusRoot = join(repoRoot, "tests", "_fixtures", "tsrx");
export const corpusFile = join(corpusRoot, "reference.json");

/** The first error the reference reports, with UTF-8 byte offsets. */
export interface ReferenceError {
  code: string | null;
  start: number;
  end: number | null;
  message: string;
}

export interface ReferenceRow {
  file: string;
  code: string | null;
  error: ReferenceError | null;
}

export interface ReferenceCorpus {
  compiler: Record<string, string>;
  rows: ReferenceRow[];
}

function packageVersion(name: string): string {
  let directory = dirname(require.resolve(name));
  for (;;) {
    try {
      const manifest = JSON.parse(readFileSync(join(directory, "package.json"), "utf8"));
      if (manifest.name === name) return manifest.version;
    } catch {
      // Not a package root; keep walking up.
    }
    const parent = dirname(directory);
    if (parent === directory) throw new Error(`no package.json for ${name}`);
    directory = parent;
  }
}

/** Installed versions of the pinned reference packages. */
export function installedCompiler(): Record<string, string> {
  return {
    "@tsrx/core": packageVersion("@tsrx/core"),
    "@tsrx/vue": packageVersion("@tsrx/vue"),
  };
}

function byteOffset(source: string, utf16: number): number {
  return Buffer.byteLength(source.slice(0, utf16), "utf8");
}

/** Compile one module exactly as the reference Vue compiler does by default. */
export function compileReference(file: string, source: string): ReferenceRow {
  try {
    const result = compile(source, file.split("/").at(-1) ?? file);
    return { file, code: result.code, error: null };
  } catch (thrown) {
    const error = thrown as { code?: string; pos?: number; end?: number; message: string };
    if (typeof error.pos !== "number") throw thrown;
    return {
      file,
      code: null,
      error: {
        code: error.code ?? null,
        start: byteOffset(source, error.pos),
        end: typeof error.end === "number" ? byteOffset(source, error.end) : null,
        message: error.message,
      },
    };
  }
}

/** Every `.tsrx` corpus module, as repository-relative paths in sorted order. */
export function corpusFiles(root = corpusRoot): string[] {
  return readdirSync(root, { withFileTypes: true, recursive: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".tsrx"))
    .map((entry) => relative(repoRoot, join(entry.parentPath, entry.name)).split(sep).join("/"))
    .sort();
}

export function recordCorpus(): ReferenceCorpus {
  return {
    compiler: installedCompiler(),
    rows: corpusFiles().map((file) =>
      compileReference(file, readFileSync(join(repoRoot, file), "utf8")),
    ),
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const corpus = recordCorpus();
  const text = `${JSON.stringify(corpus, null, 2)}\n`;
  if (process.argv.includes("--write")) writeFileSync(corpusFile, text);
  else process.stdout.write(text);
}
