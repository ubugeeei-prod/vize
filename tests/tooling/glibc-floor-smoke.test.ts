import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  containerCommand,
  parseContainerSmokeArguments,
} from "../../tools/commands/release/npm/smoke-glibc-floor-containers.ts";
import { roundOrder, summarize } from "../../tools/support/github/glibc-floor-bench.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

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
  const driver = "/work/tools/support/compat/npm/glibc-floor-container.sh";
  for (const mode of ["cli", "binding", "packages"] as const) {
    const cli = mode === "cli" ? [".artifacts/cli/vize"] : [];
    assert.deepEqual(containerCommand(options(mode)), [
      "bash",
      driver,
      mode,
      "linux-x64-gnu",
      ...cli,
    ]);
  }
  const script = fs.readFileSync(path.join(root, driver.slice("/work/".length)), "utf8");
  assert.match(script, /"\$binary" --version/u);
  assert.match(script, /"\$binary" lint \/tmp\/vize-cli-smoke\/App\.vue --no-config/u);
  assert.match(
    script,
    /exec node \/work\/tools\/support\/compat\/npm\/glibc-floor-binding\.cjs "\/work\/npm\/native\/npm\/\$platform\/vize-vitrine\.\$platform\.node"/u,
  );
  assert.match(
    script,
    /exec node tools\/commands\/release\/npm\/smoke-release-install\.ts --prepare-manifests --runtime-checks npm\/native npm\/native\/npm\/\* npm\/cli npm\/builder\/unplugin npm\/builder\/vite/u,
  );
});

test("the package floor builds every JavaScript package it packs", () => {
  const workflow = readAutomationPrograms(".github/workflows/release-glibc-floor.yml");
  assert.match(
    workflow,
    /vp run --filter '\.\/npm\/cli' build\n\s+vp run --filter '\.\/npm\/builder\/unplugin' build\n\s+vp run --filter '\.\/npm\/builder\/vite' build/u,
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
