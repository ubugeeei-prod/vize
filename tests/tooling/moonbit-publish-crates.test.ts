import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

type CargoMetadata = {
  packages: CargoPackage[];
};

type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
  manifest_path: string;
  publish: string[] | null;
  version: string;
};

type CargoDependency = { name: string; kind: string | null; req: string };

function getScriptCrateArray(variableName: string): string[] {
  const scriptPath = path.join(repoRoot, "tools", "moon", "cmd", "publish_crates", "main.mbt");
  const script = fs.readFileSync(scriptPath, "utf8");
  const arrayBody = script.match(
    new RegExp(
      `let ${variableName}\\s*:\\s*Array\\[String\\]\\s*=\\s*\\[(?<body>[\\s\\S]*?)\\n\\]`,
      "m",
    ),
  )?.groups?.body;

  assert.ok(arrayBody, `Failed to locate ${variableName} in publish_crates`);
  return Array.from(arrayBody.matchAll(/"([^"]+)"/g), ([, crateName]) => crateName);
}

const getPublishedCrates = () => getScriptCrateArray("published_crates");
const getManualPublishCrates = () => getScriptCrateArray("manual_publish_crates");
const getBlockedByManualPublishCrates = () =>
  getScriptCrateArray("blocked_by_manual_publish_crates");

function getMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}

test("publish_crates script keeps publishable workspace dependencies ordered", () => {
  const publishedCrates = getPublishedCrates();
  const publishOrder = new Map(publishedCrates.map((crateName, index) => [crateName, index]));
  const packages = new Map(getMetadata().packages.map((pkg) => [pkg.name, pkg]));

  for (const crateName of publishedCrates) {
    const pkg = packages.get(crateName);
    assert.ok(pkg, `Missing package metadata for ${crateName}`);

    for (const dependency of pkg.dependencies) {
      const dependencyOrder = publishOrder.get(dependency.name);
      const strippedOnPublish = dependency.kind === "dev";
      if (!packages.has(dependency.name) || strippedOnPublish) continue;
      assert.ok(
        dependencyOrder != null,
        `${crateName} cannot depend on deferred workspace crate ${dependency.name}`,
      );
      const crateOrder = publishOrder.get(crateName);
      assert.ok(crateOrder != null, `Missing publish order for ${crateName}`);
      assert.ok(
        dependencyOrder < crateOrder,
        `${crateName} must be published after ${dependency.name}`,
      );
    }
  }
});

test("publish_crates exactly partitions every publishable workspace crate", () => {
  const releaseCrates = [
    ...getPublishedCrates(),
    ...getManualPublishCrates(),
    ...getBlockedByManualPublishCrates(),
  ];
  const publishableCrates = getMetadata()
    .packages.filter((pkg) =>
      path.relative(repoRoot, pkg.manifest_path).startsWith(`crates${path.sep}`),
    )
    .filter((pkg) => pkg.publish === null || pkg.publish.length > 0)
    .map((pkg) => pkg.name);

  assert.equal(new Set(releaseCrates).size, releaseCrates.length, "release crate lists overlap");
  assert.deepEqual(releaseCrates.toSorted(), publishableCrates.toSorted());
});

test("publish_crates defers crates that still need manual crates.io handoff", () => {
  assert.deepEqual(getManualPublishCrates(), [
    "vize_croquis_cf",
    "vize_atelier_jsx",
    "vize_marquette",
    "vize_doctor",
  ]);
});

test("publish_crates only blocks crates that depend on manual-publish exclusions", () => {
  assert.deepEqual(getBlockedByManualPublishCrates(), ["vize_canon", "vize_patina"]);
});

