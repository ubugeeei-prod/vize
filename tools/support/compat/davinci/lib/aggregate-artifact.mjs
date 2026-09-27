// Whole-repository aggregates are run artifacts. Keep their byte-exact check
// separate from the committed fixture and ratchet inventories.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.mjs";

export const LEDGER_OUTPUT_REL = "artifacts/davinci-ledgers";

export function parseAggregateArgs(argv, usage) {
  const [mode, ...options] = argv;
  if (!["--write", "--check", "--summary"].includes(mode)) throw new Error(usage);
  let outDir = path.join(repoRoot, LEDGER_OUTPUT_REL);
  if (options.length) {
    if (options.length !== 2 || options[0] !== "--out-dir" || mode === "--summary") {
      throw new Error(usage);
    }
    outDir = path.resolve(repoRoot, options[1]);
  }
  return { mode, outDir };
}

export function emitAggregate(name, text, { mode, outDir }) {
  if (mode === "--summary") {
    process.stdout.write(text);
    return;
  }
  const target = path.join(outDir, name);
  if (mode === "--write") {
    mkdirSync(outDir, { recursive: true });
    writeFileSync(target, text);
    console.log(`wrote ${target}`);
    return;
  }
  if (!existsSync(target)) {
    console.error(`stale: ${target} does not exist; generate the run artifacts with --write`);
    process.exitCode = 1;
  } else if (readFileSync(target, "utf8") !== text) {
    console.error(`stale: ${target} drifted from the current sources`);
    process.exitCode = 1;
  } else {
    console.log(`${target} is up to date`);
  }
}

export function aggregateMain(name, generate, usage) {
  let args;
  try {
    args = parseAggregateArgs(process.argv.slice(2), usage);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 2;
    return;
  }
  // Generation validates source contracts even when the saved artifact is absent.
  emitAggregate(name, generate(), args);
}
