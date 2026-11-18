import assert from "node:assert/strict";
import { test } from "node:test";

import {
  containerScript,
  parseContainerSmokeArguments,
} from "../../tools/commands/release/npm/smoke-glibc-floor-containers.ts";
import { roundOrder, summarize } from "../../tools/support/github/glibc-floor-bench.ts";

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
    "node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks npm/native npm/native/npm/linux-x64-gnu npm/cli npm/builder/unplugin npm/builder/vite",
  );
});

test("the recipe benchmark alternates variant order and reports the floor against host", () => {
  assert.deepEqual(roundOrder(["host", "floor"], 0), ["host", "floor"]);
  assert.deepEqual(roundOrder(["host", "floor"], 1), ["floor", "host"]);
  const samples = [1, 2, 3].flatMap((round) => [
    { scenario: "cli-lint", variant: "host", round, seconds: 1 + round / 10 },
    { scenario: "cli-lint", variant: "floor", round, seconds: 1.1 + round / 10 },
  ]);
  assert.equal(
    summarize(samples),
    [
      "| Scenario | Variant | Rounds | Median (s) | Min (s) | Max (s) | vs host |",
      "| --- | --- | --- | --- | --- | --- | --- |",
      "| cli-lint | host | 3 | 1.2000 | 1.1000 | 1.3000 |  |",
      "| cli-lint | floor | 3 | 1.3000 | 1.2000 | 1.4000 | 8.3% |",
      "",
    ].join("\n"),
  );
});