test("publish_crates native script covers publish and idempotent dry-run modes", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");
  const curlLogPath = path.join(tempDir, "curl.log");
  const publishedCrates = getPublishedCrates();
  const version = getMetadata().packages.find((pkg) => pkg.name === publishedCrates[0])?.version;
  assert.ok(version);

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
        "const [command] = args;",
        "if (command === 'publish' && args.includes('--dry-run') && process.env.TEST_FAIL_PUBLISH_DRY_RUN) process.exit(1);",
        "const unresolved = (process.env.TEST_UNRESOLVED_CRATES || '').split(',');",
        "if (command === 'info' && unresolved.includes(args.at(-1).split('@')[0])) { console.error('not in registry index'); process.exit(1); }",
        "if (command === 'package' || command === 'publish' || command === 'info') process.exit(0);",
        "process.exit(1);",
      ].join("\n"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "if (process.env.CURL_LOG) fs.appendFileSync(process.env.CURL_LOG, args.join(' ') + '\\n');",
        "const endpoint = args.at(-1).split('/');",
        "const crateName = endpoint.at(-2);",
        "const version = endpoint.at(-1);",
        "if (crateName === process.env.TEST_CURL_FAIL_CRATE) { console.error('registry unavailable'); process.exit(7); }",
        "if (crateName === process.env.TEST_CURL_MALFORMED_CRATE) { fs.writeSync(1, '{bad jsonVIZE_HTTP_STATUS:200'); process.exit(0); }",
        "if (crateName === process.env.TEST_CURL_SCHEMA_CRATE) { fs.writeSync(1, JSON.stringify({ version: { crate: crateName } }) + 'VIZE_HTTP_STATUS:200'); process.exit(0); }",
        "if (crateName === process.env.TEST_CURL_SERVER_ERROR_CRATE) { fs.writeSync(1, JSON.stringify({ errors: [{ detail: 'unavailable' }] }) + 'VIZE_HTTP_STATUS:500'); process.exit(22); }",
        "const published = (process.env.TEST_PUBLISHED_CRATES || '').split(',').includes(crateName);",
        "const body = published ? { version: { crate: crateName, num: version } } : { errors: [{ detail: 'Not Found' }] };",
        "fs.writeSync(1, JSON.stringify(body) + 'VIZE_HTTP_STATUS:' + (published ? '200' : '404'));",
        "process.exit(published ? 0 : 22);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_crates", [], {
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
    assert.match(logLines[0] ?? "", /^publish --locked -p vize_carton$/);
    assert.match(logLines[1] ?? "", /^info --registry crates-io vize_carton@/);
    assert.match(logLines.at(-2) ?? "", /^publish --locked -p vize_fresco$/);
    assert.match(logLines.at(-1) ?? "", /^info --registry crates-io vize_fresco@/);

    const runDryRun = (alreadyPublished: string[], extraEnv: Record<string, string> = {}) => {
      fs.writeFileSync(cargoLogPath, "");
      fs.writeFileSync(curlLogPath, "");
      return runMoonScript("publish_crates", ["--dry-run"], {
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
      ["publish", "--dry-run", "--locked", "-p", crateName].join(" ");
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
    assert.ok(curlCalls[0].endsWith(`/vize_carton/${version}`));

    const somePublished = publishedCrates.slice(0, 5);
    const partial = runDryRun(somePublished);
    assert.equal(partial.status, 0, partial.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      ...somePublished.map(expectedInfo),
      expectedFrontier(publishedCrates[5]),
    ]);
    assert.match(partial.stdout, /vize_carton .* already published and resolvable/i);
    assert.match(partial.stdout, /registry-resolvable frontier vize_s1/i);
    assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 6);

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

    const unresolvedPrefix = runDryRun(somePublished, {
      TEST_UNRESOLVED_CRATES: publishedCrates[2],
    });
    assert.notEqual(unresolvedPrefix.status, 0);
    assert.match(unresolvedPrefix.stderr, /could not resolve .*vize_davinci/i);
    assert.deepEqual(
      fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"),
      somePublished.slice(0, 3).map(expectedInfo),
    );
    assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 3);

    const unresolvedAll = runDryRun(publishedCrates, {
      TEST_UNRESOLVED_CRATES: publishedCrates.at(-1) ?? "",
    });
    assert.notEqual(unresolvedAll.status, 0);
    assert.match(unresolvedAll.stderr, /could not resolve .*vize_fresco/i);
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
    const frontierCurlLog = fs.readFileSync(curlLogPath, "utf8").trim();
    assert.notEqual(frontierCurlLog, "");
    assert.equal(frontierCurlLog.split("\n").length, 1);

    for (const invalidArgs of [["--unknown"], ["--dry-run", "extra"]]) {
      fs.writeFileSync(cargoLogPath, "");
      fs.writeFileSync(curlLogPath, "");
      const invalid = runMoonScript("publish_crates", invalidArgs, {
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
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_crates treats a non-zero cargo publish exit as success when the crate is already resolvable", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-resolvable-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
        "if (args[0] === 'publish' && args.at(-1) === 'vize_carton') process.exit(1);",
        "if (args[0] === 'publish' || args[0] === 'info') process.exit(0);",
        "process.exit(1);",
      ].join("\n"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      [
        "require('node:fs').writeSync(1, JSON.stringify({ errors: [{ detail: 'Not Found' }] }) + 'VIZE_HTTP_STATUS:404');",
        "process.exit(22);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_crates", [], {
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
    assert.equal(logLines[0], "publish --locked -p vize_carton");
    assert.match(logLines[1] ?? "", /^info --registry crates-io vize_carton@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
