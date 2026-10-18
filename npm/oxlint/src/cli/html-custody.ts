import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import type { OxlintHtmlOptions, OxlintHtmlCompleted } from "@vizejs/native";
import type { HtmlContext } from "./html-operation.ts";
import { isStandaloneHtmlFile } from "../file-kinds.ts";

export interface HtmlCustody {
  sources: Map<string, Buffer | null>;
  originals: Map<string, Buffer>;
}
const read = (file: string): Buffer | null => {
  try {
    const stat = fs.lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink())
      throw new Error(`HTML custody requires a regular original: ${file}`);
    return fs.readFileSync(file);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
};
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const same = (left: Buffer | null, right: Buffer | null) =>
  left == null ? right == null : right != null && left.equals(right);

function metadataPath(bytes: Buffer, file: string, prefix = ""): string {
  const text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
    .decode(bytes)
    .replace(/\n$/u, "")
    .replace(/\r$/u, "");
  if (!text.startsWith(prefix) || text.length === prefix.length || /[\r\n\0]/u.test(text))
    throw new Error(`invalid Git directory metadata: ${file}`);
  return text.slice(prefix.length);
}

function regularDirectory(directory: string): string {
  const stat = fs.lstatSync(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink())
    throw new Error(`Git metadata requires a regular directory: ${directory}`);
  return fs.realpathSync(directory);
}

/** Resolve Git's per-worktree marker and shared directory without losing input custody. */
function gitDirectory(marker: string, sources: HtmlCustody["sources"]): string | null {
  let stat: fs.Stats;
  try {
    stat = fs.lstatSync(marker);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
  if (stat.isDirectory() && !stat.isSymbolicLink()) return marker;
  const bytes = read(marker);
  if (bytes == null) throw new Error(`Git metadata disappeared: ${marker}`);
  sources.set(marker, bytes);
  const directory = regularDirectory(
    path.resolve(path.dirname(marker), metadataPath(bytes, marker, "gitdir: ")),
  );
  const commonFile = path.join(directory, "commondir");
  const common = read(commonFile);
  sources.set(commonFile, common);
  return common == null
    ? directory
    : regularDirectory(path.resolve(directory, metadataPath(common, commonFile)));
}

/** Snapshots prove identity, and do not perform native ignore selection. */
export function captureHtmlCustody(
  options: OxlintHtmlOptions,
  candidates: ReadonlySet<string>,
): HtmlCustody {
  const sources = new Map<string, Buffer | null>([
    [options.rootJson, Buffer.from(options.rootBytes)],
  ]);
  const originals = new Map<string, Buffer>();
  const directories = new Set<string>();
  for (const candidate of [options.cwd, ...candidates]) {
    const stat = fs.lstatSync(candidate);
    if (isStandaloneHtmlFile(candidate) && stat.isFile() && !stat.isSymbolicLink())
      originals.set(candidate, fs.readFileSync(candidate));
    let directory = stat.isDirectory() ? candidate : path.dirname(candidate);
    for (;;) {
      directories.add(directory);
      const parent = path.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }
  for (const directory of directories) {
    for (const name of [".gitignore", options.customIgnoreFilename]) {
      const file = path.join(directory, name);
      sources.set(file, read(file));
    }
    const git = gitDirectory(path.join(directory, ".git"), sources);
    if (git != null) {
      const file = path.join(git, "info/exclude");
      sources.set(file, read(file));
    }
  }
  return { sources, originals };
}

/** Keep full source/provider/ignore bytes bound before and after the one operation. */
export function assertHtmlCustody(context: HtmlContext, completed?: OxlintHtmlCompleted): void {
  if (!context.custody || !context.provider || !context.options)
    throw new Error("original HTML custody is unavailable");
  const { custody, provider, options } = context;
  if (
    digest(fs.readFileSync(provider.entrypoint)) !== provider.sha256 ||
    !Buffer.from(provider.packageBytes).equals(fs.readFileSync(provider.package))
  )
    throw new Error("actual host provider changed during HTML execution");
  for (const [file, expected] of [...custody.sources, ...custody.originals])
    if (!same(expected, read(file)))
      throw new Error(`original HTML input changed during real host execution: ${file}`);
  if (!completed) return;
  if (
    completed.hostProfile !== options.hostProfile ||
    completed.cwd !== options.cwd ||
    completed.literalTarget !== options.literalTarget ||
    completed.rootJson !== options.rootJson ||
    completed.format !== options.format ||
    completed.engineConfigValidation !== "not-performed" ||
    Object.keys(options.presentation).some(
      (key) =>
        completed.presentation[key as keyof typeof options.presentation] !==
        options.presentation[key as keyof typeof options.presentation],
    )
  )
    throw new Error("complete HTML result lost original execution/presentation identity");
  for (const original of completed.originals) {
    const prior = custody.originals.get(original.path);
    if (!prior || !prior.equals(Buffer.from(original.bytes)))
      throw new Error(`native HTML original lacks pre-host custody: ${original.path}`);
  }
  for (const source of completed.sources) {
    if (
      !custody.sources.has(source.path) ||
      !same(
        custody.sources.get(source.path) ?? null,
        source.bytes == null ? null : Buffer.from(source.bytes),
      )
    )
      throw new Error(`native HTML authority lacks pre-host custody: ${source.path}`);
  }
  if (
    completed.executedFileCount !== completed.originals.length ||
    completed.files.length !== completed.originals.length ||
    !Number.isFinite(completed.elapsedSeconds) ||
    completed.elapsedSeconds < 0
  )
    throw new Error("complete HTML execution counts are invalid");
}
