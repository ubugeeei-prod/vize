// Publishes workspace crates to crates.io with retry and propagation checks.
// The release workflow uses this script instead of inline shell so that the
// publication order, retry policy, and post-publish verification all live in a
// single place. The script is deliberately idempotent: if a crate version is
// already published, it skips the publish call and only waits until the crate
// becomes resolvable from crates.io index queries.
//
// Cargo's local publish verification cannot topologically satisfy this
// workspace's exact-version dev-dependency cycles, so release CI performs
// workspace validation separately and this script publishes with `--no-verify`.
import { collectOutput, println, run, runMain, sleepSeconds, writeStderr } from "../command_io.ts";
import { cliArgs, moonTrim, readFileToString } from "../command_runtime.ts";
import { envPositiveInt, workspaceVersionFromContent } from "./config.ts";
import { restorePublishManifest, stagePublishManifest } from "./manifest.ts";
import {
  crateIsPublished,
  crateIsResolvable,
  crateResolutionError,
  queryCrateVersion,
  waitForResolution,
} from "./registry.ts";

/**
 * Ordered list of publishable crates. The order is topological because each
 * crate can require exact versions of earlier workspace crates.
 */
export const publishedCrates: readonly string[] = [
  "vize_oxc_parser",
  "vize_parcel_sourcemap",
  "vize_lightningcss",
  "vize_l0_derive",
  "vize_l0",
  "vize_carton",
  "vize_relief",
  "vize_l1",
  "vize_armature",
  "vize_l2",
  "vize_l3",
  "vize_l1_to_l2",
  "vize_l4",
  "vize_l2_to_l3",
  "vize_croquis",
  "vize_atelier_core",
  "vize_atelier_dom",
  "vize_atelier_vapor",
  "vize_atelier_ssr",
  "vize_atelier_sfc",
  "vize_musea",
  "vize_fresco",
  "vize_croquis_cf",
  "vize_atelier_jsx",
  "vize_marquette",
  "vize_doctor",
  "vize_canon",
  "vize_patina",
];

/** Crates without a completed Trusted Publishing handoff. */
export const manualPublishCrates: readonly string[] = [
  // Existing crate ownership and release.yml trust verified in #6086.
];

/** Existing crates that depend on a crate still awaiting manual publication. */
export const blockedByManualPublishCrates: readonly string[] = [
  // Existing crate ownership and release.yml trust verified in #6086.
];

function knownReleaseCrates(): string[] {
  return [...publishedCrates, ...manualPublishCrates, ...blockedByManualPublishCrates];
}

const USAGE =
  "Usage: node tools/support/release/publish_crates/main.ts [--dry-run] [--crate <crate>...]\n";

/**
 * Main entry point for ordered and rerun-safe crates.io publishing.
 * Each crate is published serially, retried on transient failure, and then
 * verified through Cargo resolution before the next dependent crate is
 * attempted. This avoids the common race where a dependent crate publishes
 * before crates.io index propagation catches up.
 */
