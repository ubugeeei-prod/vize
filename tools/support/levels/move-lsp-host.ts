#!/usr/bin/env node
// Replay #6834 LSP host ownership after the portable UTF-16 provider.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const moves = [
  ["davinci/vize_l0/src/lsp.rs", "crates/vize_carton/src/lsp.rs"],
  [
    "davinci/vize_l0/src/snapshots/vize_l0__lsp__tests__lsp_message_format.snap",
    "crates/vize_carton/src/snapshots/vize_carton__lsp__tests__lsp_message_format.snap",
  ],
  [
    "davinci/vize_l0/tests/line_index_coordinate_identity.rs",
    "crates/vize_carton/tests/line_index_coordinate_identity.rs",
  ],
] as const;
const hostModule = `#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host JSON-RPC records retain the existing std String and Vec contract"
)]
pub mod lsp;
`;
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
function update(file: string, transform: (source: string) => string): void {
  const original = read(file);
  const changed = transform(original);
  if (changed !== original) fs.writeFileSync(path.join(root, file), changed);
}
function replaceOnce(file: string, old: string, replacement: string): void {
  update(file, (source) => {
    if (source.includes(old)) {
      if (source.split(old).length !== 2) throw new Error(`Ambiguous integration: ${file}`);
      return source.replace(old, replacement);
    }
    if (!source.includes(replacement)) throw new Error(`Changed integration contract: ${file}`);
    return source;
  });
}

const phase = process.argv[2];
if (!["moves", "integrate", "check"].includes(phase)) {
  throw new Error("Usage: vp node tools/support/levels/move-lsp-host.ts moves|integrate|check");
}
const index = read("davinci/vize_l0/src/line_index.rs");
if (!index.includes("pub use position::{Position, Range};") || index.includes("crate::lsp")) {
  throw new Error("Missing portable UTF-16 provider; do not move the host first");
}
for (const [old, target] of moves) {
  const sourceExists = fs.existsSync(path.join(root, old));
  const targetExists = fs.existsSync(path.join(root, target));
  if (sourceExists === targetExists)
    throw new Error(`Missing move or source/target collision: ${old}`);
  if (phase === "moves" && sourceExists) {
    fs.mkdirSync(path.dirname(path.join(root, target)), { recursive: true });
    execFileSync("git", ["mv", old, target], { cwd: root, stdio: "inherit" });
  } else if (phase !== "moves" && sourceExists) {
    throw new Error(`Unmoved host source: ${old}`);
  }
}
if (phase === "integrate") {
  update("davinci/vize_l0/src/lib.rs", (source) => source.replace("pub mod lsp;\n", ""));
  update("crates/vize_carton/src/lib.rs", (source) =>
    source.includes(hostModule)
      ? source
      : source.replace("\npub mod lsp;\n", "") + "\n" + hostModule,
  );
  replaceOnce(
    "crates/vize_carton/src/lsp.rs",
    "pub use crate::line_index::{Position, Range};",
    "pub use vize_l0::line_index::{Position, Range};",
  );
  for (const name of ["Position", "Range"]) {
    replaceOnce(
      "crates/vize_carton/tests/line_index_coordinate_identity.rs",
      `vize_l0::lsp::${name}`,
      `vize_carton::lsp::${name}`,
    );
  }
  update("crates/vize_carton/Cargo.toml", (source) => {
    const binding = "vize_l0 = { workspace = true }\n";
    if (source.split(binding).length !== 2) throw new Error("Changed Carton foundation dependency");
    return source
      .split("\n")
      .filter((line) => !["serde.workspace = true", "serde_json.workspace = true"].includes(line))
      .join("\n")
      .replace(binding, binding + "serde.workspace = true\nserde_json.workspace = true\n");
  });
}
if (phase !== "moves") {
  if (read("davinci/vize_l0/src/lib.rs").includes("pub mod lsp;")) {
    throw new Error("L0 still owns the host protocol");
  }
  if (!read("crates/vize_carton/src/lib.rs").includes(hostModule)) {
    throw new Error("Carton has not integrated its scoped host module");
  }
  if (
    !read("crates/vize_carton/src/lsp.rs").includes(
      "pub use vize_l0::line_index::{Position, Range};",
    )
  ) {
    throw new Error("Host coordinate identity is not the L0 provider");
  }
}
