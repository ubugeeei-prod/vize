import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import type { OxlintHtmlOptions, OxlintHtmlOutcome } from "@vizejs/native";

import { getLintTargets, withoutLintTargets } from "./args.ts";
import { readScopedConfig } from "./scoped-config.ts";
import { loadBinding } from "../native.ts";
import { captureHtmlCustody, assertHtmlCustody, type HtmlCustody } from "./html-custody.ts";
import { presentationContext, implicitDefault } from "./presentation-context.ts";

export interface HtmlContext {
  options?: OxlintHtmlOptions;
  failure?: string;
  provider?: {
    requestedEntrypoint: string;
    entrypoint: string;
    sha256: string;
    package: string;
    packageBytes: number[];
    version: string;
  };
  environment: Record<string, string | null>;
  outcome?: OxlintHtmlOutcome;
  custody?: HtmlCustody;
}

const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** Capture original root/provider/piped-child context without executing a provider query. */
export function prepareHtmlContext(
  cwd: string,
  args: readonly string[],
  entrypoint: string,
  version: string,
  candidates: ReadonlySet<string>,
): HtmlContext {
  const context: HtmlContext = {
    environment: presentationContext(),
  };
  try {
    if (!["1.78.0", "1.86.0"].includes(version))
      throw new Error("original HTML requires a qualified Oxlint 1.78.0 or 1.86.0 host");
    const targets = getLintTargets(args);
    if (targets.length !== 1) throw new Error("original HTML requires one literal original target");
    const config = readScopedConfig(cwd, args);
    if (!config || config.module || path.dirname(config.file) !== cwd)
      throw new Error("original HTML requires one regular JSON root in the original cwd");
    const plugin = fileURLToPath(new URL("./index.mjs", import.meta.url));
    const pluginRoot = import.meta.url.endsWith(".ts")
      ? fileURLToPath(new URL("../../dist/index.mjs", import.meta.url))
      : plugin;
    const owned = fs.realpathSync(pluginRoot);
    if (
      !Array.isArray(config.value.jsPlugins) ||
      !config.value.jsPlugins.some((specifier) => ownsPlugin(specifier, config.file, owned))
    )
      throw new Error(
        "original HTML requires this wrapper's genuine Vize plugin in the original root",
      );
    const actual = fs.realpathSync(entrypoint);
    const packageFile = path.join(path.dirname(actual), "../package.json");
    const packageBytes = fs.readFileSync(packageFile);
    const metadata = JSON.parse(packageBytes.toString("utf8")) as {
      name?: string;
      version?: string;
    };
    if (metadata.name !== "oxlint" || metadata.version !== version)
      throw new Error("original HTML provider package does not match the real version handshake");
    context.provider = {
      requestedEntrypoint: entrypoint,
      entrypoint: actual,
      sha256: digest(fs.readFileSync(actual)),
      package: packageFile,
      packageBytes: Array.from(packageBytes),
      version,
    };
    let format = "default";
    let explicitFormat = false;
    let noIgnore = false;
    const patterns: string[] = [];
    let customIgnoreFilename = ".eslintignore";
    const options = withoutLintTargets(args);
    for (let index = 0; index < options.length; index++) {
      const arg = options[index];
      if (arg === "--") break;
      if (["-c", "--config", "--threads"].includes(arg)) {
        index++;
        continue;
      }
      if (arg.startsWith("--config=") || arg.startsWith("--threads=")) continue;
      if (arg === "-f" || arg === "--format") {
        explicitFormat = true;
        format = options[++index];
        continue;
      }
      if (arg.startsWith("--format=")) {
        explicitFormat = true;
        format = arg.slice(9);
        continue;
      }
      if (arg === "--ignore-pattern") {
        patterns.push(options[++index]);
        continue;
      }
      if (arg.startsWith("--ignore-pattern=")) {
        patterns.push(arg.slice(17));
        continue;
      }
      if (arg === "--ignore-path") {
        customIgnoreFilename = options[++index];
        continue;
      }
      if (arg.startsWith("--ignore-path=")) {
        customIgnoreFilename = arg.slice(14);
        continue;
      }
      if (arg === "--no-ignore") {
        noIgnore = true;
        continue;
      }
      if (
        arg === "--deny-warnings" ||
        arg === "--no-error-on-unmatched-pattern" ||
        arg === "--disable-nested-config" ||
        arg === "--type-aware" ||
        arg === "--type-check" ||
        arg === "--import-plugin" ||
        arg === "--vue-plugin"
      )
        continue;
      if (arg === "--tsconfig") {
        index++;
        continue;
      }
      if (arg.startsWith("--tsconfig=")) continue;
      throw new Error(
        `original HTML cannot preserve this option's presentation/write/severity contract: ${arg}`,
      );
    }
    if (
      path.basename(customIgnoreFilename) !== customIgnoreFilename ||
      [".", ".."].includes(customIgnoreFilename)
    )
      throw new Error("original HTML requires one custom-ignore filename");
    if (!["default", "json", "unix", "stylish"].includes(format))
      throw new Error("original HTML requires an explicit default, json, unix or stylish report");
    if (!explicitFormat && !implicitDefault(context.environment))
      throw new Error(
        "original HTML requires an explicit supported format when the host selects agent/GitHub output",
      );
    const forceColor =
      context.environment.CI != null ||
      (context.environment.FORCE_COLOR != null && context.environment.FORCE_COLOR !== "0");
    context.options = {
      cwd,
      literalTarget: targets[0],
      rootJson: config.file,
      rootBytes: Array.from(fs.readFileSync(config.file)),
      hostProfile: version,
      noIgnore,
      cliIgnorePatterns: patterns,
      customIgnoreFilename,
      format,
      presentation: {
        cwd,
        graphicalTheme: forceColor ? "color" : "plain",
        links: false,
        width: 400,
        stylishNoColor: context.environment.NO_COLOR != null,
        stylishRelative: true,
      },
    };
    context.custody = captureHtmlCustody(context.options, candidates);
  } catch (error) {
    context.failure = error instanceof Error ? error.message : String(error);
  }
  return context;
}

