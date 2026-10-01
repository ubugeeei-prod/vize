import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const source = "davinci/vize_l0/src/config/matcher.rs";
const target = "crates/vize_carton/src/config/matcher.rs";
const core = "davinci/vize_l0/src/config.rs";
const host = "crates/vize_carton/src/config.rs";
const coreManifest = "davinci/vize_l0/Cargo.toml";
const hostManifest = "crates/vize_carton/Cargo.toml";
const cliFacade = "crates/vize/src/lint_plan/matcher.rs";
const editor = "crates/vize_maestro/src/server/state/workspace_folders.rs";
const declaration = '#[cfg(feature = "lint-glob")]\npub mod matcher;\n';
const dependency = "globset = { workspace = true, optional = true }\n";
const read = (path: string) => readFileSync(join(root, path), "utf8");
const exists = (path: string) => existsSync(join(root, path));
const write = (path: string, content: string) => writeFileSync(join(root, path), content);
const requireCondition = (condition: boolean, message: string) => {
  if (!condition) throw new Error(message);
};
const replace = (path: string, before: string, after: string) => {
  const value = read(path);
  if (value.includes(before)) write(path, value.replaceAll(before, after));
  else requireCondition(value.includes(after), `unexpected integration state: ${path}`);
};
const lockPackage = (lock: string, name: string) => {
  const start = lock.indexOf(`name = "${name}"\n`);
  const end = lock.indexOf("\n[[package]]", start);
  requireCondition(start >= 0 && end > start, `missing lock package: ${name}`);
  return lock.slice(start, end);
};

requireCondition(
  exists(host) &&
    read(host).includes("mod loader;") &&
    read(host).includes("pub use vize_l0::config::*;"),
  "Carton must already own config loading and consume effective L0 types",
);

function checkMove() {
  requireCondition(
    !exists(source) && exists(target),
    "unmoved matcher source or missing host target",
  );
}

function move() {
  requireCondition(
    exists(source) !== exists(target),
    "matcher source/target collision or missing input",
  );
  if (exists(source)) {
    mkdirSync(dirname(join(root, target)), { recursive: true });
    renameSync(join(root, source), join(root, target));
  }
  checkMove();
}

