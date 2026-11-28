import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import type { OxlintTsrxOptions } from "@vizejs/native";
import { getLintTargets, withoutLintTargets } from "./args.ts";
import { collectVueLikeFilesFromTargets } from "./files.ts";
import { captureHtmlCustody } from "./html-custody.ts";
import type { ScopedConfig } from "./scoped-config.ts";
import { isTsrxFile } from "../file-kinds.ts";
import { loadBinding } from "../native.ts";

export interface TsrxHost {
  version: string;
  entrypoint: string;
}
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** Capture original authority before the real host loads config or plugin code. */
export function prepareTsrxSelection(
  cwd: string,
  args: readonly string[],
  config: ScopedConfig,
  candidates: ReadonlySet<string>,
  host?: TsrxHost,
) {
  if (!host || !["1.78.0", "1.81.0", "1.86.0"].includes(host.version))
    throw new Error(`TSRX transport requires a qualified Oxlint host (1.78.0, 1.81.0, 1.86.0).`);
  if (config.module || path.dirname(config.file) !== cwd)
    throw new Error(
      "TSRX transport requires one regular JSON config directly in the original cwd.",
    );
  const settings = config.value.settings as
    | { vize?: { pluginRules?: unknown; typeAware?: unknown } }
    | undefined;
  if (settings?.vize?.pluginRules === true)
    throw new Error("TSRX transport cannot preserve external vue.plugins execution.");
  if (settings?.vize?.typeAware === true)
    throw new Error("TSRX transport cannot preserve native type-aware execution.");
  const configured = config.value.rules as Record<string, unknown> | undefined;
  if (
    configured &&
    Object.entries(configured).some(
      ([name, setting]) =>
        name.startsWith("vize/type/") &&
        !["off", 0].includes(Array.isArray(setting) ? setting[0] : (setting as string)),
    )
  )
    throw new Error("TSRX transport cannot preserve native type-aware rules.");
  const options: OxlintTsrxOptions = {
    cwd,
    literalTargets: getLintTargets(args),
    rootJson: config.file,
    rootBytes: Array.from(Buffer.from(config.bytes)),
    hostProfile: host.version,
    noIgnore: false,
    cliIgnorePatterns: [],
    customIgnoreFilename: ".eslintignore",
  };
  const flags = withoutLintTargets(args);
  for (let index = 0; index < flags.length; index++) {
    const arg = flags[index];
    if (arg === "--ignore-pattern") options.cliIgnorePatterns.push(flags[++index]);
    else if (arg.startsWith("--ignore-pattern=")) options.cliIgnorePatterns.push(arg.slice(17));
    else if (arg === "--ignore-path") options.customIgnoreFilename = flags[++index];
    else if (arg.startsWith("--ignore-path=")) options.customIgnoreFilename = arg.slice(14);
  }
  options.noIgnore = flags.includes("--no-ignore");
  const custody = captureHtmlCustody(options, candidates, isTsrxFile);
  const originals = [...custody.originals.keys()].sort();
  const provider = fs.realpathSync(host.entrypoint);
  const providerDigest = digest(fs.readFileSync(provider));
  const metadata = path.resolve(path.dirname(provider), "../package.json");
  const metadataBytes = fs.readFileSync(metadata);
  const identity = JSON.parse(metadataBytes.toString("utf8"));
  if (identity.name !== "oxlint" || identity.version !== host.version)
    throw new Error("TSRX provider identity differs from the actual host handshake.");
  const assertCustody = () => {
    const currentOriginals = collectVueLikeFilesFromTargets(cwd, options.literalTargets).filter(
      isTsrxFile,
    );
    if (
      originals.length !== currentOriginals.length ||
      originals.some((file, index) => file !== currentOriginals[index])
    )
      throw new Error("TSRX original target membership changed during host execution.");
    if (
      digest(fs.readFileSync(provider)) !== providerDigest ||
      !fs.readFileSync(metadata).equals(metadataBytes)
    )
      throw new Error("TSRX host provider changed during execution.");
    for (const [file, bytes] of [...custody.sources, ...custody.originals]) {
      const stat = fs.lstatSync(file, { throwIfNoEntry: false });
      if (stat && (!stat.isFile() || stat.isSymbolicLink()))
        throw new Error(`TSRX authority is no longer regular: ${file}`);
      const current = stat ? fs.readFileSync(file) : null;
      if (bytes == null ? current != null : current == null || !bytes.equals(current))
        throw new Error(`TSRX original input changed during host execution: ${file}`);
    }
  };
  return {
    assertCustody,
    select(): string[] {
      assertCustody();
      const binding = loadBinding();
      if (typeof binding.selectOxlintTsrx !== "function")
        throw new Error(
          "The installed native binding does not expose original-path TSRX selection.",
        );
      const selected = binding.selectOxlintTsrx(options);
      assertCustody();
      for (const original of selected.originals) {
        const before = custody.originals.get(original.path);
        if (
          !candidates.has(original.path) ||
          !before ||
          !before.equals(Buffer.from(original.bytes)) ||
          !isTsrxFile(original.path)
        )
          throw new Error(`TSRX selection lost original source custody: ${original.path}`);
      }
      for (const source of selected.sources) {
        const before = custody.sources.get(source.path);
        const bytes = source.bytes == null ? null : Buffer.from(source.bytes);
        if (
          !custody.sources.has(source.path) ||
          (before == null ? bytes != null : bytes == null || !before.equals(bytes))
        )
          throw new Error(`TSRX selection lost original config/ignore custody: ${source.path}`);
      }
      return selected.originals.map((original) => original.path);
    },
  };
}
