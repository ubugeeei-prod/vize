import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { releaseCliFixture, type DriverEvent } from "./support/release-cli-fixture.ts";

function assertDriver(events: DriverEvent[], args: string[], cargo: string) {
  assert.deepEqual(events, [{ kind: "driver", args, cargo }]);
}

test(
  "release declines through the controlling terminal prompt before driver dispatch",
  { skip: process.platform === "win32" },
  () => {
    const fixture = releaseCliFixture();
    try {
      const result = fixture.prompt("n\n", ["minor"]);
      const output = result.stdout + result.stderr;
      assert.equal(result.error, undefined, output);
      assert.equal(result.status, 1, output);
      assert.match(output, /Proceed with release\? \[y\/N\]/);
      assert.match(output, /Aborted\./);
      assert.deepEqual(fixture.events(), []);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  },
);

test(
  "release dispatches the driver once after affirmative confirmation",
  { skip: process.platform === "win32" },
  () => {
    const fixture = releaseCliFixture();
    try {
      const result = fixture.prompt("y\n", ["minor"]);
      const output = result.stdout + result.stderr;
      assert.equal(result.error, undefined, output);
      assert.equal(result.status, 83, output);
      assert.ok(output.indexOf("New version: 0.441.0") < output.indexOf("Proceed with release?"));
      assertDriver(
        fixture.events(),
        ["tools/commands/release/pr.rs", "start", "minor"],
        fixture.cargo,
      );
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  },
);

for (const [extra, banner] of [
  [[], "New version: 0.441.0 (tag: v0.441.0)"],
  [
    ["--pin"],
    "The official pinned driver resolves the next unreserved version from immutable retirement evidence.",
  ],
] as const) {
  test(`release -y dispatches the existing driver ${extra.join(" ")}`, () => {
    const fixture = releaseCliFixture();
    try {
      const result = fixture.run(["minor", "-y", ...extra]);
      assert.equal(result.status, 83, result.stderr + result.stdout);
      assert.equal(result.stdout, `Current version: 0.440.0\n${banner}\n\n`);
      assert.equal(result.stderr, "");
      assertDriver(
        fixture.events(),
        ["tools/commands/release/pr.rs", "start", "minor", ...extra],
        fixture.cargo,
      );
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

test("an authenticated pinned target reaches the local guard before any mutation", () => {
  const fixture = releaseCliFixture(true);
  try {
    const result = fixture.run(["minor", "-y", "--prepare-only", "--pinned-target", "0.441.5"]);
    assert.equal(result.status, 1, result.stderr + result.stdout);
    assert.equal(
      result.stdout,
      "Current version: 0.440.0\nNew version: 0.441.5 (tag: v0.441.5)\n\n",
    );
    assert.equal(
      result.stderr,
      "Release preflight script not found: tools/commands/ci/github/release-local-guard.rs\n",
    );
    assert.deepEqual(fixture.events(), [
      {
        kind: "authenticate",
        args: [
          "tools/commands/release/pr.rs",
          "validate-preparation-target",
          "minor",
          "0.440.0",
          "0.441.5",
        ],
        cargo: fixture.cargo,
      },
    ]);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});

for (const args of [
  ["--print-bump", "0.440.0", "minor"],
  ["--print-readme-version-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-workspace-manifest-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-workspace-catalog-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-lockfile-catalog-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-extra-package-json-paths"],
]) {
  test(`pure mode ${args[0]} never dispatches the driver or mutates files`, () => {
    const fixture = releaseCliFixture();
    try {
      const result = fixture.run(args);
      assert.equal(result.status, 0, result.stderr + result.stdout);
      assert.equal(result.stderr, "");
      assert.deepEqual(fixture.events(), []);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

const usage =
  "Usage: node tools/support/release/release/main.ts <bump> [-y] [--pin] | --resume <PR> [--pin] | --retire <PR> --head <H> --tag <tag> --run <R> --operator-run <entry>\n";
for (const [args, message] of [
  [[], usage],
  [["minor", "--unknown"], "Unknown release arguments\n"],
  [
    ["minor", "--force-tag"],
    "Error: --force-tag is not supported; published release tags are immutable.\n",
  ],
  [["minor", "--prepare-only", "--pin"], "Error: --pin requires the release PR protocol.\n"],
  [["minor", "--pinned-target"], "Invalid internal pinned preparation target\n"],
  [
    ["minor", "--pinned-target", "0.441.0"],
    "Pinned target is internal to validated minor preparation\n",
  ],
  [
    ["minor", "--prepare-only", "--pinned-target", "0.441.0", "--pinned-target", "0.441.0"],
    "Invalid internal pinned preparation target\n",
  ],
  [
    ["patch", "--prepare-only", "--pinned-target", "0.441.0"],
    "Pinned target is internal to validated minor preparation\n",
  ],
  [["--resume"], "Failed to compute the next version\n"],
  [["--resume", "123", "--unknown"], "Unknown release arguments\n"],
  [["unsupported"], "Failed to compute the next version\n"],
  [["--retire"], "Retirement requires explicit original PR/H/tag/R/operator-run identities\n"],
  [
    [
      "--retire",
      "123",
      "--head",
      "bad",
      "--tag",
      "v0.441.0",
      "--run",
      "12",
      "--operator-run",
      "13",
    ],
    "Invalid original retirement identities\n",
  ],
] as const) {
  test(`invalid release mode fails before dispatch ${args.join(" ")}`, () => {
    const fixture = releaseCliFixture();
    try {
      const result = fixture.run([...args]);
      assert.equal(result.status, 1, result.stderr + result.stdout);
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, message);
      assert.deepEqual(fixture.events(), []);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

for (const [cargo, message] of [
  [null, "Failed to read Cargo.toml\n"],
  ["not a manifest\n", "Failed to read current version from Cargo.toml\n"],
  ['[workspace.package]\nversion = "bad"\n', "Failed to compute the next version\n"],
] as const) {
  test(`invalid Cargo input fails before dispatch: ${message.trim()}`, () => {
    const fixture = releaseCliFixture();
    try {
      const cargoPath = path.join(fixture.dir, "Cargo.toml");
      if (cargo === null) fs.unlinkSync(cargoPath);
      else fs.writeFileSync(cargoPath, cargo);
      const result = fixture.run(["minor", "-y"]);
      assert.equal(result.status, 1, result.stderr + result.stdout);
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, message);
      assert.deepEqual(fixture.events(), []);
    } finally {
      fixture.dispose();
    }
  });
}
