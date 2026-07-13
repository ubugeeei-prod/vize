import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import {
  installPublishCratesFakes,
  installPublishedVersionCurl,
} from "./support/publish-crates-fakes.ts";

type CargoMetadata = { packages: CargoPackage[] };

type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
  manifest_path: string;
  publish: string[] | null;
  version: string;
};

type CargoDependency = { name: string };

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
const getPendingFirstPublishCrates = () => getScriptCrateArray("pending_first_publish_crates");
const getBlockedByPendingFirstPublishCrates = () =>
  getScriptCrateArray("blocked_by_pending_first_publish_crates");

const getManualBootstrapCrates = () => getScriptCrateArray("manual_bootstrap_crates");

function getMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}

function assertTopologicalPublishOrder(crateNames: string[]): void {
  const publishOrder = new Map(crateNames.map((crateName, index) => [crateName, index]));
  const packages = new Map(getMetadata().packages.map((pkg) => [pkg.name, pkg]));

  for (const crateName of crateNames) {
    const pkg = packages.get(crateName);
    assert.ok(pkg, `Missing package metadata for ${crateName}`);

    for (const dependency of pkg.dependencies) {
      const dependencyOrder = publishOrder.get(dependency.name);
      if (!packages.has(dependency.name)) continue;
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
}

test("publish_crates script keeps publishable workspace dependencies ordered", () => {
  assertTopologicalPublishOrder(getPublishedCrates());
});

test("publish_crates exactly partitions every publishable workspace crate", () => {
  const releaseCrates = [
    ...getPublishedCrates(),
    ...getPendingFirstPublishCrates(),
    ...getBlockedByPendingFirstPublishCrates(),
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

test("publish_crates only defers crates that have not been created on crates.io", () => {
  assert.deepEqual(getPendingFirstPublishCrates(), [
    "vize_atlas",
    "vize_rendu",
    "vize_flow",
    "vize_module",
    "vize_croquis_cf",
    "vize_atelier_jsx",
    "vize_atelier_template",
  ]);
});

test("publish_crates only blocks crates that depend on first-publish exclusions", () => {
  assert.deepEqual(getBlockedByPendingFirstPublishCrates(), [
    "vize_relief",
    "vize_armature",
    "vize_croquis",
    "vize_atelier_core",
    "vize_atelier_dom",
    "vize_atelier_vapor",
    "vize_atelier_ssr",
    "vize_atelier_sfc",
    "vize_musea",
    "vize_canon",
    "vize_patina",
  ]);
});

test("publish_crates has one complete topological manual bootstrap", () => {
  const crates = getManualBootstrapCrates();
  assert.deepEqual(crates, [
    "vize_carton",
    "vize_atlas",
    "vize_fresco",
    "vize_flow",
    "vize_module",
    "vize_relief",
    "vize_rendu",
    "vize_armature",
    "vize_croquis",
    "vize_atelier_core",
    "vize_croquis_cf",
    "vize_atelier_dom",
    "vize_atelier_ssr",
    "vize_atelier_vapor",
    "vize_atelier_template",
    "vize_atelier_sfc",
    "vize_atelier_jsx",
    "vize_musea",
    "vize_canon",
    "vize_patina",
  ]);
  assertTopologicalPublishOrder(crates);
});

test("publish_crates requires an explicit bootstrap confirmation", () => {
  const result = runMoonScript("publish_crates", ["--bootstrap"], { cwd: repoRoot });

  assert.equal(result.status, 1);
  assert.match(result.stderr, /VIZE_CRATES_IO_BOOTSTRAP=1/);
});

test("publish_crates refuses to mix a bootstrap with an existing version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-bootstrap-version-"));
  const binDir = path.join(tempDir, "bin");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    installPublishedVersionCurl(binDir);

    const result = runMoonScript("publish_crates", ["--bootstrap"], {
      cwd: repoRoot,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VIZE_CRATES_IO_BOOTSTRAP: "1",
      },
    });

    assert.equal(result.status, 1);
    assert.match(result.stderr, /already contains vize_carton/);
    assert.match(result.stderr, /one unpublished version/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
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
    installPublishCratesFakes(binDir);

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
    const expectedPackage = [
      "package",
      "--locked",
      "--no-verify",
      ...publishedCrates.flatMap((name) => ["-p", name]),
    ].join(" ");
    const expectedFrontier = (crateName: string) =>
      ["publish", "--dry-run", "--locked", "-p", crateName].join(" ");
    const expectedInfo = (crateName: string) => `info --registry crates-io ${crateName}@${version}`;

    const nonePublished = runDryRun([]);
    assert.equal(nonePublished.status, 0, nonePublished.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      expectedPackage,
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

    const somePublished = publishedCrates.slice(0, 1);
    const partial = runDryRun(somePublished);
    assert.equal(partial.status, 0, partial.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      expectedPackage,
      ...somePublished.map(expectedInfo),
      expectedFrontier(publishedCrates[1]),
    ]);
    assert.match(partial.stdout, /vize_carton .* already published and resolvable/i);
    assert.match(partial.stdout, /registry-resolvable frontier vize_fresco/i);
    assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 2);

    const allPublished = runDryRun(publishedCrates);
    assert.equal(allPublished.status, 0, allPublished.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      expectedPackage,
      ...publishedCrates.map(expectedInfo),
    ]);
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
      assert.equal(fs.readFileSync(cargoLogPath, "utf8").trim(), expectedPackage);
      assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 1);
    }

    const unresolvedPrefix = runDryRun(somePublished, {
      TEST_UNRESOLVED_CRATES: publishedCrates[0],
    });
    assert.notEqual(unresolvedPrefix.status, 0);
    assert.match(unresolvedPrefix.stderr, /could not resolve .*vize_carton/i);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      expectedPackage,
      ...somePublished.map(expectedInfo),
    ]);
    assert.equal(fs.readFileSync(curlLogPath, "utf8").trim().split("\n").length, 1);

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

    const packageFailure = runDryRun([], { TEST_FAIL_PACKAGE: "1" });
    assert.notEqual(packageFailure.status, 0);
    assert.match(packageFailure.stderr, /Crate package validation failed/);
    assert.equal(fs.readFileSync(cargoLogPath, "utf8").trim(), expectedPackage);
    assert.equal(fs.readFileSync(curlLogPath, "utf8"), "");

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
