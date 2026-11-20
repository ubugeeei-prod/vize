// The lint substrate ledger (#6848): every Patina rule, the level it belongs
// on and the engine that runs it today. `--write`/`--check` keep the committed
// per-rule ledger in step with source; `--summary` emits the count tables for
// the generated run bundle (see generated-ledgers.ts).
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  LEDGER_REL,
  REGEN,
  buildLedger,
  ledgerPath,
  renderLedger,
  renderSummary,
} from "./lib/lint-substrate.ts";

const usage =
  "usage: node tools/support/compat/davinci/lint-substrate.ts --write | --check | --summary [--ledger <path>]";

function main(argv: readonly string[]): number {
  const [mode, ...options] = argv;
  if (!["--write", "--check", "--summary"].includes(mode)) {
    console.error(usage);
    return 2;
  }
  let target = ledgerPath();
  if (options.length > 0) {
    if (options.length !== 2 || options[0] !== "--ledger" || mode === "--summary") {
      console.error(usage);
      return 2;
    }
    target = path.resolve(options[1]);
  }
  const rows = buildLedger();
  if (mode === "--summary") {
    process.stdout.write(renderSummary(rows));
    return 0;
  }
  const text = renderLedger(rows);
  if (mode === "--write") {
    writeFileSync(target, text);
    console.log(`wrote ${target}`);
    return 0;
  }
  const shown = target === ledgerPath() ? LEDGER_REL : target;
  if (!existsSync(target)) {
    console.error(`stale: ${shown} does not exist; run ${REGEN}`);
    return 1;
  }
  if (readFileSync(target, "utf8") !== text) {
    console.error(`stale: ${shown} drifted from the rule sources; run ${REGEN}`);
    return 1;
  }
  console.log(`${shown} is up to date`);
  return 0;
}

process.exitCode = main(process.argv.slice(2));