export async function publishCrates(args: readonly string[]): Promise<number> {
  let dryRun = false;
  const selectedCrates: string[] = [];
  let index = 0;
  while (index < args.length) {
    const arg = args[index];
    if (arg === "--dry-run") {
      if (dryRun) {
        writeStderr(USAGE);
        return 1;
      }
      dryRun = true;
      index += 1;
    } else if (arg === "--crate") {
      if (index + 1 >= args.length) {
        writeStderr(USAGE);
        return 1;
      }
      selectedCrates.push(args[index + 1]);
      index += 2;
    } else {
      writeStderr(USAGE);
      return 1;
    }
  }

  const publishTargets: string[] = [];
  if (selectedCrates.length === 0) {
    publishTargets.push(...publishedCrates);
  } else {
    const knownCrates = knownReleaseCrates();
    for (const crateName of selectedCrates) {
      if (!knownCrates.includes(crateName)) {
        writeStderr(`Unknown release crate: ${crateName}\n${USAGE}`);
        return 1;
      }
      if (publishTargets.includes(crateName)) {
        writeStderr(`Duplicate release crate: ${crateName}\n${USAGE}`);
        return 1;
      }
      publishTargets.push(crateName);
    }
  }

  const cargoToml = readFileToString("Cargo.toml");
  if (cargoToml === null) {
    writeStderr("Failed to read Cargo.toml\n");
    return 1;
  }
  const version = workspaceVersionFromContent(cargoToml);
  if (version === null) {
    writeStderr("Failed to read current version from Cargo.toml\n");
    return 1;
  }

  const maxAttempts = envPositiveInt("PUBLISH_RETRY_LIMIT", 60);
  const delaySeconds = envPositiveInt("PUBLISH_RETRY_DELAY", 5);
  const resolutionAttempts = envPositiveInt("PUBLISH_RESOLUTION_RETRY_LIMIT", maxAttempts);
  const resolutionDelaySeconds = envPositiveInt("PUBLISH_RESOLUTION_RETRY_DELAY", delaySeconds);
  println(`Expected version: ${version}`);
  if (manualPublishCrates.length > 0) {
    println(`Deferred manual-publish crates: ${manualPublishCrates.join(", ")}`);
  }
  if (blockedByManualPublishCrates.length > 0) {
    println(`Deferred dependent crates: ${blockedByManualPublishCrates.join(", ")}`);
  }
  if (selectedCrates.length > 0) {
    println(`Selected crate publish plan: ${publishTargets.join(", ")}`);
  }

  if (dryRun) {
    let frontier: string | null = null;
    for (const crateName of publishTargets) {
      const state = await queryCrateVersion(crateName, version);
      if (state.kind === "query-failed") {
        writeStderr(`Release registry check failed: ${state.message}\n`);
        return 1;
      }
      if (state.kind === "missing") {
        frontier = crateName;
        break;
      }
      const resolutionError = await crateResolutionError(crateName, version);
      if (resolutionError !== null) {
        writeStderr(`Release registry check failed: ${resolutionError}\n`);
        return 1;
      }
      println(`${crateName} v${version} is already published and resolvable.`);
    }
    if (frontier === null) {
      println(`Every crate in the v${version} publish plan is already published and resolvable.`);
      return 0;
    }
    println(`Dry-running registry-resolvable frontier ${frontier} v${version}...`);
    const staged = stagePublishManifest(frontier);
    if (staged === null) {
      writeStderr(`Failed to prepare publish manifest for ${frontier}\n`);
      return 1;
    }
    const dryRunExit = await run("cargo", [
      "publish",
      "--dry-run",
      "--allow-dirty",
      "--no-verify",
      "-p",
      frontier,
    ]);
    if (!restorePublishManifest(staged)) {
      writeStderr(`Failed to restore publish manifest for ${frontier}\n`);
      return 1;
    }
    if (dryRunExit !== 0) {
      writeStderr(`Crate publish dry-run failed for ${frontier} v${version}.\n`);
      return 1;
    }
    println(
      `Registry dry-run covered ${frontier}; downstream crates are validated after each serial publish step.`,
    );
    return 0;
  }

  for (const crateName of publishTargets) {
    println(`Publishing ${crateName} v${version}...`);
    if (await crateIsPublished(crateName, version)) {
      println(`${crateName} v${version} is already published, skipping publish.`);
      if (
        (await waitForResolution(
          crateName,
          version,
          resolutionAttempts,
          resolutionDelaySeconds,
        )) !== 0
      ) {
        return 1;
      }
      continue;
    }

    let published = false;
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      const staged = stagePublishManifest(crateName);
      if (staged === null) {
        writeStderr(`Failed to prepare publish manifest for ${crateName}\n`);
        return 1;
      }
      const publish = await collectOutput("cargo", [
        "publish",
        "--allow-dirty",
        "--no-verify",
        "-p",
        crateName,
      ]);
      if (!restorePublishManifest(staged)) {
        writeStderr(`Failed to restore publish manifest for ${crateName}\n`);
        return 1;
      }
      if (publish.status === 0) {
        published = true;
        break;
      }
      const stdoutText = moonTrim(publish.stdout);
      const stderrText = moonTrim(publish.stderr);
      if (stdoutText !== "") {
        println(stdoutText);
      }
      if (stderrText !== "") {
        writeStderr(`${stderrText}\n`);
      }
      if (await crateIsPublished(crateName, version)) {
        println(`${crateName} v${version} was published despite a non-zero cargo publish exit.`);
        published = true;
        break;
      }
      if (await crateIsResolvable(crateName, version)) {
        println(
          `${crateName} v${version} is already resolvable despite a non-zero cargo publish exit.`,
        );
        published = true;
        break;
      }
      if (attempt >= maxAttempts) {
        writeStderr(`Timed out publishing ${crateName} v${version}.\n`);
        return 1;
      }
      println(`Retrying cargo publish for ${crateName} v${version}... (${attempt}/${maxAttempts})`);
      await sleepSeconds(delaySeconds);
    }

    if (!published) {
      return 1;
    }
    println(`${crateName} v${version} published successfully.`);
    if (
      (await waitForResolution(crateName, version, resolutionAttempts, resolutionDelaySeconds)) !==
      0
    ) {
      return 1;
    }
  }

  println("Done!");
  return 0;
}

if (import.meta.main) {
  runMain(() => publishCrates(cliArgs()));
}
