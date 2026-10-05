/** Real checkout controls for the sole manual source dependency-link exception. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  lstatSync,
  lutimesSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, relative } from "node:path";
import test from "node:test";
import { captureSourceCustody } from "./typechecker-native-source-custody.mjs";

const WORKFLOW = ".github/workflows/typechecker-native-phases.yml";
const BRIDGES = [
  "Cargo.toml",
  "Cargo.lock",
  "package.json",
  "pnpm-lock.yaml",
  ...["corpus", "leaf-corpus", "protocol"].map(
    (name) => `tools/benchmarks/scripts/type-snapshot-cli-${name}.mjs`,
  ),
];
function git(root, ...args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8", timeout: 30_000 });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
const untracked = (root) => git(root, "ls-files", "--others", "--exclude-standard", "-z");
function put(root, path, text = "fixture bytes\n") {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), text);
}
function fixture(run) {
  const temporary = realpathSync(mkdtempSync(join(os.tmpdir(), "native-dependency-link-")));
  const root = join(temporary, "driver");
  const sourceRoot = join(temporary, "source");
  mkdirSync(root);
  try {
    git(root, "init", "--initial-branch=main");
    git(root, "config", "user.name", "Dependency link fixture");
    git(root, "config", "user.email", "dependency-link@example.invalid");
    git(root, "config", "commit.gpgsign", "false");
    for (const path of [
      ...BRIDGES,
      WORKFLOW,
      "crates/vize_canon/examples/native_phase_projection.rs",
    ])
      put(root, path);
    put(root, ".gitignore", "node_modules/\ntarget/\n");
    git(root, "add", "-A");
    git(root, "commit", "-m", "production fixture");
    const sha = git(root, "rev-parse", "HEAD").trim();
    git(root, "update-ref", "refs/remotes/origin/main", sha);
    git(root, "worktree", "add", "--detach", sourceRoot, sha);
    const options = {
      driverRoot: root,
      sourceRoot,
      env: {
        GITHUB_EVENT_NAME: "workflow_dispatch",
        GITHUB_REPOSITORY: "ubugeeei-prod/vize",
        GITHUB_REF: "refs/heads/main",
        GITHUB_WORKFLOW_SHA: sha,
        GITHUB_RUN_ID: "12345",
        GITHUB_RUN_ATTEMPT: "1",
        DRIVER_SHA: sha,
        SOURCE_SHA: sha,
        MAIN_HEAD_SHA: sha,
        MAIN_SOURCE_SHA: sha,
      },
    };
    run({
      root,
      sourceRoot,
      temporary,
      options,
      modules: join(root, "node_modules"),
      link: join(sourceRoot, "node_modules"),
    });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}
function install(f) {
  put(f.modules, "locked-fixture.txt");
  symlinkSync(f.modules, f.link);
}
function hide(f) {
  writeFileSync(join(f.root, ".git/info/exclude"), "node_modules\n");
}

await test("actual directory ignore exposes the link; valid installation changes only its custody", () => {
  fixture((f) => {
    const before = captureSourceCustody(f.options);
    assert.equal(before.sourceDependencyLink, null);
    install(f);
    assert.equal(untracked(f.sourceRoot), "node_modules\0");
    const after = captureSourceCustody(f.options);
    assert.deepEqual(captureSourceCustody(f.options), after);
    const { sourceDependencyLink: _beforeLink, ...beforeAuthority } = before;
    const { sourceDependencyLink: link, ...afterAuthority } = after;
    assert.deepEqual(beforeAuthority, afterAuthority);
    assert.equal(link.path, f.link);
    assert.equal(link.target, f.modules);
    assert.equal(link.canonicalTarget, realpathSync(f.link));
    const stat = lstatSync(f.link, { bigint: true });
    for (const key of "dev ino mode size mtimeNs ctimeNs".split(" "))
      assert.equal(link.revision[key], stat[key].toString());
    assert.deepEqual(Object.keys(link.driverDirectory), ["path", "dev", "ino", "mode"]);
    assert.equal(link.driverDirectory.ino, lstatSync(f.modules, { bigint: true }).ino.toString());
    assert.equal(lstatSync(f.modules).isDirectory(), true);
    assert.equal(lstatSync(f.modules).isSymbolicLink(), false);
    assert.equal(
      readFileSync(join(f.sourceRoot, ".gitignore"), "utf8"),
      "node_modules/\ntarget/\n",
    );
    assert.equal(untracked(f.sourceRoot), "node_modules\0");
  });
});

await test("ignored valid link still receives a nonnull custody record", () => {
  fixture((f) => {
    install(f);
    hide(f);
    assert.equal(untracked(f.sourceRoot), "");
    assert.equal(captureSourceCustody(f.options).sourceDependencyLink.target, f.modules);
  });
});

for (const mode of ["relative", "offsite", "dangling", "canonical-alias"]) {
  await test(`reject source dependency target ${mode}`, () =>
    fixture((f) => {
      install(f);
      rmSync(f.link);
      let target = f.modules;
      if (mode === "relative") target = relative(f.sourceRoot, f.modules);
      if (mode === "offsite") {
        target = join(f.temporary, "offsite");
        mkdirSync(target);
      }
      if (mode === "dangling") rmSync(f.modules, { recursive: true });
      if (mode === "canonical-alias")
        target = f.modules.startsWith("/private/var/")
          ? f.modules.replace("/private/var/", "/var/")
          : `${f.root}/./node_modules`;
      symlinkSync(target, f.link);
      if (mode === "canonical-alias") assert.equal(realpathSync(f.link), f.modules);
      assert.throws(() => captureSourceCustody(f.options));
    }));
}

for (const paired of [true, false])
  for (const mode of ["driver-symlink", "cycle", "driver-file"]) {
    await test(`reject driver ${mode}, paired=${paired}`, () =>
      fixture((f) => {
        install(f);
        if (!paired) rmSync(f.link);
        rmSync(f.modules, { recursive: true });
        if (mode === "driver-file") writeFileSync(f.modules, "not a directory");
        else {
          const target =
            mode === "cycle" ? (paired ? f.link : f.modules) : join(f.temporary, "offsite");
          if (mode !== "cycle") mkdirSync(target);
          symlinkSync(target, f.modules);
        }
        hide(f);
        assert.throws(() => captureSourceCustody(f.options), /physical directory/u);
      }));
  }

await test("PR cannot borrow the manual exception for its driver-side dependency link", () => {
  fixture((f) => {
    mkdirSync(join(f.temporary, "offsite"));
    symlinkSync(join(f.temporary, "offsite"), f.modules);
    const options = {
      ...f.options,
      sourceRoot: f.root,
      env: {
        ...f.options.env,
        GITHUB_EVENT_NAME: "pull_request",
        PR_BASE_SHA: f.options.env.SOURCE_SHA,
      },
    };
    assert.throws(() => captureSourceCustody(options), /untracked source inputs/u);
  });
});

for (const path of [
  "crates/vize/build.rs",
  "node_modules\n",
  "node_modules ",
  "node_modules-other",
]) {
  await test(`other untracked input remains forbidden: ${JSON.stringify(path)}`, () =>
    fixture((f) => {
      install(f);
      put(f.sourceRoot, path);
      assert(untracked(f.sourceRoot).includes(path + "\0"));
      assert.throws(() => captureSourceCustody(f.options), /untracked source inputs/u);
    }));
}

for (const [field, value] of [
  ["GITHUB_EVENT_NAME", "push"],
  ["GITHUB_REF", "refs/heads/experiment"],
  ["GITHUB_WORKFLOW_SHA", "main"],
  ["SOURCE_SHA", "main"],
]) {
  await test(`valid link does not authorize invalid context ${field}`, () =>
    fixture((f) => {
      install(f);
      assert.throws(() =>
        captureSourceCustody({ ...f.options, env: { ...f.options.env, [field]: value } }),
      );
    }));
}

for (const path of BRIDGES) {
  await test(`valid link does not authorize bridge drift: ${path}`, () =>
    fixture((f) => {
      install(f);
      put(f.root, path, "different committed driver bytes\n");
      git(f.root, "add", "--", path);
      git(f.root, "commit", "-m", "driver bridge drift");
      const driver = git(f.root, "rev-parse", "HEAD").trim();
      git(f.root, "update-ref", "refs/remotes/origin/main", driver);
      f.options.env.DRIVER_SHA = driver;
      f.options.env.GITHUB_WORKFLOW_SHA = driver;
      f.options.env.MAIN_HEAD_SHA = driver;
      assert.throws(
        () => captureSourceCustody(f.options),
        /production inputs|dependency or corpus drift/u,
      );
    }));
}

for (const ignored of [false, true]) {
  await test(`retargeting is rejected even when ignore-hidden=${ignored}`, () =>
    fixture((f) => {
      install(f);
      const before = captureSourceCustody(f.options);
      if (ignored) hide(f);
      mkdirSync(join(f.temporary, "offsite"));
      rmSync(f.link);
      symlinkSync(join(f.temporary, "offsite"), f.link);
      assert.equal(before.sourceDependencyLink.target, f.modules);
      assert.throws(() => captureSourceCustody(f.options), /target mismatch/u);
    }));
}

for (const mode of ["recreated-link", "same-inode-timestamp", "replaced-driver-directory"]) {
  await test(`full repeated custody detects ${mode} with the same canonical target`, () =>
    fixture((f) => {
      install(f);
      const before = captureSourceCustody(f.options);
      if (mode === "recreated-link") {
        rmSync(f.link);
        symlinkSync(f.modules, f.link);
      }
      if (mode === "same-inode-timestamp") lutimesSync(f.link, new Date(10_000), new Date(20_000));
      if (mode === "replaced-driver-directory") {
        renameSync(f.modules, join(f.temporary, "old-dependencies"));
        put(f.modules, "locked-fixture.txt");
      }
      const after = captureSourceCustody(f.options);
      const beforeLink = before.sourceDependencyLink;
      const afterLink = after.sourceDependencyLink;
      assert.equal(readlinkSync(f.link), f.modules);
      assert.equal(afterLink.canonicalTarget, beforeLink.canonicalTarget);
      assert.notDeepEqual(after, before, "final campaign receipt equality must detect replacement");
      if (mode === "same-inode-timestamp") {
        assert.equal(afterLink.revision.ino, beforeLink.revision.ino);
        assert.notEqual(afterLink.revision.mtimeNs, beforeLink.revision.mtimeNs);
      }
      if (mode === "replaced-driver-directory") {
        assert.deepEqual(afterLink.revision, beforeLink.revision);
        assert.notEqual(afterLink.driverDirectory.ino, beforeLink.driverDirectory.ino);
        assert.equal(
          readFileSync(join(f.modules, "locked-fixture.txt"), "utf8"),
          "fixture bytes\n",
        );
      }
    }));
}

await test("ordinary source-root and dependency-directory mutations do not create false custody changes", () => {
  fixture((f) => {
    install(f);
    const before = captureSourceCustody(f.options);
    mkdirSync(join(f.sourceRoot, "target"));
    put(f.modules, ".cache/ignored-runtime-output");
    assert.deepEqual(captureSourceCustody(f.options), before);
  });
});

for (const mode of ["link-revision", "target-directory"]) {
  await test(`stable observation fences an actual in-call ${mode} change`, () =>
    fixture((f) => {
      install(f);
      // Isolated child patches only observation timing; actual files/Git/guard are real.
      const script = `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
const target = ${JSON.stringify(f.modules)}, link = ${JSON.stringify(f.link)};
if (${JSON.stringify(mode)} === 'link-revision') {
  const read = fs.readlinkSync; let changed = false;
  fs.readlinkSync = (...args) => {
    const raw = read(...args);
    if (args[0] === link && !changed) { changed = true; fs.lutimesSync(link, new Date(10000), new Date(20000)); }
    return raw;
  };
} else {
  const open = fs.openSync; let changed = false;
  fs.openSync = (...args) => {
    const fd = open(...args);
    if (args[0] === target && !changed) {
      changed = true; fs.renameSync(target, ${JSON.stringify(join(f.temporary, "old-dependencies"))}); fs.mkdirSync(target);
    }
    return fd;
  };
}
syncBuiltinESMExports();
const { captureSourceCustody } = await import(${JSON.stringify(new URL("./typechecker-native-source-custody.mjs", import.meta.url).href)});
assert.throws(() => captureSourceCustody(${JSON.stringify(f.options)}), /changed during custody/);
`;
      const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
        encoding: "utf8",
        timeout: 30_000,
      });
      assert.equal(result.error, undefined, result.error?.message);
      assert.equal(result.status, 0, result.stderr);
    }));
}
