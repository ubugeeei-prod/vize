import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const oldPath = "davinci/vize_l1_to_l2/tests/support/folio_spans.rs";
const newPath = "davinci/vize_l1_to_l2/tests/support/dump_spans.rs";
const supportPath = "davinci/vize_l1_to_l2/tests/support/mod.rs";
const decisionPath = "docs/davinci/decisions/2026-09-27-level-restructure.md";
const companionPath = "docs/davinci/decisions/2026-10-04-l2-span-support-name.md";
const sourceHash = "59aac2c9a3ee9d615551f42beb0dda95557302da1b5be735a2d33f3ecf36e364";
const phase = process.argv[2];
if (!["moves", "integrate", "check"].includes(phase)) {
  throw new Error("Usage: rename-l2-span-support.ts moves|integrate|check");
}

const hasOld = existsSync(resolve(root, oldPath));
const hasNew = existsSync(resolve(root, newPath));
if (hasOld === hasNew) throw new Error(`Expected exactly one of ${oldPath} and ${newPath}`);
if (phase !== "moves" && hasOld) throw new Error(`Unmoved support module: ${oldPath}`);
const source = readFileSync(resolve(root, hasOld ? oldPath : newPath));
if (createHash("sha256").update(source).digest("hex") !== sourceHash) {
  throw new Error("Changed original span oracle; review its source before replaying");
}

const references = [
  ["mod folio_spans;", "mod dump_spans;"],
  [
    "pub use folio_spans::assert_folio_spans_resolve;",
    "pub use dump_spans::assert_folio_spans_resolve;",
  ],
];
const support = readFileSync(resolve(root, supportPath), "utf8");
let rewrittenSupport = support;
const states = references.map(([before, after]) => {
  const oldCount = support.split(before).length - 1;
  const newCount = support.split(after).length - 1;
  if (oldCount + newCount !== 1) throw new Error(`Ambiguous support anchor: ${before}`);
  rewrittenSupport = rewrittenSupport.replace(before, after);
  return oldCount === 0;
});
if (states[0] !== states[1]) throw new Error("Partially integrated support module");

const decision = readFileSync(resolve(root, decisionPath), "utf8");
const original = "while `.folio` fixture bytes and extensions remain unchanged.";
const link = "[L2 span support naming](./2026-10-04-l2-span-support-name.md)";
const addition = ` ${link} moves one private span-oracle module byte-for-byte, updates only its module references, and preserves every law, diagnostic and fixture byte with unchanged dependency declarations.`;
if (!existsSync(resolve(root, companionPath))) throw new Error("Missing paired naming decision");
if (decision.split(original).length !== 2) throw new Error("Ambiguous naming decision anchor");
if (decision.includes(link) && !decision.includes(original + addition)) {
  throw new Error("Unexpected span support naming decision");
}
const rewrittenDecision = decision.includes(link)
  ? decision
  : decision.replace(original, original + addition);
if (phase === "check" && (support !== rewrittenSupport || decision !== rewrittenDecision)) {
  throw new Error("Unintegrated span support naming");
}

// All source, collision and integration checks finish before the first write.
if (phase === "moves") {
  if (hasOld) execFileSync("git", ["mv", oldPath, newPath], { cwd: root });
} else if (phase === "integrate") {
  if (support !== rewrittenSupport) writeFileSync(resolve(root, supportPath), rewrittenSupport);
  if (decision !== rewrittenDecision) writeFileSync(resolve(root, decisionPath), rewrittenDecision);
}
