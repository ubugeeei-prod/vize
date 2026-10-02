import { createHash } from "node:crypto";
import { existsSync, readFileSync, renameSync } from "node:fs";
import { resolve } from "node:path";

// Exact frozen corpus bytes; valid and malformed inputs stay equally authored.
const moves = [
  {
    from: "tests/fuzz/regressions/l1_program/native-module.js",
    to: "tests/fuzz/regressions/l1_program/native-module.js.input",
    sha256: "734d3c83a5d641f07b083ba97036a71daa5fa74c09ef8de7897df0afa2c3243e",
  },
  {
    from: "tests/fuzz/regressions/l1_program/native-module.tsx",
    to: "tests/fuzz/regressions/l1_program/native-module.tsx.input",
    sha256: "a2fa37c7423b29f92f797b76abb06fab6ad3b365bbc2d854b3cb279086534d6e",
  },
  {
    from: "tests/fuzz/regressions/l1_program/pure-recovery.js",
    to: "tests/fuzz/regressions/l1_program/pure-recovery.js.input",
    sha256: "70fa10164b219c26375100fa28b763bc12fce85423b19932c2969cdcfa2fb979",
  },
  {
    from: "tests/fuzz/regressions/l1_program/pure-rewind.js",
    to: "tests/fuzz/regressions/l1_program/pure-rewind.js.input",
    sha256: "4d512b93149545b991c03f143efabaf7510feacff8125724edc6b0610ccf1ec7",
  },
  {
    from: "tests/fuzz/regressions/l1_program/pure-ts-operator.ts",
    to: "tests/fuzz/regressions/l1_program/pure-ts-operator.ts.input",
    sha256: "cb91e97e41d2202545925f335bd74794a9176bd2d0fd8c6dd29803ae7cce5e54",
  },
];

for (const { from, to, sha256 } of moves) {
  const source = resolve(from);
  const destination = resolve(to);
  const hasSource = existsSync(source);
  const hasDestination = existsSync(destination);
  if (hasSource === hasDestination) throw new Error(`Expected exactly one of ${from} and ${to}`);
  const input = readFileSync(hasSource ? source : destination);
  if (createHash("sha256").update(input).digest("hex") !== sha256) {
    throw new Error(`Frozen corpus bytes differ: ${from}`);
  }
  if (hasSource) renameSync(source, destination);
}