function integrate() {
  checkMove();
  requireCondition(
    read(core).includes(declaration) || read(host).includes(declaration),
    "unexpected matcher module declaration",
  );
  requireCondition(
    read(hostManifest).includes("vize_l0 = { workspace = true }\n") &&
      read(hostManifest).includes("[dependencies]\n"),
    "unexpected Carton manifest",
  );
  requireCondition(
    read("tests/tooling/support/davinci-host-imports.ts").includes("host_config_matcher"),
    "the reviewed matcher host-import gate companion must accompany replay",
  );
  requireCondition(
    read(coreManifest).includes(dependency) || read(hostManifest).includes(dependency),
    "unexpected optional globset declaration",
  );
  const coreFeature = read(coreManifest)
    .split("\n")
    .filter((line) => /^\s*lint-glob\s*=/u.test(line));
  requireCondition(
    coreFeature.length === 0 ||
      (coreFeature.length === 1 && coreFeature[0] === 'lint-glob = ["dep:globset"]'),
    "unexpected L0 lint-glob feature",
  );
  requireCondition(
    read(hostManifest).includes('lint-glob = ["vize_l0/lint-glob"]') ||
      read(hostManifest).includes('lint-glob = ["dep:globset"]'),
    "unexpected host lint-glob feature",
  );
  for (const manifest of ["crates/vize/Cargo.toml", "crates/vize_maestro/Cargo.toml"]) {
    requireCondition(
      (read(manifest).includes('vize_l0 = { workspace = true, features = ["lint-glob"] }') ||
        read(manifest).includes("vize_l0 = { workspace = true }")) &&
        (read(manifest).includes("vize_carton.workspace = true") ||
          read(manifest).includes('vize_carton = { workspace = true, features = ["lint-glob"] }')),
      `unexpected matcher consumer manifest: ${manifest}`,
    );
  }
  requireCondition(
    read(cliFacade).includes("vize_l0::config::matcher") ||
      read(cliFacade).includes("vize_carton::config::matcher"),
    "unexpected CLI matcher facade",
  );
  requireCondition(
    read(editor).includes("    matcher::LintPlanScope,") ||
      read(editor).includes("use vize_carton::config::matcher::LintPlanScope;"),
    "unexpected LSP matcher import",
  );
  const lock = read("Cargo.lock");
  const hostPackage = lockPackage(lock, "vize_carton");
  const corePackage = lockPackage(lock, "vize_l0");
  requireCondition(
    corePackage.includes(' "globset",\n') || hostPackage.includes(' "globset",\n'),
    "globset must remain in an existing reviewed package dependency",
  );
  const updatedHostPackage = hostPackage.includes(' "globset",\n')
    ? hostPackage
    : hostPackage.replace(' "davinci_test_support",\n', ' "davinci_test_support",\n "globset",\n');
  requireCondition(
    updatedHostPackage.includes(' "globset",\n'),
    "unexpected Carton lock dependency layout",
  );
  write(core, read(core).replace(declaration, ""));
  if (!read(host).includes(declaration))
    replace(host, "mod loader;\n", "mod loader;\n" + declaration);
  write(
    coreManifest,
    read(coreManifest).replace(dependency, "").replace('lint-glob = ["dep:globset"]\n', ""),
  );
  if (!read(hostManifest).includes(dependency))
    replace(hostManifest, "[dependencies]\n", "[dependencies]\n" + dependency);
  replace(hostManifest, 'lint-glob = ["vize_l0/lint-glob"]', 'lint-glob = ["dep:globset"]');
  for (const manifest of ["crates/vize/Cargo.toml", "crates/vize_maestro/Cargo.toml"]) {
    replace(
      manifest,
      'vize_l0 = { workspace = true, features = ["lint-glob"] }',
      "vize_l0 = { workspace = true }",
    );
    replace(
      manifest,
      "vize_carton.workspace = true",
      'vize_carton = { workspace = true, features = ["lint-glob"] }',
    );
  }
  replace(cliFacade, "vize_l0::config::matcher", "vize_carton::config::matcher");
  write(
    editor,
    read(editor)
      .replace("    matcher::LintPlanScope,\n", "")
      .replace(
        "use vize_l0::config::{",
        "use vize_carton::config::matcher::LintPlanScope;\nuse vize_l0::config::{",
      )
      .replace(
        "use vize_carton::config::matcher::LintPlanScope;\nuse vize_carton::config::matcher::LintPlanScope;",
        "use vize_carton::config::matcher::LintPlanScope;",
      ),
  );
  write(
    "Cargo.lock",
    lock
      .replace(corePackage, corePackage.replace(' "globset",\n', ""))
      .replace(hostPackage, updatedHostPackage),
  );
  check();
}

function check() {
  checkMove();
  requireCondition(
    !read(core).includes("mod matcher;") && read(host).includes(declaration),
    "matcher must be owned only by Carton",
  );
  requireCondition(
    !read(coreManifest).includes("globset") && !read(coreManifest).includes("lint-glob"),
    "L0 must not retain the host matcher dependency or feature",
  );
  requireCondition(
    read(hostManifest).includes(dependency) &&
      read(hostManifest).includes('lint-glob = ["dep:globset"]'),
    "Carton must own optional globset and its feature",
  );
  const lock = read("Cargo.lock");
  requireCondition(
    !lockPackage(lock, "vize_l0").includes(' "globset",\n') &&
      lockPackage(lock, "vize_carton").includes(' "globset",\n'),
    "Cargo.lock must retain globset only at the Carton host owner",
  );
  requireCondition(
    !read(cliFacade).includes("vize_l0::config::matcher") &&
      read(cliFacade).includes("vize_carton::config::matcher"),
    "CLI must consume the host matcher",
  );
  requireCondition(
    !read(editor).includes("    matcher::LintPlanScope,") &&
      read(editor).includes("use vize_carton::config::matcher::LintPlanScope;"),
    "LSP must consume the host matcher",
  );
  for (const manifest of ["crates/vize/Cargo.toml", "crates/vize_maestro/Cargo.toml"]) {
    requireCondition(
      read(manifest).includes('vize_carton = { workspace = true, features = ["lint-glob"] }') &&
        !read(manifest).includes('vize_l0 = { workspace = true, features = ["lint-glob"] }'),
      `host feature must be enabled at the actual consumer: ${manifest}`,
    );
  }
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
      "usage: vp node tools/support/levels/move-config-matcher-host.ts moves|integrate|check",
    );
}
