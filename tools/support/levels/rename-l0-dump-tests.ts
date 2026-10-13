import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

// Historical source names remain in the foundation ownership replay.
const root = fileURLToPath(new URL("../../../", import.meta.url));
const names = [
  ["folio_derive_laws.rs", "dump_derive_laws.rs"],
  ["folio_dump.rs", "dump_hash_gate.rs"],
];
const directory = "davinci/vize_l0/tests/";
const phase = process.argv[2];
if (!["moves", "integrate", "check"].includes(phase)) {
  throw new Error("Usage: rename-l0-dump-tests.ts moves|integrate|check");
}

// Validate the entire move set before changing any index entry.
for (const [oldName, newName] of names) {
  const oldExists = existsSync(resolve(root, directory + oldName));
  const newExists = existsSync(resolve(root, directory + newName));
  if (oldExists === newExists) {
    throw new Error(`Expected exactly one of ${oldName} and ${newName}`);
  }
  if (phase !== "moves" && oldExists) throw new Error(`Unmoved target: ${oldName}`);
}
if (phase === "moves") {
  for (const [oldName, newName] of names) {
    if (existsSync(resolve(root, directory + oldName))) {
      execFileSync("git", ["mv", directory + oldName, directory + newName], { cwd: root });
    }
  }
} else {
  // The Python foundation replay this once patched was removed with the
  // other replayed one-shot scripts; only the plan record is still integrated.
  const updates = new Map<string, string>();
  const records = "docs/davinci/plan/phase-2-records.md";
  let source = readFileSync(resolve(root, records), "utf8");
  const oldLink = "[`folio_derive_laws`](../../../davinci/vize_l0/tests/folio_derive_laws.rs)";
  const newLink = "[`dump_derive_laws`](../../../davinci/vize_l0/tests/dump_derive_laws.rs)";
  if (source.includes(oldLink)) source = source.replace(oldLink, newLink);
  else if (!source.includes(newLink)) throw new Error("Missing current foundation law link");
  updates.set(records, source);

  // All integration preconditions are checked before any tracked write.
  for (const [path, changed] of updates) {
    const before = readFileSync(resolve(root, path), "utf8");
    if (before !== changed) {
      if (phase === "check") throw new Error(`Unintegrated reference: ${path}`);
      writeFileSync(resolve(root, path), changed);
    }
  }
}
