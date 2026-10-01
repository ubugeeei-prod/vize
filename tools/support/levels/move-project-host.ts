import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const source = "davinci/vize_l0/src/config/project.rs";
const target = "crates/vize_carton/src/config/project.rs";
const core = "davinci/vize_l0/src/config.rs";
const host = "crates/vize_carton/src/config.rs";
const callers = [
  "crates/vize_maestro/src/server/state/batch_cache.rs",
  "crates/vize_maestro/src/server/state/corsa.rs",
];
const read = (path: string) => readFileSync(join(root, path), "utf8");
const exists = (path: string) => existsSync(join(root, path));
const write = (path: string, content: string) => writeFileSync(join(root, path), content);
const requireCondition = (condition: boolean, message: string) => {
  if (!condition) throw new Error(message);
};

requireCondition(
  exists(host) &&
    read(host).includes("mod loader;") &&
    read(host).includes("pub use vize_l0::config::*;"),
  "Carton must already own the configuration loader and consume effective L0 types",
);

function checkMove() {
  requireCondition(
    !exists(source) && exists(target),
    "unmoved project source or missing host target",
  );
}

function move() {
  requireCondition(
    exists(source) !== exists(target),
    "project source/target collision or missing input",
  );
  if (exists(source)) {
    mkdirSync(dirname(join(root, target)), { recursive: true });
    renameSync(join(root, source), join(root, target));
  }
  checkMove();
}

function integrate() {
  checkMove();
  const coreValue = read(core);
  const hostValue = read(host);
  requireCondition(
    coreValue.includes("mod project;") || hostValue.includes("mod project;"),
    "unexpected project declaration state",
  );
  requireCondition(
    coreValue.includes("pub use project::ProjectModel;") ||
      hostValue.includes("pub use project::ProjectModel;"),
    "unexpected project export state",
  );
  for (const file of callers) {
    requireCondition(
      read(file).includes("vize_l0::config::ProjectModel::new") ||
        read(file).includes("vize_carton::config::ProjectModel::new"),
      `unexpected project caller: ${file}`,
    );
  }
  requireCondition(
    read("tests/tooling/support/davinci-host-imports.ts").includes("host_project_selection"),
    "the reviewed project-host gate companion must accompany replay",
  );
  write(
    core,
    coreValue.replace("mod project;\n", "").replace("pub use project::ProjectModel;\n", ""),
  );
  write(
    host,
    hostValue
      .replace(
        /^mod loader;$/mu,
        hostValue.includes("mod project;\n") ? "mod loader;" : "mod loader;\nmod project;",
      )
      .replace(
        "pub use vize_l0::config::*;",
        hostValue.includes("pub use project::ProjectModel;\n")
          ? "pub use vize_l0::config::*;"
          : "pub use project::ProjectModel;\npub use vize_l0::config::*;",
      ),
  );
  for (const file of callers) {
    write(
      file,
      read(file).replaceAll(
        "vize_l0::config::ProjectModel::new",
        "vize_carton::config::ProjectModel::new",
      ),
    );
  }
  check();
}

function check() {
  checkMove();
  requireCondition(
    !read(core).includes("mod project;") && !read(core).includes("pub use project::ProjectModel;"),
    "L0 must not retain the host project model or forward to Carton",
  );
  requireCondition(
    read(host).includes("mod project;") && read(host).includes("pub use project::ProjectModel;"),
    "Carton must own the project model",
  );
  for (const file of callers) {
    requireCondition(
      !read(file).includes("vize_l0::config::ProjectModel") &&
        read(file).includes("vize_carton::config::ProjectModel::new"),
      `project host caller must use Carton: ${file}`,
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
      "usage: vp node tools/support/levels/move-project-host.ts moves|integrate|check",
    );
}
