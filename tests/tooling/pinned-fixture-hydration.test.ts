import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import { hydratePinnedFixture } from "../../.github/actions/hydrate-pinned-fixture/hydrate-pinned-fixture.ts";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const fixture = "tests/_fixtures/_git/element-plus";
const actualGit = execFileSync("which", ["git"], { encoding: "utf8" }).trim();
const environment = {
  ...process.env,
  GIT_ALLOW_PROTOCOL: "file",
  GIT_AUTHOR_NAME: "Fixture",
  GIT_AUTHOR_EMAIL: "fixture@example.invalid",
  GIT_COMMITTER_NAME: "Fixture",
  GIT_COMMITTER_EMAIL: "fixture@example.invalid",
};

function git(cwd: string, ...args: string[]) {
  return execFileSync(actualGit, ["-c", "core.hooksPath=/dev/null", "-C", cwd, ...args], {
    env: environment,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  }).trim();
}

function fixtureRepository() {
  const directory = realpathSync(mkdtempSync(join(tmpdir(), "vize-pinned-hydration-")));
  const upstream = join(directory, "upstream");
  mkdirSync(upstream);
  git(upstream, "init", "--template=");
  writeFileSync(join(upstream, "base"), "history before the pinned fixture\n");
  git(upstream, "add", ".");
  git(upstream, "commit", "-m", "base");
  mkdirSync(join(upstream, "src"));
  writeFileSync(join(upstream, "src/control.vue"), "<template>pinned</template>\n");
  writeFileSync(join(upstream, "run.sh"), "#!/bin/sh\nexit 0\n");
  chmodSync(join(upstream, "run.sh"), 0o755);
  symlinkSync("src/control.vue", join(upstream, "internal-link"));
  writeFileSync(join(directory, "outside"), "external symlink target must stay untouched\n");
  symlinkSync(join(directory, "outside"), join(upstream, "external-link"));
  git(upstream, "add", ".");
  git(upstream, "commit", "-m", "pin");
  const pin = git(upstream, "rev-parse", "HEAD");
  writeFileSync(
    join(upstream, "src/control.vue"),
    "<template>unrelated default branch</template>\n",
  );
  writeFileSync(join(upstream, "default-only"), "not pinned\n");
  git(upstream, "add", ".");
  git(upstream, "commit", "-m", "default");
  const defaultHead = git(upstream, "rev-parse", "HEAD");
  const source = join(directory, "source");
  mkdirSync(source);
  git(source, "init", "--template=");
  git(source, "submodule", "add", pathToFileURL(upstream).href, fixture);
  git(join(source, fixture), "checkout", "--detach", pin);
  git(source, "add", ".");
  git(source, "commit", "-m", "pin fixture");
  const clone = (name: string) => {
    const target = join(directory, name);
    git(directory, "clone", "--no-hardlinks", source, target);
    return target;
  };
  return {
    directory,
    source,
    upstream,
    pin,
    defaultHead,
    clone,
    clean: () => rmSync(directory, { recursive: true, force: true }),
  };
}

function withEnvironment<T>(values: Record<string, string>, action: () => T): T {
  const before = { ...process.env };
  Object.assign(process.env, environment, values);
  try {
    return action();
  } finally {
    for (const key of Object.keys(process.env)) if (!(key in before)) delete process.env[key];
    Object.assign(process.env, before);
  }
}

function snapshot(root: string) {
  const target = join(root, fixture);
  const database = git(target, "rev-parse", "--absolute-git-dir");
  const paths = execFileSync(actualGit, ["-C", target, "ls-files", "-z"], { encoding: "utf8" })
    .split("\0")
    .filter(Boolean);
  const files = Object.fromEntries(
    paths.map((path) => {
      const full = join(target, path);
      const stat = lstatSync(full);
      return [
        path,
        stat.isSymbolicLink()
          ? { symlink: readlinkSync(full) }
          : {
              mode: stat.mode & 0o777,
              sha256: createHash("sha256").update(readFileSync(full)).digest("hex"),
            },
      ];
    }),
  );
  return {
    head: git(target, "rev-parse", "HEAD"),
    tree: git(target, "rev-parse", "HEAD^{tree}"),
    origin: git(target, "remote", "get-url", "origin"),
    status: git(target, "status", "--porcelain=v1"),
    parentStatus: git(root, "status", "--porcelain=v1"),
    gitfile: lstatSync(join(target, ".git")).isFile(),
    database: relative(root, database),
    shallow: git(target, "rev-parse", "--is-shallow-repository"),
    history: git(target, "rev-list", "--count", "HEAD"),
    files,
  };
}

