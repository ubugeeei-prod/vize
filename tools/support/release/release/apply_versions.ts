import { fileURLToPath } from "node:url";
import { collectStdout, println, run, writeStderr } from "../command_io.ts";
import {
  isDir,
  joinPath,
  moonTrim,
  pathExists,
  readDir,
  readFileToString,
  replaceFirst,
  tryWriteStringToFile,
} from "../command_runtime.ts";
import {
  verifyManifestAlignmentAfterCommit,
  verifyManifestAlignmentBeforeMutation,
} from "./manifest_alignment.ts";
import {
  replaceNativeBinaryLockfileCatalog,
  replaceNativeBinaryWorkspaceCatalog,
} from "./native_catalog.ts";
import {
  extraReleasePackageJsonPaths,
  replaceFirstVersionLine,
  replacePackageJsonVersion,
  runLocalReleaseGuard,
  stageFiles,
} from "./preparation.ts";
import { updateReleaseReadmes } from "./readmes.ts";
import { updateWorkspaceManifestVersions } from "./versions.ts";

/**
 * These standalone workspaces depend on the in-tree SDK and build with
 * --locked. Updating the root workspace lockfile cannot refresh them.
 */
export const standaloneGuestManifests: readonly string[] = [
  "davinci/vize_extension_host/tests/guests/expression-echo/Cargo.toml",
  "davinci/vize_extension_host/tests/guests/output-echo/Cargo.toml",
  "davinci/vize_extension_host/tests/guests/typed-expression-echo/Cargo.toml",
  "examples/volt-target/Cargo.toml",
];

/** Rewrites one existing file, reporting the original read or write diagnostic. */
function rewriteFile(target: string, transform: (content: string) => string): boolean {
  const content = readFileToString(target);
  if (content === null) {
    writeStderr(`Failed to read ${target}\n`);
    return false;
  }
  if (!tryWriteStringToFile(target, transform(content))) {
    writeStderr(`Failed to update ${target}\n`);
    return false;
  }
  return true;
}

