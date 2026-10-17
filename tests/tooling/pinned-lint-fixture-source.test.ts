import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { preparePinnedFixtureWorkspace } from "../_helpers/realworld-patch.ts";

import {
  preparePinnedFixtureDependencies,
  readPinnedFixtureDependencyPlan,
} from "../_helpers/pinned-fixture-dependencies.ts";
import {
  assertPinnedFixtureFiles,
  exportPinnedFixtureGitlinks,
  readPinnedFixtureSource,
} from "../_helpers/pinned-fixture-source.ts";

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
}

function withFixture(run: (root: string, pin: string) => void): void {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pinned-lint-"));
  try {
    git(root, ["init", "-q"]);
    fs.writeFileSync(path.join(root, "Original.vue"), "<template><div /></template>\n");
    fs.writeFileSync(path.join(root, "vite.config.ts"), "export default {}\n");
    fs.writeFileSync(path.join(root, "package.json"), '{"packageManager":"pnpm@10.28.2"}\n');
    fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    git(root, ["add", "."]);
    git(root, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "pin",
    ]);
    run(root, git(root, ["rev-parse", "HEAD"]));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("an unhydrated fixture cannot authenticate its parent's Git HEAD", () => {
  withFixture((root, pin) => {
    const empty = path.join(root, "empty-submodule");
    fs.mkdirSync(empty);
    assert.throws(() => readPinnedFixtureSource(empty, pin), /initialized repository/);
    assert.throws(() => readPinnedFixtureSource(root, "0".repeat(40)), /owning Gitlink/);
  });
});

test("dependency setup may add installed files while all original blobs stay exact", () => {
  withFixture((root, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    fs.mkdirSync(path.join(root, "node_modules"));
    fs.writeFileSync(path.join(root, "node_modules", "installed.json"), '{"version":"1.0.0"}');
    assert.equal(source.blobs.length, 4);
    assertPinnedFixtureFiles(root, source);
  });
});

test("nested Gitlinks must be hydrated at their own pin and their complete blobs are exported", () => {
  withFixture((root) => {
    const nested = path.join(root, "assets");
    fs.mkdirSync(nested);
    git(nested, ["init", "-q"]);
    fs.writeFileSync(path.join(nested, "original.txt"), "nested original\n");
    git(nested, ["add", "."]);
    git(nested, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "nested",
    ]);
    const nestedPin = git(nested, ["rev-parse", "HEAD"]);
    git(root, ["update-index", "--add", "--cacheinfo", "160000", nestedPin, "assets"]);
    git(root, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "gitlink",
    ]);
    const pin = git(root, ["rev-parse", "HEAD"]);
    const source = readPinnedFixtureSource(root, pin);
    assert.deepEqual(source.gitlinks, [{ path: "assets", revision: nestedPin }]);
    assertPinnedFixtureFiles(root, source);
    const exported = path.join(root, "exported");
    fs.mkdirSync(exported);
    for (const file of ["Original.vue", "vite.config.ts", "package.json", "pnpm-lock.yaml"]) {
      fs.copyFileSync(path.join(root, file), path.join(exported, file));
    }
    exportPinnedFixtureGitlinks(exported, root, source);
    assertPinnedFixtureFiles(exported, source);
    fs.appendFileSync(path.join(exported, "assets", "original.txt"), "changed");
    assert.throws(
      () => assertPinnedFixtureFiles(exported, source),
      /original fixture bytes changed/,
    );
    fs.rmSync(nested, { recursive: true });
    fs.mkdirSync(nested);
    assert.throws(() => readPinnedFixtureSource(root, pin), /initialized repository/);
  });
});

for (const file of ["Original.vue", "vite.config.ts", "package.json", "pnpm-lock.yaml"]) {
  test(`fixture verification rejects rewritten ${file}`, () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      fs.appendFileSync(path.join(root, file), "changed\n");
      assert.throws(() => assertPinnedFixtureFiles(root, source), /original fixture bytes changed/);
    });
  });
}

test(
  "a regular source replaced with a symlink does not inherit the target's identity",
  {
    skip: process.platform === "win32",
  },
  () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      const original = path.join(root, "Original.vue");
      const replacement = path.join(root, "replacement.vue");
      fs.copyFileSync(original, replacement);
      fs.unlinkSync(original);
      fs.symlinkSync(replacement, original);
      assert.throws(() => assertPinnedFixtureFiles(root, source), /changed type/);
    });
  },
);

test(
  "executable permissions are part of the authenticated fixture on Unix",
  {
    skip: process.platform === "win32",
  },
  () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      fs.chmodSync(path.join(root, "vite.config.ts"), 0o755);
      assert.throws(() => assertPinnedFixtureFiles(root, source), /executable mode changed/);
    });
  },
);