function wants(trace: string) {
  return [
    ...new Set(
      readFileSync(trace, "utf8")
        .split("\n")
        .flatMap((line) => {
          const match = /\bwant ([a-f0-9]{40})/.exec(line);
          return match ? [match[1]!] : [];
        }),
    ),
  ].sort();
}

test("exact-pin fetch omits unrelated default HEAD and preserves the whole original source protocol", () => {
  const data = fixtureRepository();
  try {
    const canonical = data.clone("canonical");
    const direct = data.clone("direct");
    const before = join(data.directory, "canonical.packets");
    const after = join(data.directory, "direct.packets");
    withEnvironment({ GIT_TRACE_PACKET: before }, () => {
      execFileSync(
        actualGit,
        ["-C", canonical, "submodule", "update", "--init", "--depth", "1", "--", fixture],
        { env: process.env },
      );
    });
    const result = withEnvironment({ GIT_TRACE_PACKET: after }, () =>
      hydratePinnedFixture(direct, fixture),
    );
    assert.equal(result.mode, "direct");
    assert.deepEqual(snapshot(direct), snapshot(canonical));
    assert.deepEqual(wants(before), [data.pin, data.defaultHead].sort());
    assert.deepEqual(wants(after), [data.pin]);
    assert.equal(snapshot(direct).history, "1");
    assert.equal(snapshot(direct).shallow, "true");
    assert.equal(existsSync(join(direct, fixture, "default-only")), false);
  } finally {
    data.clean();
  }
});

test("two private materializations and an existing clean fixture cannot mutate one another or upstream", () => {
  const data = fixtureRepository();
  try {
    const first = data.clone("first");
    const second = data.clone("second");
    withEnvironment({}, () => {
      hydratePinnedFixture(first, fixture);
      hydratePinnedFixture(second, fixture);
    });
    const peer = snapshot(second);
    const upstream = git(data.upstream, "rev-parse", "HEAD");
    const again = withEnvironment({}, () => hydratePinnedFixture(second, fixture));
    assert.equal(again.mode, "existing");
    writeFileSync(join(first, fixture, "src/control.vue"), "private edit\n");
    git(join(first, fixture), "config", "local.private", "only-first");
    assert.deepEqual(snapshot(second), peer);
    assert.equal(git(data.upstream, "rev-parse", "HEAD"), upstream);
    assert.equal(
      readFileSync(join(data.directory, "outside"), "utf8"),
      "external symlink target must stay untouched\n",
    );
    assert.equal(
      spawnSync(actualGit, ["-C", join(second, fixture), "config", "--get", "local.private"])
        .status,
      1,
    );
    const database = git(join(second, fixture), "rev-parse", "--absolute-git-dir");
    assert.equal(existsSync(join(database, "objects/info/alternates")), false);
    const objectFiles = (folder: string): string[] =>
      readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
        const path = join(folder, entry.name);
        return entry.isDirectory() ? objectFiles(path) : [path];
      });
    const privateObjects = objectFiles(join(database, "objects"));
    assert.ok(privateObjects.length > 0);
    for (const path of privateObjects) assert.equal(lstatSync(path).nlink, 1);
    assert.throws(() => withEnvironment({}, () => hydratePinnedFixture(first, fixture)), /dirty/);
    rmSync(join(second, fixture), { recursive: true });
    assert.equal(withEnvironment({}, () => hydratePinnedFixture(second, fixture)).mode, "fallback");
    assert.deepEqual(snapshot(second), peer);
  } finally {
    data.clean();
  }
});

