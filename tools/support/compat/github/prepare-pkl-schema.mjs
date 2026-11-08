import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { toolingTestCommand } from "./run-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const schemaTest = "tests/tooling/config-generation.test.ts";
const pklPantryAssets = new Map([
  [
    "package://pkg.pkl-lang.org/pkl-pantry/org.json_schema@1.1.0",
    { metadata: 262739219, zip: 262739222 },
  ],
  [
    "package://pkg.pkl-lang.org/pkl-pantry/pkl.experimental.uri@1.0.3",
    { metadata: 174336285, zip: 174336284 },
  ],
]);

const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");

export function recoverPklPackage(uri, cache, download, assets = pklPantryAssets) {
  const match =
    /^(package:\/\/pkg\.pkl-lang\.org\/pkl-pantry\/([a-z0-9._-]+)@(\d+\.\d+\.\d+))::sha256:([a-f0-9]{64})$/.exec(
      uri,
    );
  const pinned = match && assets.get(match[1]);
  if (!match || !pinned) throw new Error(`No pinned Pkl recovery asset for ${uri}`);
  const [, packageUri, name, version, metadataChecksum] = match;
  const temporary = mkdtempSync(join(cache, "pkl-release-recovery-"));
  const metadataTemporary = join(temporary, "metadata.json");
  const zipTemporary = join(temporary, "package.zip");
  try {
    for (const [id, path] of [
      [pinned.metadata, metadataTemporary],
      [pinned.zip, zipTemporary],
    ]) {
      const result = download(id, path);
      if (result.status !== 0 || result.error) {
        throw new Error(
          `Pinned Pkl recovery asset ${id} failed: ${result.error?.message ?? ""}\n${result.stderr ?? ""}`,
        );
      }
    }
    if (sha256(metadataTemporary) !== metadataChecksum) {
      throw new Error("Pinned Pkl recovery metadata checksum mismatch");
    }
    const metadata = JSON.parse(readFileSync(metadataTemporary, "utf8"));
    if (
      metadata.packageUri !== packageUri ||
      !/^[a-f0-9]{64}$/.test(metadata.packageZipChecksums?.sha256 ?? "") ||
      sha256(zipTemporary) !== metadata.packageZipChecksums.sha256
    ) {
      throw new Error("Pinned Pkl recovery package identity or zip checksum mismatch");
    }
    const target = join(cache, "package-2", "pkg.pkl-lang.org", "pkl-pantry", `${name}@${version}`);
    mkdirSync(target, { recursive: true });
    renameSync(metadataTemporary, join(target, `${name}@${version}.json`));
    renameSync(zipTemporary, join(target, `${name}@${version}.zip`));
  } finally {
    rmSync(temporary, { force: true, recursive: true });
  }
}

export function needsPklSchemaCache(plan, available, shard, tier) {
  if (!["pr", "merge"].includes(tier) || plan.tier !== tier) {
    throw new Error("Pkl preparation plan does not match the required execution tier");
  }
  return toolingTestCommand(plan, available, shard).slice(2).includes(schemaTest);
}

export function pinnedPklPackages(lock) {
  if (lock.schemaVersion !== 1 || !lock.resolvedDependencies) {
    throw new Error("invalid committed Pkl dependency lock");
  }
  const packages = Object.entries(lock.resolvedDependencies).map(([key, dependency]) => {
    const match =
      /^projectpackage:\/\/(pkg\.pkl-lang\.org\/pkl-pantry\/[a-z0-9._-]+)@(\d+)\.\d+\.\d+$/.exec(
        dependency.uri ?? "",
      );
    const checksum = dependency.checksums?.sha256;
    if (
      dependency.type !== "remote" ||
      !match ||
      key !== `package://${match[1]}@${match[2]}` ||
      !/^[a-f0-9]{64}$/.test(checksum ?? "")
    ) {
      throw new Error("Pkl dependency must retain its exact remote version and checksum");
    }
    return `${dependency.uri.replace(/^projectpackage:/, "package:")}::sha256:${checksum}`;
  });
  if (packages.length === 0 || new Set(packages).size !== packages.length) {
    throw new Error("committed Pkl dependency lock is empty or duplicated");
  }
  return packages;
}

