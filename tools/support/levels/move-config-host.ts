import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const core = "davinci/vize_l0/src/config";
const host = "crates/vize_carton/src/config";
const read = (path: string) => readFileSync(join(root, path), "utf8");
const write = (path: string, value: string) => {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), value);
};
const exists = (path: string) => existsSync(join(root, path));
const requireCondition = (condition: boolean, message: string) => {
  if (!condition) throw new Error(message);
};
const replace = (path: string, before: string, after: string) => {
  const value = read(path);
  if (value.includes(before)) write(path, value.replaceAll(before, after));
  else requireCondition(value.includes(after), `unexpected integration state: ${path}`);
};
requireCondition(
  exists(`${core}/document.rs`) &&
    read(`${core}/document.rs`).includes("pub struct ConfigDocument(RawVizeConfig);"),
  "the in-memory ConfigDocument provider must exist first",
);

const moves = [
  [`${core}/loader.rs`, `${host}/loader.rs`],
  [`${core}/loader`, `${host}/loader`],
];
const oldSnapshot = `${host}/loader/snapshots/vize_l0__config__loader__tests__validate_explicit_config_path_malformed_errors.snap`;
const newSnapshot = oldSnapshot.replace("vize_l0__", "vize_carton__");
const coreRoot = `${core}.rs`;
const hostRoot = `${host}.rs`;
const prefix = `//! Host discovery and evaluation of Vize configuration.\n\n#[expect(\n    clippy::disallowed_types,\n    clippy::disallowed_methods,\n    reason = "host config evaluation retains its existing std error and process strings"\n)]\nmod loader;\n`;
const suffix = "pub use vize_l0::config::*;\n";
const projectDeclarations = () => (exists(`${host}/project.rs`) ? "mod project;\n" : "");
const projectExports = () =>
  exists(`${host}/project.rs`) ? "pub use project::ProjectModel;\n" : "";
const loaderExports = /pub use loader::\{[\s\S]*?\n\};\n/;
const bridgeInventory = "docs/davinci/plan/foundation-storage-bridges.json";
const retiredBridge =
  "davinci/vize_l0/src/config/loader.rs: forbidden std storage path: std::string::String";
const hostGate = "tests/tooling/support/davinci-stage-dependencies.ts";
const hostGateImport = 'import { withoutHostRuntimeReferences } from "./davinci-host-imports.ts";';
const oldHostMask = String.raw`source.replace(/\bvize_carton::corsa_(?:api_mode|resolver)\b/gu, "host_runtime")`;
const newHostMask = "withoutHostRuntimeReferences(source, path.relative(repoRoot, fullPath))";

function checkMoves() {
  for (const [source, target] of moves) {
    requireCondition(
      !exists(source) && exists(target),
      `unmoved source or missing target: ${source}`,
    );
  }
  requireCondition(
    !exists(oldSnapshot) && exists(newSnapshot),
    "snapshot owner name must move with unchanged contents",
  );
}

function move() {
  for (const [source, target] of moves) {
    requireCondition(
      exists(source) !== exists(target),
      `source/target collision or missing input: ${source}`,
    );
  }
  requireCondition(!(exists(oldSnapshot) && exists(newSnapshot)), "snapshot collision");
  const sourceSnapshot = oldSnapshot.replace(`${host}/`, `${core}/`);
  requireCondition(
    [sourceSnapshot, oldSnapshot, newSnapshot].filter(exists).length === 1,
    "missing or ambiguous fixed snapshot",
  );
  for (const [source, target] of moves) {
    if (exists(source)) {
      mkdirSync(dirname(join(root, target)), { recursive: true });
      renameSync(join(root, source), join(root, target));
    }
  }
  if (exists(oldSnapshot)) renameSync(join(root, oldSnapshot), join(root, newSnapshot));
  checkMoves();
}

