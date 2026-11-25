import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const wrapperPath = "tools/support/launchers/rustc.sh";

// A copy of the wrapper inside a scratch workspace, a fake compiler that
// reports what it received, and a fake kache that forwards to it.
function scratchWorkspace() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-rustc-wrapper-")));
  const wrapper = path.join(root, wrapperPath);
  fs.mkdirSync(path.dirname(wrapper), { recursive: true });
  fs.copyFileSync(path.join(repoRoot, wrapperPath), wrapper);
  fs.chmodSync(wrapper, 0o755);
  const rustc = path.join(root, "fake-rustc");
  fs.writeFileSync(
    rustc,
    '#!/bin/sh\nprintf "bootstrap=%s\\n" "${RUSTC_BOOTSTRAP:-}"\nprintf "%s\\n" "$@"\n',
  );
  fs.chmodSync(rustc, 0o755);
  const kache = path.join(root, "fake-kache");
  fs.writeFileSync(kache, '#!/bin/sh\necho via-kache\nexec "$@"\n');
  fs.chmodSync(kache, 0o755);
  return { root, wrapper, rustc, kache };
}

function run(
  workspace: ReturnType<typeof scratchWorkspace>,
  args: string[],
  env: Record<string, string>,
): string[] {
  const environment: NodeJS.ProcessEnv = { PATH: process.env.PATH, ...env };
  return execFileSync(workspace.wrapper, [workspace.rustc, ...args], {
    env: environment,
    encoding: "utf8",
  })
    .trimEnd()
    .split("\n");
}

test("the rustc wrapper adds the parallel frontend to workspace crates only", (t) => {
  const workspace = scratchWorkspace();
  t.after(() => fs.rmSync(workspace.root, { recursive: true, force: true }));
  const member = path.join(workspace.root, "crates/vize_canon");
  const crate = ["--crate-name", "vize_canon", "--edition=2024", "crates/vize_canon/src/lib.rs"];
  const frontend = { VIZE_RUSTC_FRONTEND_JOBS: "8" };

  assert.deepEqual(
    run(workspace, crate, {
      ...frontend,
      CARGO_MANIFEST_DIR: member,
      CARGO_CRATE_NAME: "vize_canon",
    }),
    ["bootstrap=vize_canon", ...crate, "-Zunstable-options", "--jobs-frontend=8"],
  );

  // A registry dependency compiles exactly as Cargo asked.
  const dependency = ["--crate-name", "proc_macro2", "src/lib.rs"];
  assert.deepEqual(
    run(workspace, dependency, {
      ...frontend,
      CARGO_MANIFEST_DIR: "/home/runner/.cargo/registry/src/proc-macro2-1.0.107",
      CARGO_CRATE_NAME: "proc_macro2",
    }),
    ["bootstrap=", ...dependency],
  );
  // A sibling checkout whose path extends the workspace's is not a member.
  assert.deepEqual(
    run(workspace, crate, {
      ...frontend,
      CARGO_MANIFEST_DIR: `${workspace.root}-base/crates/vize_canon`,
      CARGO_CRATE_NAME: "vize_canon",
    }),
    ["bootstrap=", ...crate],
  );

  // A build-script probe inherits the member's environment but names another
  // crate: unstable features must stay unavailable to it.
  const probe = ["--crate-name", "probe", "--crate-type=lib", "-"];
  assert.deepEqual(
    run(workspace, probe, {
      ...frontend,
      CARGO_MANIFEST_DIR: member,
      CARGO_CRATE_NAME: "build_script_build",
    }),
    ["bootstrap=", ...probe],
  );
  // Cargo's version and target queries pass through.
  assert.deepEqual(run(workspace, ["-vV"], { ...frontend, CARGO_MANIFEST_DIR: member }), [
    "bootstrap=",
    "-vV",
  ]);

  for (const jobs of ["1", ""]) {
    assert.deepEqual(
      run(workspace, crate, {
        VIZE_RUSTC_FRONTEND_JOBS: jobs,
        CARGO_MANIFEST_DIR: member,
        CARGO_CRATE_NAME: "vize_canon",
      }),
      ["bootstrap=", ...crate],
    );
  }
});

test("the rustc wrapper hands every call to kache when the shell provides it", (t) => {
  const workspace = scratchWorkspace();
  t.after(() => fs.rmSync(workspace.root, { recursive: true, force: true }));
  const member = path.join(workspace.root, "crates/vize_canon");
  const crate = ["--crate-name", "vize_canon", "crates/vize_canon/src/lib.rs"];

  assert.deepEqual(
    run(workspace, crate, {
      VIZE_KACHE: workspace.kache,
      VIZE_RUSTC_FRONTEND_JOBS: "4",
      CARGO_MANIFEST_DIR: member,
      CARGO_CRATE_NAME: "vize_canon",
    }),
    ["via-kache", "bootstrap=vize_canon", ...crate, "-Zunstable-options", "--jobs-frontend=4"],
  );
  assert.deepEqual(run(workspace, ["-vV"], { VIZE_KACHE: workspace.kache }), [
    "via-kache",
    "bootstrap=",
    "-vV",
  ]);
  assert.deepEqual(run(workspace, ["-vV"], { VIZE_KACHE: "" }), ["bootstrap=", "-vV"]);
});