/** Mutate and commit only the isolated candidate prepared from current main. */
export async function prepareCandidate(
  cargoToml: string,
  currentVersion: string,
  newVersion: string,
): Promise<number> {
  const cargoTomlPath = "Cargo.toml";
  const tag = `v${newVersion}`;
  if (!(await runLocalReleaseGuard(tag))) return 1;
  if (!(await verifyManifestAlignmentBeforeMutation())) return 1;

  println("Updating Cargo.toml...");
  if (
    !tryWriteStringToFile(
      cargoTomlPath,
      updateWorkspaceManifestVersions(cargoToml, currentVersion, newVersion),
    )
  ) {
    writeStderr("Failed to update Cargo.toml\n");
    return 1;
  }

  println("Updating Cargo.lock...");
  if (
    (await run("rust-script", [
      fileURLToPath(new URL("../../../commands/release/pr.rs", import.meta.url)),
      "rewrite-workspace-lock",
      currentVersion,
      newVersion,
    ])) !== 0
  ) {
    writeStderr("Failed to update Cargo.lock\n");
    return 1;
  }

  const filesToStage = ["Cargo.toml", "Cargo.lock"];
  println("Updating npm packages...");
  for (const entryName of readDir("npm")) {
    const packageDir = joinPath("npm", entryName);
    if (!isDir(packageDir)) continue;
    const packageJsonPath = joinPath(packageDir, "package.json");
    if (!pathExists(packageJsonPath)) continue;
    if (
      !rewriteFile(packageJsonPath, (content) =>
        replacePackageJsonVersion(content, currentVersion, newVersion),
      )
    ) {
      return 1;
    }
    println(`  Updated ${packageJsonPath}`);
    filesToStage.push(packageJsonPath);
  }
  for (const packageJsonPath of await extraReleasePackageJsonPaths()) {
    if (!pathExists(packageJsonPath)) continue;
    if (
      !rewriteFile(packageJsonPath, (content) =>
        replacePackageJsonVersion(content, currentVersion, newVersion),
      )
    ) {
      return 1;
    }
    println(`  Updated ${packageJsonPath}`);
    filesToStage.push(packageJsonPath);
  }

  const workspaceYamlPath = "pnpm-workspace.yaml";
  const workspaceYaml = readFileToString(workspaceYamlPath);
  if (workspaceYaml === null) {
    writeStderr(`Failed to read ${workspaceYamlPath}\n`);
    return 1;
  }
  const updatedWorkspaceYaml = replaceNativeBinaryWorkspaceCatalog(
    workspaceYaml,
    currentVersion,
    newVersion,
  );
  if (updatedWorkspaceYaml !== workspaceYaml) {
    if (!tryWriteStringToFile(workspaceYamlPath, updatedWorkspaceYaml)) {
      writeStderr(`Failed to update ${workspaceYamlPath}\n`);
      return 1;
    }
    println(`  Updated ${workspaceYamlPath}`);
    filesToStage.push(workspaceYamlPath);
  }

  const lockfilePath = "pnpm-lock.yaml";
  const lockfile = readFileToString(lockfilePath);
  if (lockfile === null) {
    writeStderr(`Failed to read ${lockfilePath}\n`);
    return 1;
  }
  const updatedLockfile = replaceNativeBinaryLockfileCatalog(lockfile, currentVersion, newVersion);
  if (updatedLockfile !== lockfile) {
    if (!tryWriteStringToFile(lockfilePath, updatedLockfile)) {
      writeStderr(`Failed to update ${lockfilePath}\n`);
      return 1;
    }
    println(`  Updated ${lockfilePath} native-binaries catalog block`);
    filesToStage.push(lockfilePath);
  }

  const zedCargoToml = "editors/zed/Cargo.toml";
  const zedExtensionToml = "editors/zed/extension.toml";
  const zedCargoLock = "editors/zed/Cargo.lock";
  for (const zedManifest of [zedCargoToml, zedExtensionToml]) {
    if (!pathExists(zedManifest)) continue;
    if (
      !rewriteFile(zedManifest, (content) =>
        replaceFirstVersionLine(content, currentVersion, newVersion),
      )
    ) {
      return 1;
    }
    println(`  Updated ${zedManifest}`);
    filesToStage.push(zedManifest);
  }
  if (pathExists(zedCargoLock)) {
    if (
      (await run("cargo", [
        "update",
        "--manifest-path",
        "editors/zed/Cargo.toml",
        "-p",
        "vize-zed-extension",
      ])) !== 0
    ) {
      writeStderr("Failed to update editors/zed/Cargo.lock\n");
      return 1;
    }
    println(`  Updated ${zedCargoLock}`);
    filesToStage.push(zedCargoLock);
  }

  for (const manifest of standaloneGuestManifests) {
    if (!pathExists(manifest)) continue;
    const guestLockfile = replaceFirst(manifest, "Cargo.toml", "Cargo.lock");
    // Reuse the protected integration's exact version transformation. Cargo's
    // offline resolver can downgrade unrelated packages absent from its cache.
    if (
      (await run("rust-script", [
        "tools/commands/release/pr.rs",
        "rewrite-guest-lock",
        guestLockfile,
        currentVersion,
        newVersion,
      ])) !== 0
    ) {
      writeStderr(`Failed to update the SDK guest lockfile for ${manifest}\n`);
      return 1;
    }
    println(`  Updated ${guestLockfile}`);
    filesToStage.push(guestLockfile);
  }

  if (!(await updateReleaseReadmes(currentVersion, newVersion, filesToStage))) {
    return 1;
  }

  println("Committing changes...");
  if ((await run("git", ["add", ...stageFiles(filesToStage)])) !== 0) {
    writeStderr("Failed to stage release files\n");
    return 1;
  }

  const staged = await collectStdout("git", ["diff", "--cached", "--name-only"]);
  if (staged.status !== 0) {
    writeStderr("Failed to inspect staged release files\n");
    return 1;
  }
  if (moonTrim(staged.stdout) === "") {
    writeStderr("No release changes were staged; refusing to create an empty release.\n");
    return 1;
  }
  if ((await run("git", ["commit", "--no-verify", "-m", `chore: release ${tag}`])) !== 0) {
    writeStderr("Failed to create the release commit\n");
    return 1;
  }

  if (!(await verifyManifestAlignmentAfterCommit())) return 1;

  println("Release candidate commit prepared; no tag was created.");
  return 0;
}