function gitWrapper(
  directory: string,
  behavior:
    | "unavailable"
    | "wrong"
    | "corrupt"
    | "missing"
    | "error"
    | "mixed-error"
    | "stdout-error",
  defaultHead: string,
) {
  const bin = join(directory, `bin-${behavior}`);
  mkdirSync(bin);
  const program = join(bin, "git");
  writeFileSync(
    program,
    `#!${process.execPath}
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const behavior = ${JSON.stringify(behavior)};
const args = process.argv.slice(2);
const direct = args.includes('fetch') && args.some(value => value.includes('.prepare-'));
if (direct && ['unavailable', 'mixed-error', 'stdout-error'].includes(behavior)) {
  console.error('fatal: Server does not allow request for unadvertised object ' + args[args.length - 1]);
  if (behavior === 'mixed-error') console.error('fatal: pack signature mismatch');
  if (behavior === 'stdout-error') console.log('fatal: pack signature mismatch');
  process.exit(128);
}
if (direct && behavior === 'error') {
  console.error('fatal: pack signature mismatch');
  process.exit(128);
}
if (direct && behavior === 'wrong') args[args.length - 1] = ${JSON.stringify(defaultHead)};
const result = spawnSync(${JSON.stringify(actualGit)}, args, { stdio: 'inherit' });
if (direct && ['corrupt', 'missing'].includes(behavior) && result.status === 0) {
  const cwd = args[args.indexOf('-C') + 1];
  const head = args[args.length - 1];
  const loose = cwd + '/.git/objects/' + head.slice(0, 2) + '/' + head.slice(2);
  const folder = cwd + '/.git/objects/pack';
  const files = fs.existsSync(loose) ? [loose]
    : fs.readdirSync(folder).filter(name => name.endsWith('.pack')).map(name => folder + '/' + name);
  if (!files.length) throw Error('No actual Git object to damage');
  for (const file of files) {
    if (behavior === 'missing') fs.unlinkSync(file);
    else {
      fs.chmodSync(file, 0o600);
      const fd = fs.openSync(file, 'r+');
      fs.writeSync(fd, Buffer.from('broken'), 0, 6, 0);
      fs.closeSync(fd);
    }
  }
  console.error('CONTROL_DAMAGED_REAL_GIT_OBJECT');
}
process.exit(result.status ?? 1);
`,
  );
  chmodSync(program, 0o755);
  return `${bin}:${process.env.PATH ?? ""}`;
}

test("unavailable exact fetch retains the original canonical fallback and whole pinned source", () => {
  const data = fixtureRepository();
  try {
    const canonical = data.clone("canonical");
    const fallback = data.clone("fallback");
    git(canonical, "submodule", "update", "--init", "--depth", "1", "--", fixture);
    const result = withEnvironment(
      { PATH: gitWrapper(data.directory, "unavailable", data.defaultHead) },
      () => hydratePinnedFixture(fallback, fixture),
    );
    assert.equal(result.mode, "fallback");
    assert.deepEqual(snapshot(fallback), snapshot(canonical));
    assert.equal(
      readdirSync(dirname(join(fallback, fixture))).some((name) => name.includes(".prepare-")),
      false,
    );
  } finally {
    data.clean();
  }
});

for (const behavior of [
  "wrong",
  "corrupt",
  "missing",
  "error",
  "mixed-error",
  "stdout-error",
] as const) {
  test(`${behavior} exact-fetch output refuses without fallback or clean-source success`, () => {
    const data = fixtureRepository();
    try {
      const target = data.clone("target");
      const outcome = spawnSync(
        process.execPath,
        [
          join(repository, ".github/actions/hydrate-pinned-fixture/hydrate-pinned-fixture.ts"),
          target,
          fixture,
        ],
        {
          encoding: "utf8",
          env: { ...environment, PATH: gitWrapper(data.directory, behavior, data.defaultHead) },
        },
      );
      if (behavior === "corrupt" || behavior === "missing")
        assert.match(outcome.stderr, /CONTROL_DAMAGED_REAL_GIT_OBJECT/);
      if (behavior === "stdout-error")
        assert.match(outcome.stdout, /fatal: pack signature mismatch/);
      assert.equal(outcome.status, 1);
      assert.doesNotMatch(outcome.stdout, /"schema":"vize-direct-pinned-fixture-v1"/);
      assert.match(
        outcome.stderr,
        behavior === "wrong"
          ? /different fixture commit/
          : behavior === "error" || behavior === "mixed-error" || behavior === "stdout-error"
            ? /cannot waive this failure/
            : /Command failed/,
      );
      assert.equal(readdirSync(join(target, fixture)).length, 0);
      assert.equal(
        readdirSync(dirname(join(target, fixture))).some((name) => name.includes(".prepare-")),
        false,
      );
      assert.equal(git(target, "status", "--porcelain=v1"), "");
    } finally {
      data.clean();
    }
  });
}

