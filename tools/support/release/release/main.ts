// Confirm the requested version bump, then create and watch a release PR.
// The internal prepare-only mode edits one isolated worktree without a tag.
import { println, run, runMain, writeStderr, writeStdout } from "../command_io.ts";
import { cliArgs, readFileToString } from "../command_runtime.ts";
import { prepareCandidate } from "./apply_versions.ts";
import {
  replaceNativeBinaryLockfileCatalog,
  replaceNativeBinaryWorkspaceCatalog,
} from "./native_catalog.ts";
import {
  confirmRelease,
  extraReleasePackageJsonPaths,
  hasInteractiveTerminal,
} from "./preparation.ts";
import { replaceReadmeVersionReferences } from "./readmes.ts";
import {
  bumpVersion,
  updateWorkspaceManifestVersions,
  workspaceVersionFromContent,
} from "./versions.ts";

const RELEASE_DRIVER = "tools/commands/release/pr.rs";

function fail(message: string): number {
  writeStderr(message);
  return 1;
}

function positiveRetirementIdentity(value: string): boolean {
  return value !== "" && !value.startsWith("0") && /^[0-9]+$/.test(value);
}

function isFullCommitIdentity(value: string): boolean {
  return value.length === 40 && /^[0-9a-fA-F]+$/.test(value);
}

/** Runs a pure rewrite mode over one input file and prints its result. */
function printRewrite(
  inputPath: string,
  rewrite: (content: string) => string,
  reportReadFailure: boolean,
): number {
  const content = readFileToString(inputPath);
  if (content === null) {
    return reportReadFailure ? fail(`Failed to read ${inputPath}\n`) : 1;
  }
  writeStdout(rewrite(content));
  return 0;
}

