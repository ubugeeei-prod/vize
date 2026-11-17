import assert from "node:assert/strict";
import { test } from "node:test";

import {
  containerScript,
  parseContainerSmokeArguments,
} from "../../tools/commands/release/npm/smoke-glibc-floor-containers.ts";
import { callgrindTotal, summarize } from "../../tools/support/github/glibc-floor-bench.ts";

test("container smoke arguments require a GNU platform, a mode, and images", () => {
  assert.deepEqual(
    parseContainerSmokeArguments(
      ["--platform", "linux-arm64-gnu", "--mode", "packages", "amazonlinux:2023", "almalinux:8"],
      "/runner/node",
    ),
    {
      platform: "linux-arm64-gnu",
      mode: "packages",
      images: ["amazonlinux:2023", "almalinux:8"],
      cli: undefined,
      nodeDir: "/runner/node",
    },
  );
  for (const [args, message] of [
    [["--platform", "linux-x64-musl", "--mode", "cli", "--cli", "vize", "centos:7"], /--platform/],
    [["--platform", "linux-x64-gnu", "centos:7"], /--mode must be one of/],
    [["--platform", "linux-x64-gnu", "--mode", "cli", "centos:7"], /--mode cli requires --cli/],
    [["--platform", "linux-x64-gnu", "--mode", "binding"], /At least one container image/],
    [
      ["--platform", "linux-x64-gnu", "--mode", "binding", "--node-dir"],
      /--node-dir needs a value/,
    ],
  ] as const) {
    assert.throws(() => parseContainerSmokeArguments(args, "/runner/node"), message);
  }
});

test("each container mode runs the artifact it names", () => {
  const options = (mode: "packages" | "binding" | "cli") =>
    parseContainerSmokeArguments(
      ["--platform", "linux-x64-gnu", "--mode", mode, "--cli", ".artifacts/cli/vize", "centos:7"],
      "/runner/node",
    );
  const cli = containerScript(options("cli")).split("\n");
  assert.equal(cli[0], "set -euo pipefail");
  assert.equal(cli[2], "/work/.artifacts/cli/vize --version");
  assert.equal(
    cli.at(-1),
    "/work/.artifacts/cli/vize lint /tmp/vize-cli-smoke/App.vue --no-config",
  );

  // The quoted program spans lines, so check where it starts and what it loads.
  const binding = containerScript(options("binding"));
  assert.match(binding, /\nnode -e '\nconst binding = require\(process\.argv\[1\]\);\n/);
  assert.equal(
    binding.slice(binding.lastIndexOf("' ") + 2),
    "/work/npm/native/npm/linux-x64-gnu/vize-vitrine.linux-x64-gnu.node",
  );

  const packages = containerScript(options("packages"));
  assert.equal(
    packages.split("\n").at(-1),
    "node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks npm/native npm/native/npm/linux-x64-gnu npm/cli npm/builder/vite",
  );
});

test("the recipe benchmark reads callgrind totals and reports every variant against the baseline", () => {
  assert.equal(callgrindTotal("events: Ir\nsummary: 354272\n\nob=(2) ???\n"), 354272);
  assert.equal(callgrindTotal("events: Ir\ntotals: 12\n"), 12);
  assert.throws(() => callgrindTotal("events: Ir\n"), /no totals/);
  const measurements = [
    {
      kind: "cli" as const,
      scenario: "vize --version",
      variant: "gnu-host",
      instructions: [3_100_000, 3_100_010, 3_099_990],
      beforeMain: [1_500_000, 1_500_000, 1_500_000],
    },
    {
      kind: "cli" as const,
      scenario: "vize --version",
      variant: "musl-static",
      instructions: [1_550_000, 1_550_000, 1_550_000],
      beforeMain: [50_000, 50_000, 50_000],
    },
  ];
  const sizes = [
    { kind: "cli" as const, variant: "gnu-host", bytes: 1000, gzipBytes: 400 },
    { kind: "cli" as const, variant: "musl-static", bytes: 990, gzipBytes: 404 },
  ];
  assert.equal(
    summarize(measurements, sizes),
    [
      "| Scenario | Variant | Instructions | vs baseline | Before `main` | vs baseline |",
      "| --- | --- | --- | --- | --- | --- |",
      "| vize --version | gnu-host | 3,100,000 | baseline | 1,500,000 |  |",
      "| vize --version | musl-static | 1,550,000 | -50.00% | 50,000 | -96.67% |",
      "",
      "| Artifact | Variant | Bytes | vs baseline | gzip bytes | vs baseline |",
      "| --- | --- | --- | --- | --- | --- |",
      "| cli | gnu-host | 1,000 | baseline | 400 | baseline |",
      "| cli | musl-static | 990 | -1.00% | 404 | +1.00% |",
      "",
    ].join("\n"),
  );
});