test("Rust CI jobs export the wrapper unless the job already chose one", (t) => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-frontend-env-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const script = path.join(
    repoRoot,
    ".github/actions/setup-rust-sticky-cache/scripts/enable-parallel-frontend.sh",
  );
  const enable = (env: Record<string, string>) => {
    const githubEnv = path.join(scratch, `env-${Math.random()}`);
    fs.writeFileSync(githubEnv, "");
    execFileSync("bash", ["-e", script], {
      env: { PATH: process.env.PATH, GITHUB_WORKSPACE: repoRoot, GITHUB_ENV: githubEnv, ...env },
    });
    return fs.readFileSync(githubEnv, "utf8");
  };

  assert.equal(
    enable({ VIZE_RUSTC_FRONTEND_JOBS: "8" }),
    `RUSTC_WRAPPER=${path.join(repoRoot, wrapperPath)}\nVIZE_RUSTC_FRONTEND_JOBS=8\n`,
  );
  assert.equal(enable({ VIZE_RUSTC_FRONTEND_JOBS: "8", RUSTC_WRAPPER: "sccache" }), "");
  assert.equal(enable({ VIZE_RUSTC_FRONTEND_JOBS: "8", GITHUB_WORKSPACE: scratch }), "");
  for (const jobs of ["", "0", "8 -Zother"]) {
    assert.throws(() => enable({ VIZE_RUSTC_FRONTEND_JOBS: jobs }));
  }
});

test("the dev shell and Rust CI jobs route rustc through the wrapper", () => {
  const read = (file: string) => fs.readFileSync(path.join(repoRoot, file), "utf8");
  const activate = read("tools/nix/scripts/activate-workspace.sh");
  assert.match(
    activate,
    /export RUSTC_WRAPPER="\$VIZE_WORKSPACE_ROOT\/tools\/support\/launchers\/rustc\.sh"/u,
  );
  assert.match(activate, /export VIZE_KACHE="\$\{VIZE_KACHE-@KACHE@\/bin\/kache\}"/u);
  assert.match(read("tools/nix/dev-shell.nix"), /KACHE = "\$\{config\.packages\.kache\}"/u);

  const action = read(".github/actions/setup-rust-sticky-cache/action.yml");
  const enable = read(
    ".github/actions/setup-rust-sticky-cache/scripts/enable-parallel-frontend.sh",
  );
  assert.match(action, /runner\.os != 'Windows' && inputs\.frontend-jobs != '1'/u);
  // Off by default: native history captures and installed-LSP acceptance
  // reject any rustc wrapper. Only the Rust-only jobs opt in.
  assert.match(action, /frontend-jobs:\n(?: {4}.*\n)*? {4}default: "1"/u);
  for (const workflow of ["pr-rust-checks", "pr-rust-products", "01-rust-feature-tests"]) {
    assert.match(read(`.github/workflows/${workflow}.yml`), /frontend-jobs: "8"/u, workflow);
  }
  assert.match(enable, /wrapper="\$GITHUB_WORKSPACE\/tools\/support\/launchers\/rustc\.sh"/u);
  assert.match(enable, /-z "\$\{RUSTC_WRAPPER:-\}" && -x "\$wrapper"/u);
  // The parallel frontend runs on the pinned compiler: no second toolchain.
  assert.doesNotMatch(read("tools/nix/dev-shell.nix"), /rust-bin\.nightly/u);
});

// Cargo names a test's binaries `CARGO_BIN_EXE_<bin>`, and bin names may carry
// hyphens (`vize-marquette`). Dash, the `/bin/sh` of the Linux runners, drops
// such variables before `exec`, so the wrapper must not run under `/bin/sh`.
test("the rustc wrapper keeps hyphenated Cargo variables", (t) => {
  const workspace = scratchWorkspace();
  t.after(() => fs.rmSync(workspace.root, { recursive: true, force: true }));
  assert.match(fs.readFileSync(workspace.wrapper, "utf8"), /^#!\/usr\/bin\/env bash\n/u);
  const probe = path.join(workspace.root, "env-rustc");
  fs.writeFileSync(
    probe,
    "#!/usr/bin/env node\nconsole.log(process.env['CARGO_BIN_EXE_a-b'] ?? 'missing');\n",
  );
  fs.chmodSync(probe, 0o755);
  for (const jobs of ["1", "8"]) {
    const out = execFileSync(workspace.wrapper, [probe, "--crate-name", "cli"], {
      env: {
        PATH: process.env.PATH,
        VIZE_RUSTC_FRONTEND_JOBS: jobs,
        CARGO_MANIFEST_DIR: path.join(workspace.root, "crates/vize_marquette"),
        CARGO_CRATE_NAME: "cli",
        "CARGO_BIN_EXE_a-b": "/target/a-b",
      },
      encoding: "utf8",
    });
    assert.equal(out.trimEnd(), "/target/a-b", `frontend jobs ${jobs}`);
  }
});