export async function release(args: readonly string[]): Promise<number> {
  if (args.length === 3 && args[0] === "--print-bump") {
    const version = bumpVersion(args[1], args[2]);
    if (version === null) return 1;
    println(version);
    return 0;
  }
  if (args.length === 4 && args[0] === "--print-readme-version-update") {
    return printRewrite(
      args[1],
      (content) => replaceReadmeVersionReferences(content, args[2], args[3]),
      false,
    );
  }
  if (args.length === 4 && args[0] === "--print-workspace-manifest-update") {
    return printRewrite(
      args[1],
      (content) => updateWorkspaceManifestVersions(content, args[2], args[3]),
      false,
    );
  }
  if (args.length === 4 && args[0] === "--print-workspace-catalog-update") {
    return printRewrite(
      args[1],
      (content) => replaceNativeBinaryWorkspaceCatalog(content, args[2], args[3]),
      true,
    );
  }
  if (args.length === 4 && args[0] === "--print-lockfile-catalog-update") {
    return printRewrite(
      args[1],
      (content) => replaceNativeBinaryLockfileCatalog(content, args[2], args[3]),
      true,
    );
  }
  if (args.length === 1 && args[0] === "--print-extra-package-json-paths") {
    for (const packageJsonPath of await extraReleasePackageJsonPaths()) {
      println(packageJsonPath);
    }
    return 0;
  }
  if (args.length === 0) {
    return fail(
      "Usage: node tools/support/release/release/main.ts <bump> [-y] [--pin] | --resume <PR> [--pin] | --retire <PR> --head <H> --tag <tag> --run <R> --operator-run <entry>\n",
    );
  }

  if (args[0] === "--retire") {
    if (
      !(
        args.length === 10 &&
        args[2] === "--head" &&
        args[4] === "--tag" &&
        args[6] === "--run" &&
        args[8] === "--operator-run"
      )
    ) {
      return fail("Retirement requires explicit original PR/H/tag/R/operator-run identities\n");
    }
    const tagParts = args[5].split(".");
    if (
      !(
        positiveRetirementIdentity(args[1]) &&
        positiveRetirementIdentity(args[7]) &&
        positiveRetirementIdentity(args[9]) &&
        isFullCommitIdentity(args[3]) &&
        tagParts.length === 3 &&
        tagParts[0] === "v0" &&
        positiveRetirementIdentity(tagParts[1]) &&
        tagParts[2] === "0"
      )
    ) {
      return fail("Invalid original retirement identities\n");
    }
    return run("rust-script", [
      RELEASE_DRIVER,
      "retire",
      args[1],
      args[3],
      args[5],
      args[7],
      args[9],
    ]);
  }

  if (args.length === 2 && args[0] === "--resume") {
    return run("rust-script", [RELEASE_DRIVER, "resume", args[1]]);
  }
  if (args.length === 3 && args[0] === "--resume" && args[2] === "--pin") {
    return run("rust-script", [RELEASE_DRIVER, "resume", args[1], "--pin"]);
  }
  const bump = args[0];
  let prepareOnly = false;
  let autoConfirm = false;
  let pin = false;
  let pinnedTarget: string | null = null;
  let targetValueIndex = -1;
  for (let i = 1; i < args.length; i++) {
    if (i === targetValueIndex) continue;
    const arg = args[i];
    if (arg === "--pinned-target") {
      if (!(i + 1 < args.length && pinnedTarget === null)) {
        return fail("Invalid internal pinned preparation target\n");
      }
      pinnedTarget = args[i + 1];
      targetValueIndex = i + 1;
      continue;
    }
    if (arg === "--prepare-only") {
      prepareOnly = true;
      continue;
    }
    if (arg === "-y") {
      autoConfirm = true;
      continue;
    }
    if (arg === "--pin") {
      pin = true;
      continue;
    }
    if (arg === "--force-tag") {
      return fail("Error: --force-tag is not supported; published release tags are immutable.\n");
    }
    return fail("Unknown release arguments\n");
  }
  if (pin && prepareOnly) {
    return fail("Error: --pin requires the release PR protocol.\n");
  }
  if (pinnedTarget !== null && (!prepareOnly || bump !== "minor")) {
    return fail("Pinned target is internal to validated minor preparation\n");
  }

  const cargoToml = readFileToString("Cargo.toml");
  if (cargoToml === null) {
    return fail("Failed to read Cargo.toml\n");
  }
  const currentVersion = workspaceVersionFromContent(cargoToml);
  if (currentVersion === null) {
    return fail("Failed to read current version from Cargo.toml\n");
  }
  const ordinaryVersion = bumpVersion(currentVersion, bump);
  if (ordinaryVersion === null) {
    return fail("Failed to compute the next version\n");
  }
  let newVersion = ordinaryVersion;
  if (pinnedTarget !== null) {
    // Authentication precedes confirmation in this existing internal mode.
    if (
      (await run("rust-script", [
        RELEASE_DRIVER,
        "validate-preparation-target",
        bump,
        currentVersion,
        pinnedTarget,
      ])) !== 0
    ) {
      return fail("Pinned target was not authenticated; no release files changed\n");
    }
    newVersion = pinnedTarget;
  }
  const tag = `v${newVersion}`;

  println(`Current version: ${currentVersion}`);
  if (pin) {
    println(
      "The official pinned driver resolves the next unreserved version from immutable retirement evidence.",
    );
  } else {
    println(`New version: ${newVersion} (tag: ${tag})`);
  }
  println("");

  if (!autoConfirm) {
    if (!(await hasInteractiveTerminal())) {
      return fail(
        "Error: Confirmation requires an interactive terminal. Re-run with -y to skip the prompt.\n",
      );
    }
    if (!(await confirmRelease())) {
      println("Aborted.");
      return 1;
    }
  }

  if (!prepareOnly) {
    return run("rust-script", [RELEASE_DRIVER, "start", bump, ...(pin ? ["--pin"] : [])]);
  }
  return prepareCandidate(cargoToml, currentVersion, newVersion);
}

if (import.meta.main) {
  runMain(() => release(cliArgs()));
}