/** One genuine native operation after normal stock/Vue setup, never a carrier. */
export function executeHtml(context: HtmlContext): OxlintHtmlOutcome {
  if (context.failure || !context.options)
    throw new Error(context.failure ?? "original HTML context is unavailable");
  const binding = loadBinding();
  if (typeof binding.lintOxlintHtml !== "function")
    throw new Error("installed native binding does not expose original HTML execution");
  assertHtmlCustody(context);
  if (!Buffer.from(context.options.rootBytes).equals(fs.readFileSync(context.options.rootJson)))
    throw new Error("original root bytes changed during real host setup");
  const outcome = binding.lintOxlintHtml(context.options);
  context.outcome = outcome;
  if (Boolean(outcome.completed) === Boolean(outcome.refused))
    throw new Error("original HTML operation did not return an exclusive complete/refused result");
  if (outcome.refused) throw new Error(`${outcome.refused.kind}: ${outcome.refused.details}`);
  assertHtmlCustody(context, outcome.completed);
  return outcome;
}

function ownsPlugin(specifier: unknown, configFile: string, owned: string): boolean {
  if (typeof specifier !== "string") return false;
  try {
    if (specifier.startsWith("file:")) return fs.realpathSync(fileURLToPath(specifier)) === owned;
    if (path.isAbsolute(specifier) || specifier.startsWith("./") || specifier.startsWith("../"))
      return fs.realpathSync(path.resolve(path.dirname(configFile), specifier)) === owned;
    const parts = specifier.split("/");
    if (parts.length !== (specifier.startsWith("@") ? 2 : 1)) return false;
    // This owned package is import-only. require.resolve cannot establish its
    // ESM identity. Bind its nearest actual package metadata and known import
    // entry, without depending on newer Node resolver APIs or running a query.
    const ownedMetadata = fs.realpathSync(path.join(path.dirname(owned), "../package.json"));
    let directory = path.dirname(configFile);
    for (;;) {
      const candidate = path.join(directory, "node_modules", specifier, "package.json");
      if (fs.existsSync(candidate))
        return (
          fs.realpathSync(candidate) === ownedMetadata &&
          fs.realpathSync(path.join(path.dirname(candidate), "dist/index.mjs")) === owned
        );
      const parent = path.dirname(directory);
      if (parent === directory) return false;
      directory = parent;
    }
  } catch {
    return false;
  }
}