function withDependencyArchive(run: (root: string, archive: string, pin: string) => void): void {
  withFixture((root) => {
    fs.writeFileSync(
      path.join(root, "package.json"),
      '{"name":"create-vue","packageManager":"pnpm@11.10.0"}\n',
    );
    fs.writeFileSync(path.join(root, ".npmrc"), "engine-strict=true\n");
    git(root, ["add", "package.json", ".npmrc"]);
    git(root, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "closure",
    ]);
    const pin = git(root, ["rev-parse", "HEAD"]);
    const archive = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dependency-archive-"));
    try {
      for (const { path: file } of readPinnedFixtureSource(root, pin).blobs) {
        fs.copyFileSync(path.join(root, file), path.join(archive, file));
      }
      run(root, archive, pin);
    } finally {
      fs.rmSync(archive, { recursive: true, force: true });
    }
  });
}

test("dependency planning retains every source blob separately from the declared corpus and original install inputs", () => {
  withDependencyArchive((root, archive, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    const plan = readPinnedFixtureDependencyPlan(archive, root, source, ["Original.vue"]);
    assert.deepEqual(plan.source, source);
    assert.equal(plan.source.blobs.length, 5);
    assert.deepEqual(
      plan.declaredCorpus.blobs,
      source.blobs.filter(({ path }) => path === "Original.vue"),
    );
    assert.equal(plan.managerVersion, "11.10.0");
    assert.equal(plan.packageManager, "pnpm@11.10.0");
    assert.deepEqual(plan.stubs, []);
    assert.deepEqual(plan.modules, [
      { importer: "package.json", specifier: "@tsconfig/node24/tsconfig.json" },
    ]);
    assert.deepEqual(
      plan.originalInputs.map(({ path: file, contentsBase64 }) => ({
        file: path.basename(file),
        contents: Buffer.from(contentsBase64, "base64").toString("utf8"),
      })),
      [
        { file: ".npmrc", contents: "engine-strict=true\n" },
        {
          file: "package.json",
          contents: '{"name":"create-vue","packageManager":"pnpm@11.10.0"}\n',
        },
        { file: "pnpm-lock.yaml", contents: "lockfileVersion: '9.0'\n" },
      ],
    );
    assert.equal(fs.existsSync(path.join(archive, "node_modules")), false);
  });
});

test("dependency preparation refuses upstream writes, untrusted corpus and foreign dependencies before launching installers", () => {
  withDependencyArchive((root, archive, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    assert.throws(
      () => preparePinnedFixtureDependencies(root, root, source, ["Original.vue"]),
      /never modify the upstream checkout/,
    );
    assert.throws(
      () => preparePinnedFixtureDependencies(archive, root, source, ["untracked.vue"]),
      /must belong to the original source/,
    );
    assert.throws(
      () =>
        preparePinnedFixtureDependencies(archive, root, source, ["Original.vue", "Original.vue"]),
      /must be unique/,
    );
    fs.mkdirSync(path.join(archive, "node_modules"));
    assert.throws(
      () => preparePinnedFixtureDependencies(archive, root, source, ["Original.vue"]),
      /without foreign node_modules/,
    );
    fs.rmdirSync(path.join(archive, "node_modules"));
    for (const file of [".npmrc", "pnpm-lock.yaml", "vite.config.ts"]) {
      const original = fs.readFileSync(path.join(archive, file));
      fs.appendFileSync(path.join(archive, file), "changed\n");
      assert.throws(
        () => preparePinnedFixtureDependencies(archive, root, source, ["Original.vue"]),
        /original fixture bytes changed/,
      );
      fs.writeFileSync(path.join(archive, file), original);
    }
    assert.equal(
      fs.existsSync(path.join(archive, ".vize-pinned-fixture-dependencies.json")),
      false,
    );
    assertPinnedFixtureFiles(root, source);
  });
});

test("an explicit repository root keeps fixture authority and output inside that checkout", () => {
  withFixture((source, pin) => {
    const repository = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pinned-root-"));
    let fixture: ReturnType<typeof preparePinnedFixtureWorkspace> | undefined;
    try {
      const fixturePath = "tests/_fixtures/_git/isolated";
      const upstream = path.join(repository, fixturePath);
      fs.cpSync(source, upstream, { recursive: true });
      fs.writeFileSync(
        path.join(repository, "tests/_fixtures/vue-ecosystem-fixtures.json"),
        JSON.stringify({
          projects: [{ id: "isolated", displayName: "isolated", fixturePath, revision: pin }],
        }),
      );
      fixture = preparePinnedFixtureWorkspace({
        repositoryRoot: repository,
        fixtureId: "isolated",
        includePaths: ["Original.vue"],
      });
      assert.equal(fixture.upstreamDir, upstream);
      assert.equal(fixture.entry.revision, pin);
      assert.ok(
        fixture.workspaceDir.startsWith(
          path.join(repository, "target/vize-tests/realworld-patches"),
        ),
      );
      assert.equal(
        fixture.read("Original.vue"),
        fs.readFileSync(path.join(source, "Original.vue"), "utf8"),
      );
      assert.equal(git(upstream, ["status", "--porcelain"]), "");
    } finally {
      fixture?.dispose();
      fs.rmSync(repository, { recursive: true, force: true });
    }
  });
});