test("changed authored pin and URL are fetched exactly; dirty declarations and redirected paths refuse", () => {
  const data = fixtureRepository();
  try {
    const newUrl = pathToFileURL(data.upstream).href + "/";
    git(data.source, "config", "--file", ".gitmodules", `submodule.${fixture}.url`, newUrl);
    git(join(data.source, fixture), "checkout", "--detach", data.defaultHead);
    git(data.source, "add", ".");
    git(data.source, "commit", "-m", "new pin and URL");
    const target = data.clone("changed");
    const result = withEnvironment({}, () => hydratePinnedFixture(target, fixture));
    assert.equal(result.head, data.defaultHead);
    assert.equal(snapshot(target).origin, newUrl);
    writeFileSync(join(target, ".gitmodules"), "changed declaration\n");
    assert.throws(
      () => withEnvironment({}, () => hydratePinnedFixture(target, fixture)),
      /declarations differ/,
    );
    const redirected = data.clone("redirected");
    const outside = join(data.directory, "outside-dir");
    mkdirSync(outside);
    rmSync(join(redirected, fixture), { recursive: true });
    symlinkSync(outside, join(redirected, fixture));
    assert.throws(
      () => withEnvironment({}, () => hydratePinnedFixture(redirected, fixture)),
      /symlink/,
    );
    assert.deepEqual(readdirSync(outside), []);
    assert.throws(() => hydratePinnedFixture(redirected, "../outside"), /single source-tree/);
  } finally {
    data.clean();
  }
});

test("existing Git proxy rewrites preserve the canonical origin and private-object authority", () => {
  const data = fixtureRepository();
  try {
    const url = "https://fixture.invalid/element-plus";
    git(data.source, "config", "--file", ".gitmodules", `submodule.${fixture}.url`, url);
    git(data.source, "add", ".gitmodules");
    git(data.source, "commit", "-m", "transport rewrite");
    const target = data.clone("proxy");
    const result = withEnvironment(
      {
        GIT_CONFIG_COUNT: "1",
        GIT_CONFIG_KEY_0: `url.${pathToFileURL(data.upstream).href}.insteadOf`,
        GIT_CONFIG_VALUE_0: url,
      },
      () => hydratePinnedFixture(target, fixture),
    );
    assert.equal(result.head, data.pin);
    assert.equal(git(join(target, fixture), "config", "--get", "remote.origin.url"), url);
    assert.throws(
      () =>
        withEnvironment({ GIT_ALTERNATE_OBJECT_DIRECTORIES: data.upstream }, () =>
          hydratePinnedFixture(target, fixture),
        ),
      /private Git context/,
    );
  } finally {
    data.clean();
  }
});

test("plain local relative origins retain canonical history and parent-repository resolution", () => {
  const data = fixtureRepository();
  try {
    git(data.source, "config", "--file", ".gitmodules", `submodule.${fixture}.url`, "../upstream");
    git(data.source, "add", ".gitmodules");
    git(data.source, "commit", "-m", "relative origin");
    const canonical = data.clone("relative-before");
    const direct = data.clone("relative-after");
    git(canonical, "submodule", "update", "--init", "--depth", "1", "--", fixture);
    const result = withEnvironment({}, () => hydratePinnedFixture(direct, fixture));
    assert.equal(result.mode, "fallback");
    assert.equal(snapshot(canonical).shallow, "false");
    assert.equal(snapshot(canonical).history, "2");
    assert.deepEqual(snapshot(direct), snapshot(canonical));
  } finally {
    data.clean();
  }
});

test("file-protocol relative origins keep exact-fetch depth and original whole-source parity", () => {
  const data = fixtureRepository();
  try {
    git(data.source, "config", "--file", ".gitmodules", `submodule.${fixture}.url`, "../upstream");
    git(data.source, "add", ".gitmodules");
    git(data.source, "commit", "-m", "relative origin over file protocol");
    const canonical = data.clone("file-before");
    const direct = data.clone("file-after");
    for (const root of [canonical, direct])
      git(root, "remote", "set-url", "origin", pathToFileURL(data.source).href);
    git(canonical, "submodule", "update", "--init", "--depth", "1", "--", fixture);
    const result = withEnvironment({}, () => hydratePinnedFixture(direct, fixture));
    assert.equal(result.mode, "direct");
    assert.equal(snapshot(canonical).shallow, "true");
    assert.equal(snapshot(canonical).history, "1");
    assert.deepEqual(snapshot(direct), snapshot(canonical));
  } finally {
    data.clean();
  }
});
