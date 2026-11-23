import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runReleaseCommand } from "./support/release-command.ts";
import { assertPublishManifestIsSanitized } from "./support/fake-cargo-publish.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { getMetadata, getPublishedCrates } from "./support/publish-crates-plan.ts";

test("publish_crates native script covers publish and idempotent dry-run modes", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-crates-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");
  const curlLogPath = path.join(tempDir, "curl.log");
  const publishedCrates = getPublishedCrates();
  const version = getMetadata().packages.find((pkg) => pkg.name === publishedCrates[0])?.version;
  assert.ok(version);
  const manifestSnapshots = new Map(
    publishedCrates.map((crateName) => {
      const manifestPath = getMetadata().packages.find(
        (pkg) => pkg.name === crateName,
      )!.manifest_path;
      return [manifestPath, fs.readFileSync(manifestPath, "utf8")] as const;
    }),
  );
  const cargoLockPath = path.join(repoRoot, "Cargo.lock");
  const originalCargoLock = fs.readFileSync(cargoLockPath, "utf8");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      readToolingProgram("commands/publish-crates/cargo-dry-run.js.in", {
        CHECK_MANIFEST: assertPublishManifestIsSanitized,
      }),
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram(
        "commands/publish-crates/curl-native-script-covers-idempotent-dry-run-modes.js",
      ),
    );

    const result = runReleaseCommand("publish_crates", [], {
      cwd: repoRoot,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CARGO_LOG: cargoLogPath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const logLines = fs.readFileSync(cargoLogPath, "utf8").trim().split("\n");
    assert.deepEqual(
      logLines,
      publishedCrates.flatMap((crateName) => [
        `publish --allow-dirty --no-verify -p ${crateName}`,
        `info --registry crates-io ${crateName}@${version}`,
      ]),
    );

    const runDryRun = (alreadyPublished: string[], extraEnv: Record<string, string> = {}) => {
      fs.writeFileSync(cargoLogPath, "");
      fs.writeFileSync(curlLogPath, "");
      return runReleaseCommand("publish_crates", ["--dry-run"], {
        cwd: repoRoot,
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          CARGO_LOG: cargoLogPath,
          CURL_LOG: curlLogPath,
          TEST_PUBLISHED_CRATES: alreadyPublished.join(","),
          TEST_PUBLISHED_VERSION: version,
          ...extraEnv,
        },
      });
    };
    const expectedFrontier = (crateName: string) =>
      ["publish", "--dry-run", "--allow-dirty", "--no-verify", "-p", crateName].join(" ");
    const expectedInfo = (crateName: string) => `info --registry crates-io ${crateName}@${version}`;

    const nonePublished = runDryRun([]);
    assert.equal(nonePublished.status, 0, nonePublished.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      expectedFrontier(publishedCrates[0]),
    ]);
    const curlCalls = fs.readFileSync(curlLogPath, "utf8").trim().split("\n");
    assert.equal(curlCalls.length, 1);
    for (const flag of [
      "--fail-with-body",
      "--connect-timeout 5",
      "--max-time 15",
      "--retry 2",
      "--retry-delay 1",
      "--retry-connrefused",
    ]) {
      assert.ok(curlCalls[0].includes(flag), `missing ${flag}`);
    }
    assert.match(curlCalls[0], /--write-out VIZE_HTTP_STATUS:%\{http_code\}/);
    assert.ok(curlCalls[0].endsWith(`/${publishedCrates[0]}/${version}`));

    const originalPartialPrefix = [
      "vize_oxc_parser",
      "vize_l0_derive",
      "vize_l0",
      "vize_carton",
      "vize_relief",
    ];
    const somePublished = publishedCrates.slice(0, 7);
    assert.deepEqual(somePublished, [
      "vize_oxc_parser",
      "vize_parcel_sourcemap",
      "vize_lightningcss",
      "vize_l0_derive",
      "vize_l0",
      "vize_carton",
      "vize_relief",
    ]);
    assert.deepEqual(
      somePublished.filter(
        (name) => !["vize_parcel_sourcemap", "vize_lightningcss"].includes(name),
      ),
      originalPartialPrefix,
    );
    const partial = runDryRun(somePublished);
    assert.equal(partial.status, 0, partial.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      ...somePublished.map(expectedInfo),
      expectedFrontier(publishedCrates[somePublished.length]),
    ]);
    assert.match(partial.stdout, /vize_carton .* already published and resolvable/i);
    assert.match(
      partial.stdout,
      new RegExp(`registry-resolvable frontier ${publishedCrates[somePublished.length]}`, "i"),
    );
    assert.equal(
      fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length,
      somePublished.length + 1,
    );

    const allPublished = runDryRun(publishedCrates);
    assert.equal(allPublished.status, 0, allPublished.stderr);
    assert.deepEqual(
      fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"),
      publishedCrates.map(expectedInfo),
    );
    assert.match(allPublished.stdout, /Every crate .* already published and resolvable/);

    for (const [envName, diagnostic] of [
      ["TEST_CURL_FAIL_CRATE", /curl exit 7/],
      ["TEST_CURL_MALFORMED_CRATE", /invalid JSON/],
      ["TEST_CURL_SCHEMA_CRATE", /version without num/],
      ["TEST_CURL_SERVER_ERROR_CRATE", /HTTP 500/],
    ] as const) {
      const failedQuery = runDryRun([], { [envName]: publishedCrates[0] });
      assert.notEqual(failedQuery.status, 0);
      assert.match(failedQuery.stderr, diagnostic);
      assert.equal(fs.readFileSync(cargoLogPath, "utf8"), "");
      assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 1);
    }

    const unresolvedCrate = somePublished[2];
    const unresolvedPrefix = runDryRun(somePublished, {
      TEST_UNRESOLVED_CRATES: unresolvedCrate,
    });
    assert.notEqual(unresolvedPrefix.status, 0);
    assert.match(unresolvedPrefix.stderr, /could not resolve published prerequisite/i);
    assert.ok(unresolvedPrefix.stderr.includes(`${unresolvedCrate}@${version}`));
    assert.deepEqual(
      fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"),
      somePublished.slice(0, 3).map(expectedInfo),
    );
    assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 3);

    const unresolvedAll = runDryRun(publishedCrates, {
      TEST_UNRESOLVED_CRATES: publishedCrates.at(-1) ?? "",
    });
    assert.notEqual(unresolvedAll.status, 0);
    assert.match(
      unresolvedAll.stderr,
      new RegExp(`could not resolve .*${publishedCrates.at(-1)}`, "i"),
    );
    assert.doesNotMatch(unresolvedAll.stdout, /Every crate/);
    const unresolvedAllCargo = fs.readFileSync(cargoLogPath, "utf8");
    assert.equal(
      unresolvedAllCargo.trim().split("\n").at(-1),
      expectedInfo(publishedCrates.at(-1) ?? ""),
    );
    assert.doesNotMatch(unresolvedAllCargo, /^publish --dry-run/m);

    const frontierFailure = runDryRun([], { TEST_FAIL_PUBLISH_DRY_RUN: "1" });
    assert.notEqual(frontierFailure.status, 0);
    assert.match(frontierFailure.stderr, /Crate publish dry-run failed/);
    assert.equal(
      fs.readFileSync(cargoLogPath, "utf8").trim(),
      expectedFrontier(publishedCrates[0]),
    );
    for (const [manifestPath, original] of manifestSnapshots) {
      assert.equal(fs.readFileSync(manifestPath, "utf8"), original);
    }
    assert.equal(fs.readFileSync(cargoLockPath, "utf8"), originalCargoLock);
    const frontierCurlLog = fs.readFileSync(curlLogPath, "utf8").trim();
    assert.notEqual(frontierCurlLog, "");
    assert.equal(frontierCurlLog.split("\n").length, 1);

    for (const invalidArgs of [["--unknown"], ["--dry-run", "extra"]]) {
      fs.writeFileSync(cargoLogPath, "");
      fs.writeFileSync(curlLogPath, "");
      const invalid = runReleaseCommand("publish_crates", invalidArgs, {
        cwd: repoRoot,
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          CARGO_LOG: cargoLogPath,
          CURL_LOG: curlLogPath,
        },
      });
      assert.notEqual(invalid.status, 0);
      assert.match(invalid.stderr, /Usage: .*publish_crates.*\[--dry-run\]/);
      assert.equal(fs.readFileSync(cargoLogPath, "utf8"), "");
      assert.equal(fs.readFileSync(curlLogPath, "utf8"), "");
    }
  } finally {
    for (const [manifestPath, original] of manifestSnapshots) {
      fs.writeFileSync(manifestPath, original);
    }
    fs.writeFileSync(cargoLockPath, originalCargoLock);
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_crates treats a non-zero cargo publish exit as success when the crate is already resolvable", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-crates-resolvable-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      readToolingProgram("commands/publish-crates/cargo-published-version.js.in", {
        CHECK_MANIFEST: assertPublishManifestIsSanitized,
      }),
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram(
        "commands/publish-crates/curl-treats-non-zero-cargo-exit-as-success-crate.js",
      ),
    );

    const result = runReleaseCommand("publish_crates", [], {
      cwd: repoRoot,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CARGO_LOG: cargoLogPath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already resolvable despite a non-zero cargo publish exit/i);
    const logLines = fs.readFileSync(cargoLogPath, "utf8").trim().split("\n");
    const failedPublish = logLines.indexOf("publish --allow-dirty --no-verify -p vize_l0");
    assert.ok(failedPublish >= 0, "the non-zero publish fixture was exercised");
    assert.match(logLines[failedPublish + 1] ?? "", /^info --registry crates-io vize_l0@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