export function verifyPklRuntime(manifest, version, run) {
  if (
    !/^\d+\.\d+\.\d+$/.test(version ?? "") ||
    manifest.name !== "@pkl-community/pkl" ||
    manifest.version !== version
  ) {
    throw new Error("installed Pkl package does not match the committed catalog version");
  }
  const result = run(["--version"], 10000);
  if (result.status !== 0 || !result.stdout?.startsWith(`Pkl ${version} (`)) {
    throw new Error("actual Pkl CLI does not match the committed catalog version");
  }
}

export function downloadPklPackages(packages, cache, run, recover) {
  for (const uri of packages) {
    for (let attempt = 1; attempt <= 3; attempt += 1) {
      let result = run(["download-package", "--no-transitive", "--cache-dir", cache, uri], 30000);
      if (result.status === 0 && !result.error) break;
      const githubReleaseFailure =
        /Received unexpected status code `50[0234]`[^]*github\.com\/apple\/pkl-pantry\/releases\/download/.test(
          result.stderr ?? "",
        );
      if (githubReleaseFailure && recover) {
        recover(uri, cache);
        result = run(["download-package", "--no-transitive", "--cache-dir", cache, uri], 30000);
        if (result.status === 0 && !result.error) break;
      }
      const timeout =
        !/checksum|digest|invalid|unknown/i.test(result.stderr ?? "") &&
        (result.error?.code === "ETIMEDOUT" ||
          /Exception when making request[^]*request timed out/.test(result.stderr ?? ""));
      if (!timeout || attempt === 3) {
        throw new Error(
          `Pinned Pkl package download failed after ${attempt} attempt(s): ${uri}\n${result.error?.message ?? ""}\n${result.stderr ?? ""}`,
        );
      }
    }
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const plan = JSON.parse(readFileSync(process.env.VIZE_TOOLING_TEST_PLAN, "utf8"));
  if (
    needsPklSchemaCache(
      plan,
      toolingTestFiles(),
      process.env.VIZE_TOOLING_TEST_SHARD,
      process.env.VIZE_TOOLING_TEST_TIER,
    )
  ) {
    const packageRoot = join(root, "npm/cli/node_modules/@pkl-community/pkl");
    const version = parse(readFileSync(join(root, "pnpm-workspace.yaml"), "utf8")).catalogs[
      "repo-tooling"
    ]["@pkl-community/pkl"];
    const pkl = createRequire(import.meta.url)(join(packageRoot, "lib/index.js")).getExePath();
    const run = (args, timeout) =>
      spawnSync(pkl, args, {
        cwd: root,
        encoding: "utf8",
        timeout,
      });
    verifyPklRuntime(
      JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8")),
      version,
      run,
    );
    const packages = pinnedPklPackages(
      JSON.parse(readFileSync(join(root, "npm/cli/pkl/PklProject.deps.json"), "utf8")),
    );
    // Pkl trusts existing cache bytes. Start fresh rather than restoring a cache
    // whose contents would not be revalidated against the committed checksums.
    const cache = mkdtempSync(join(process.env.RUNNER_TEMP ?? tmpdir(), "vize-pkl-schema-"));
    const download = (asset, output) => {
      const headers = [
        "--header",
        "Accept: application/octet-stream",
        "--header",
        "X-GitHub-Api-Version: 2022-11-28",
      ];
      if (process.env.GH_TOKEN)
        headers.push("--header", `Authorization: Bearer ${process.env.GH_TOKEN}`);
      return spawnSync(
        "curl",
        [
          "--fail-with-body",
          "--show-error",
          "--location",
          "--retry",
          "3",
          "--retry-all-errors",
          ...headers,
          "--output",
          output,
          `https://api.github.com/repos/apple/pkl-pantry/releases/assets/${asset}`,
        ],
        { cwd: root, encoding: "utf8", timeout: 90000 },
      );
    };
    downloadPklPackages(packages, cache, run, (uri, destination) =>
      recoverPklPackage(uri, destination, download),
    );
    if (!process.env.GITHUB_ENV)
      throw new Error("Pkl preparation requires the Actions environment file");
    appendFileSync(process.env.GITHUB_ENV, `VIZE_PKL_SCHEMA_CACHE=${cache}\n`);
    process.stdout.write(
      `Pkl ${version}: verified ${packages.join(", ")} in fresh cache ${cache}\n`,
    );
  } else {
    process.stdout.write("This tooling worker does not select config generation.\n");
  }
}