function integrate() {
  checkMoves();
  const value = read(coreRoot);
  const exports =
    value.match(loaderExports)?.[0] ??
    (exists(hostRoot) ? read(hostRoot).match(loaderExports)?.[0] : undefined);
  requireCondition(exports !== undefined, "missing host loader exports");
  const expectedHost = prefix + projectDeclarations() + exports + projectExports() + suffix;
  requireCondition(
    !exists(hostRoot) || read(hostRoot) === expectedHost,
    "unexpected existing Carton config module",
  );
  const lib = "crates/vize_carton/src/lib.rs";
  const callerChanges = [
    ["crates/vize/src/config.rs", "pub use vize_l0::config::*;", "pub use vize_carton::config::*;"],
    [
      "crates/vize_maestro/src/server/state/config.rs",
      "vize_l0::config::load",
      "vize_carton::config::load",
    ],
    [
      "crates/vize_maestro/src/server/state/workspace_folders.rs",
      "let (loaded, plan, _) = vize_l0::config::",
      "let (loaded, plan, _) = vize_carton::config::",
    ],
    [hostGate, oldHostMask, newHostMask],
  ];
  requireCondition(
    exists("tests/tooling/support/davinci-host-imports.ts"),
    "the published host-import gate companion must accompany replay",
  );
  requireCondition(
    read(hostGate).includes('import { parse as parseToml } from "@iarna/toml";'),
    "unexpected host gate imports",
  );
  requireCondition(
    read(lib).includes("pub mod config;") || read(lib).includes("pub use vize_l0::*;\n"),
    "unexpected Carton module root",
  );
  for (const [path, before, after] of callerChanges) {
    requireCondition(
      read(path).includes(before) || read(path).includes(after),
      `unexpected caller: ${path}`,
    );
  }
  const lock = read("Cargo.lock");
  const start = lock.indexOf('name = "vize_l0"\n');
  const end = lock.indexOf("\n[[package]]", start);
  requireCondition(start >= 0 && end > start, "missing L0 lock entry");
  requireCondition(
    read("davinci/vize_l0/Cargo.toml").includes('name = "vize_l0"'),
    "unexpected provider manifest",
  );
  write(hostRoot, expectedHost);
  write(
    coreRoot,
    value
      .replace("mod loader;\n", "")
      .replace(loaderExports, "")
      .replace(
        "//! Shared Vize configuration loading.",
        "//! In-memory Vize configuration values and projections.",
      ),
  );
  if (!read(lib).includes("pub mod config;"))
    replace(lib, "pub use vize_l0::*;\n", "pub use vize_l0::*;\n\npub mod config;\n");
  for (const [path, before, after] of callerChanges) replace(path, before, after);
  if (!read(hostGate).includes(hostGateImport)) {
    replace(
      hostGate,
      'import { parse as parseToml } from "@iarna/toml";',
      'import { parse as parseToml } from "@iarna/toml";\n' + hostGateImport,
    );
  }
  write(
    "davinci/vize_l0/Cargo.toml",
    read("davinci/vize_l0/Cargo.toml").replace("pklrust = { workspace = true }\n", ""),
  );
  write(
    "Cargo.lock",
    lock.slice(0, start) + lock.slice(start, end).replace(' "pklrust",\n', "") + lock.slice(end),
  );
  write(bridgeInventory, read(bridgeInventory).replace(`    "${retiredBridge}",\n`, ""));
  check();
}

function check() {
  checkMoves();
  const exports = read(hostRoot).match(loaderExports)?.[0];
  requireCondition(
    exports !== undefined &&
      read(hostRoot) === prefix + projectDeclarations() + exports + projectExports() + suffix,
    "unexpected Carton host exports",
  );
  requireCondition(
    !read(coreRoot).includes("mod loader;") && !loaderExports.test(read(coreRoot)),
    "L0 still exports host loaders",
  );
  requireCondition(
    read("crates/vize/src/config.rs").includes("pub use vize_carton::config::*;"),
    "CLI must use the host loader owner",
  );
  requireCondition(
    !read("crates/vize_maestro/src/server/state/config.rs").includes("vize_l0::config::load"),
    "LSP must use the host loader owner",
  );
  requireCondition(
    read("crates/vize_maestro/src/server/state/workspace_folders.rs").includes(
      "let (loaded, plan, _) = vize_carton::config::",
    ),
    "workspace config must use the host loader owner",
  );
  requireCondition(
    !read("davinci/vize_l0/Cargo.toml").includes("pklrust"),
    "L0 must not retain the unused PKL host dependency",
  );
  requireCondition(
    !read(bridgeInventory).includes(retiredBridge),
    "retired L0 loader witness must leave the exact inventory",
  );
  requireCondition(
    read(hostGate).includes(hostGateImport) &&
      read(hostGate).includes(newHostMask) &&
      !read(hostGate).includes(oldHostMask),
    "stage gate must distinguish actual host config calls from Carton storage imports",
  );
}

switch (process.argv[2]) {
  case "moves":
    move();
    break;
  case "integrate":
    integrate();
    break;
  case "check":
    check();
    break;
  default:
    throw new Error(
      "usage: vp node tools/support/levels/move-config-host.ts moves|integrate|check",
    );
}
