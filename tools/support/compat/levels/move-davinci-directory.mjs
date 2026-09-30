// Replay on fresh main after a conflict. Commit --move-only before --references.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { moveFoundation } from "./move-foundation.mjs";
import { moveSharedTypes } from "./move-shared-types.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const names = [
  "vize_davinci",
  "vize_davinci_derive",
  "vize_l0",
  "vize_l1",
  "vize_l2",
  "vize_l3",
  "vize_l4",
  "vize_l1_to_l2",
  "vize_l2_to_l3",
  "vize_extension_contract",
  "vize_extension_host",
  "vize_guest",
  "vize_dialect_moonbit",
];
const mode = process.argv[2];
if (
  ![
    "--move-only",
    "--references",
    "--shared-moves-only",
    "--shared-types",
    "--foundation-moves-only",
    "--foundation",
  ].includes(mode)
)
  throw new Error("expected --move-only, --references, --shared-moves-only or --shared-types");
if (mode.startsWith("--foundation")) {
  moveFoundation(root, mode);
  process.exit(0);
}
if (mode.startsWith("--shared")) {
  moveSharedTypes(root, mode);
  process.exit(0);
}

if (mode === "--move-only") {
  mkdirSync(path.join(root, "davinci"), { recursive: true });
  for (const name of names) {
    const before = path.join(root, "crates", name),
      after = path.join(root, "davinci", name);
    if (existsSync(before)) {
      if (existsSync(after)) throw new Error(`both directories exist: ${name}`);
      renameSync(before, after);
    }
    if (!existsSync(after)) throw new Error(`missing Davinci crate: ${name}`);
  }
} else {
  const files = execFileSync("git", ["ls-files", "-z"], { cwd: root, maxBuffer: 8 * 1024 * 1024 })
    .toString()
    .split("\0")
    .filter(Boolean);
  for (const file of files) {
    // Captured oracle bytes and upstream provenance retain their original paths.
    if (file.includes("/vize_guest/versions/")) continue;
    if (/\.(snap|input\.txt|expected\.json|bin)$/u.test(file)) continue;
    if (file.includes("/fixtures/") && !/\.(rs|toml)$/u.test(file)) continue;
    const absolute = path.join(root, file);
    if (!statSync(absolute).isFile()) continue;
    const buffer = readFileSync(absolute);
    if (buffer.includes(0)) continue;
    const source = buffer.toString("utf8");
    let next = source;
    for (const name of names) {
      next = next.replaceAll(`crates/${name}`, `davinci/${name}`);
      next = next.replaceAll(`"crates", "${name}"`, `"davinci", "${name}"`);
      next = next.replaceAll(`'crates', '${name}'`, `'davinci', '${name}'`);
      next = next.replace(
        new RegExp(`(["'])crates\\1(,\\s*)(["'])${name}\\3`, "gu"),
        `$1davinci$1$2$3${name}$3`,
      );
      next = next.replaceAll(`crates\\/${name}`, `davinci\\/${name}`);
    }
    const legacyFoundation = "vize_" + "carton";
    next = next.replaceAll(`crates/${legacyFoundation}/src/`, "davinci/vize_l0/src/");
    next = next.replaceAll(`"crates", "${legacyFoundation}", "src"`, '"davinci", "vize_l0", "src"');
    // Synthetic historical metadata is independent of the live directory split.
    next = next.replaceAll("/fixtures/crates/", "/fixtures/crates/");
    if (next !== source) writeFileSync(absolute, next);
  }
}
