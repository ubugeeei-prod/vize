import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { releaseRegistryFixture, type RegistryEvent } from "./support/release-registry-fixture.ts";
import { runRepositoryGuardFixture } from "./support/release-guard-fixture.ts";

function assertFresh(events: RegistryEvent[], kinds: string[], cargo: string) {
  assert.deepEqual(
    events.map((event) => event.kind),
    kinds,
  );
  assert.deepEqual(events[0]?.args, ["update"]);
  assert.equal(events[0]?.completionDirectories.length, 1);
  assert.equal(new Set(events.map((event) => event.pid)).size, events.length);
  for (const event of events) {
    assert.ok(event.pid > 0);
    assert.equal(event.marker, null, "task marker must not leak into child operations");
    assert.equal(event.cargo, cargo, "freshness and driver dispatch precede file mutation");
  }
}

test(
  "native release declines before registry transport through the original terminal prompt",
  {
    skip: process.platform === "win32",
  },
  () => {
    const fixture = releaseRegistryFixture(37);
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
  "native release refreshes once after affirmative confirmation and before driver dispatch",
  {
    skip: process.platform === "win32",
  },
  () => {
    const fixture = releaseRegistryFixture();
    try {
      const result = fixture.prompt("y\n", ["minor"]);
      const output = result.stdout + result.stderr;
      assert.equal(result.error, undefined, output);
      assert.equal(result.status, 83, output);
      assert.ok(output.indexOf("Proceed with release? [y/N]") < output.indexOf("registry stdout"));
      assertFresh(fixture.events(), ["update", "driver"], fixture.cargo);
      assert.deepEqual(fixture.events()[1]?.args, [
        "tools/commands/release/pr.rs",
        "start",
        "minor",
      ]);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  },
);

for (const extra of [[], ["--pin"]]) {
  test(`native release -y awaits freshness before the existing driver ${extra.join(" ")}`, () => {
    const fixture = releaseRegistryFixture();
    try {
      const result = fixture.run(["minor", "-y", ...extra]);
      assert.equal(result.status, 83, result.stderr + result.stdout);
      assert.doesNotMatch(result.stdout, /Proceed with release/);
      assertFresh(fixture.events(), ["update", "driver"], fixture.cargo);
      assert.deepEqual(fixture.events()[1]?.args, [
        "tools/commands/release/pr.rs",
        "start",
        "minor",
        ...extra,
      ]);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

test("native release preserves registry failure status and raw streams before any mutation", () => {
  const fixture = releaseRegistryFixture(37);
  try {
    const result = fixture.run(["minor", "-y"]);
    assert.equal(result.status, 37, result.stderr + result.stdout);
    assert.equal(result.stderr, "registry stderr\n");
    assert.equal(
      result.stdout,
      "Current version: 0.440.0\nNew version: 0.441.0 (tag: v0.441.0)\n\nregistry stdout\n",
    );
    assertFresh(fixture.events(), ["update"], fixture.cargo);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});

test(
  "a terminated registry child cannot become successful freshness before mutation",
  {
    skip: process.platform === "win32",
  },
  () => {
    const fixture = releaseRegistryFixture(0, false, true);
    try {
      const result = fixture.run(["minor", "-y"]);
      assert.equal(result.status, 143, result.stderr + result.stdout);
      assert.match(result.stdout, /registry stdout\n$/);
      assert.match(result.stderr, /registry stderr\n/);
      assertFresh(fixture.events(), ["update"], fixture.cargo);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  },
);

for (const signal of ["SIGTERM", "SIGKILL"] as const) {
  test(
    `an actually terminated registry shell (${signal}) cannot authorize mutation`,
    {
      skip: process.platform === "win32",
    },
    () => {
      const fixture = releaseRegistryFixture(0, false, false, signal);
      try {
        const result = fixture.run(["minor", "-y"]);
        assert.ok(
          [1, signal === "SIGTERM" ? 15 : 9].includes(result.status ?? -1),
          result.stderr + result.stdout,
        );
        assertFresh(fixture.events(), ["update"], fixture.cargo);
        assert.ok((fixture.events()[0]?.shellPid ?? 0) > 1);
        assert.match(fixture.events()[0]?.shellCommand ?? "", /node "\$@"/);
        assert.match(result.stderr, /Registry freshness did not complete/);
        fixture.assertUnchanged();
      } finally {
        fixture.dispose();
      }
    },
  );
}

test("native prepare-only refreshes while metadata is original, then performs the existing candidate mutation", () => {
  const fixture = releaseRegistryFixture();
  let prepared: ReturnType<typeof runRepositoryGuardFixture> | undefined;
  try {
    prepared = runRepositoryGuardFixture({
      branch: "main",
      env: {
        MOON_BIN: fixture.env.MOON_BIN,
        MOON_HOME: fixture.env.MOON_HOME,
        REAL_MOON_BIN: fixture.env.REAL_MOON_BIN,
        REGISTRY_EVENTS: fixture.env.REGISTRY_EVENTS,
        TMPDIR: fixture.env.TMPDIR,
        TMP: fixture.env.TMP,
        TEMP: fixture.env.TEMP,
        VIZE_RELEASE_REGISTRY_REFRESH: "1",
      },
    });
    assert.equal(prepared.result.status, 0, prepared.result.stderr + prepared.result.stdout);
    assertFresh(fixture.events(), ["update"], prepared.cargoToml);
    assert.equal(
      fs.readFileSync(prepared.cargoTomlPath, "utf8"),
      prepared.cargoToml.replace("0.290.0", "0.290.1"),
    );
    assert.match(prepared.gitLog, /^commit --no-verify -m chore: release v0\.290\.1$/m);
    assert.doesNotMatch(prepared.gitLog, /^(?:tag|push)\b/m);
  } finally {
    if (prepared) fs.rmSync(prepared.tempDir, { recursive: true, force: true });
    fixture.dispose();
  }
});

const pureModes = [
  ["--print-bump", "0.440.0", "minor"],
  ["--print-readme-version-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-workspace-manifest-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-workspace-catalog-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-lockfile-catalog-update", "Cargo.toml", "0.440.0", "0.441.0"],
  ["--print-extra-package-json-paths"],
];
for (const args of pureModes) {
  test(`task-requested freshness precedes pure mode ${args[0]}`, () => {
    const fixture = releaseRegistryFixture();
    try {
      const result = fixture.run(args);
      assert.equal(result.status, 0, result.stderr + result.stdout);
      assert.ok(result.stdout.startsWith("registry stdout\n"));
      assert.equal(result.stderr, "registry stderr\n");
      assertFresh(fixture.events(), ["update"], fixture.cargo);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

const invalidModes = [
  [],
  ["minor", "--unknown"],
  ["minor", "--force-tag"],
  ["minor", "--prepare-only", "--pin"],
  ["minor", "--pinned-target"],
  ["minor", "--pinned-target", "0.441.0"],
  ["minor", "--prepare-only", "--pinned-target", "0.441.0", "--pinned-target", "0.441.0"],
  ["patch", "--prepare-only", "--pinned-target", "0.441.0"],
  ["--resume"],
  ["--resume", "123", "--unknown"],
  ["unsupported"],
  ["--retire"],
  ["--retire", "123", "--head", "bad", "--tag", "v0.441.0", "--run", "12", "--operator-run", "13"],
];
for (const args of invalidModes) {
  test(`task-requested freshness retains invalid-mode precedence ${args.join(" ")}`, () => {
    const fixture = releaseRegistryFixture(37);
    try {
      const result = fixture.run(args);
      assert.equal(result.status, 37, result.stderr + result.stdout);
      assert.equal(result.stdout, "registry stdout\n");
      assert.equal(result.stderr, "registry stderr\n");
      assertFresh(fixture.events(), ["update"], fixture.cargo);
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

for (const cargo of [null, "not a manifest\n", '[workspace.package]\nversion = "bad"\n']) {
  test(`task freshness precedes invalid Cargo input ${cargo}`, () => {
    const fixture = releaseRegistryFixture(37);
    try {
      const cargoPath = path.join(fixture.dir, "Cargo.toml");
      if (cargo === null) fs.unlinkSync(cargoPath);
      else fs.writeFileSync(cargoPath, cargo);
      const result = fixture.run(["minor", "-y"]);
      assert.equal(result.status, 37, result.stderr + result.stdout);
      assert.equal(result.stdout, "registry stdout\n");
      assert.equal(result.stderr, "registry stderr\n");
      assert.deepEqual(
        fixture.events().map((event) => event.kind),
        ["update"],
      );
    } finally {
      fixture.dispose();
    }
  });
}

const driverModes = [
  ["--resume", "123"],
  ["--resume", "123", "--pin"],
  [
    "--retire",
    "123",
    "--head",
    "a".repeat(40),
    "--tag",
    "v0.441.0",
    "--run",
    "12",
    "--operator-run",
    "13",
  ],
];
for (const args of driverModes) {
  test(`task freshness precedes existing driver mode ${args[0]} ${args.length}`, () => {
    const fixture = releaseRegistryFixture();
    try {
      const result = fixture.run(args);
      assert.equal(result.status, 83, result.stderr + result.stdout);
      assertFresh(fixture.events(), ["update", "driver"], fixture.cargo);
      assert.deepEqual(
        fixture.events()[1]?.args,
        args[0] === "--resume"
          ? ["tools/commands/release/pr.rs", "resume", ...args.slice(1)]
          : ["tools/commands/release/pr.rs", "retire", args[1], args[3], args[5], args[7], args[9]],
      );
      fixture.assertUnchanged();
    } finally {
      fixture.dispose();
    }
  });
}

test("pinned-target authentication retains eager freshness and its original error precedence", () => {
  const fixture = releaseRegistryFixture();
  try {
    const result = fixture.run(["minor", "--prepare-only", "--pinned-target", "0.441.0"]);
    assert.equal(result.status, 1, result.stderr + result.stdout);
    assert.equal(result.stdout, "registry stdout\n");
    assert.equal(
      result.stderr,
      "registry stderr\nPinned target was not authenticated; no release files changed\n",
    );
    assertFresh(fixture.events(), ["update", "authenticate"], fixture.cargo);
    assert.deepEqual(fixture.events()[1]?.args, [
      "tools/commands/release/pr.rs",
      "validate-preparation-target",
      "minor",
      "0.440.0",
      "0.441.0",
    ]);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});

test("authenticated pinned preparation consumes freshness once before the existing guard", () => {
  const fixture = releaseRegistryFixture(0, true);
  try {
    const result = fixture.run(["minor", "-y", "--prepare-only", "--pinned-target", "0.441.0"]);
    assert.equal(result.status, 1, result.stderr + result.stdout);
    assertFresh(fixture.events(), ["update", "authenticate", "driver"], fixture.cargo);
    assert.match(result.stderr, /Release preflight failed before repository mutation/);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});
