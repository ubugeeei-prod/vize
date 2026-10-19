import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { execFile } from "child_process";
import { promisify } from "util";
import { workspace, type ExtensionContext } from "vscode";
import { parseVizeVersion, serverMatchesExtension } from "./extension-core.js";
import { downloadFile } from "./release-download.js";
import { extractReleaseArchive } from "./release-archive.js";
import { trustedServerPath, workspaceDevServerPaths } from "./server-trust.js";
import type {
  ExtensionRuntimeState,
  InspectedServerCandidate,
  ReleaseServerTarget,
  ServerCandidate,
  VizeStatus,
} from "./server-state-types.js";

const execFileAsync = promisify(execFile);

export function getExtensionVersion(context: ExtensionContext): string | undefined {
  const packageJson = context.extension.packageJSON as { version?: unknown };
  return typeof packageJson.version === "string" ? packageJson.version : undefined;
}

/** Candidate inspection and verified release installation retain generation admission. */
export function createServerDiscovery(
  state: ExtensionRuntimeState,
  updateStatusBar: (status: VizeStatus, detail: string) => void,
  warnAboutVersionMismatch: (
    candidate: InspectedServerCandidate,
    version?: string,
  ) => Promise<void>,
) {
  async function findServerPath(
    context: ExtensionContext,
    config: ReturnType<typeof workspace.getConfiguration>,
    generation: number,
  ): Promise<string | undefined> {
    const exeName = process.platform === "win32" ? "vize.exe" : "vize";
    const expectedVersion = getExtensionVersion(context);
    state.selectedServerCandidate = undefined;

    const configuredPath = resolveTrustedServerPath(config);
    if (configuredPath) {
      if (fs.existsSync(configuredPath)) {
        const candidate = await inspectServerCandidate({
          path: configuredPath,
          source: "configured",
        });
        if (generation !== state.configurationSyncGeneration) return undefined;
        logSelectedServer(candidate, expectedVersion);
        void warnAboutVersionMismatch(candidate, expectedVersion);
        state.selectedServerCandidate = candidate;
        return configuredPath;
      }

      state.outputChannel.appendLine(`Configured server path does not exist: ${configuredPath}`);
    }

    const candidates = await inspectServerCandidates(collectServerCandidates(context, exeName));
    if (generation !== state.configurationSyncGeneration) return undefined;

    if (expectedVersion) {
      const matchingCandidate = candidates.find(
        (candidate) =>
          candidate.version !== undefined &&
          serverMatchesExtension(candidate.version, expectedVersion),
      );
      if (matchingCandidate) {
        logSelectedServer(matchingCandidate, expectedVersion);
        state.selectedServerCandidate = matchingCandidate;
        return matchingCandidate.path;
      }

      const releaseCandidate = await resolveReleaseServerCandidate(
        context,
        exeName,
        expectedVersion,
        generation,
      );
      if (generation !== state.configurationSyncGeneration) return undefined;
      if (releaseCandidate) {
        logSelectedServer(releaseCandidate, expectedVersion);
        state.selectedServerCandidate = releaseCandidate;
        return releaseCandidate.path;
      }
    }

    const fallbackCandidate = candidates[0];
    if (fallbackCandidate) {
      logSelectedServer(fallbackCandidate, expectedVersion);
      void warnAboutVersionMismatch(fallbackCandidate, expectedVersion);
      state.selectedServerCandidate = fallbackCandidate;
      return fallbackCandidate.path;
    }

    state.outputChannel.appendLine("Server not found in any location");
    return undefined;
  }

  function collectServerCandidates(context: ExtensionContext, exeName: string): ServerCandidate[] {
    const candidates: ServerCandidate[] = [];

    const bundledPaths = [
      path.join(context.extensionPath, "dist", exeName),
      path.join(context.extensionPath, "server", exeName),
    ];
    for (const serverPath of bundledPaths) {
      candidates.push({ path: serverPath, source: "bundled" });
    }

    const devPaths = [
      path.join(context.extensionPath, "..", "..", "target", "release", exeName),
      path.join(context.extensionPath, "..", "..", "target", "debug", exeName),
      ...getWorkspaceDevPaths(exeName),
    ];
    for (const serverPath of devPaths) {
      candidates.push({ path: serverPath, source: "development" });
    }

    const homeDir = process.env.HOME || process.env.USERPROFILE || "";
    if (homeDir) {
      candidates.push({
        path: path.join(homeDir, ".cargo", "bin", exeName),
        source: "cargo",
      });
    }

    const pathEnv = process.env.PATH || "";
    const pathSeparator = process.platform === "win32" ? ";" : ":";
    const pathDirs = [...pathEnv.split(pathSeparator), ...getImplicitPathDirs()];
    for (const dir of pathDirs) {
      if (dir) {
        candidates.push({ path: path.join(dir, exeName), source: "path" });
      }
    }

    return dedupeCandidates(candidates).filter((candidate) => fs.existsSync(candidate.path));
  }

  function dedupeCandidates(candidates: ServerCandidate[]): ServerCandidate[] {
    const seen = new Set<string>();
    const deduped: ServerCandidate[] = [];

    for (const candidate of candidates) {
      const key = path.resolve(candidate.path);
      if (seen.has(key)) {
        continue;
      }

      seen.add(key);
      deduped.push(candidate);
    }

    return deduped;
  }

  async function inspectServerCandidates(
    candidates: ServerCandidate[],
  ): Promise<InspectedServerCandidate[]> {
    return Promise.all(candidates.map(inspectServerCandidate));
  }

  async function inspectServerCandidate(
    candidate: ServerCandidate,
  ): Promise<InspectedServerCandidate> {
    try {
      const { stdout } = await execFileAsync(candidate.path, ["--version"], {
        timeout: 3000,
      });
      return {
        ...candidate,
        version: parseVizeVersion(stdout),
      };
    } catch (error) {
      return {
        ...candidate,
        versionError: String(error),
      };
    }
  }

  function logSelectedServer(
    candidate: InspectedServerCandidate,
    expectedVersion: string | undefined,
  ): void {
    const version = candidate.version ?? "unknown";
    const expected = expectedVersion ? `, extension ${expectedVersion}` : "";
    state.outputChannel.appendLine(
      `Using ${candidate.source} server: ${candidate.path} (server ${version}${expected})`,
    );

    if (candidate.versionError) {
      state.outputChannel.appendLine(`Could not inspect server version: ${candidate.versionError}`);
    }
  }

  function getWorkspaceDevPaths(exeName: string): string[] {
    const folders = (workspace.workspaceFolders ?? []).map((folder) => folder.uri.fsPath);
    return workspaceDevServerPaths(folders, exeName, workspace.isTrusted);
  }

  function resolveTrustedServerPath(
    config: ReturnType<typeof workspace.getConfiguration>,
  ): string | undefined {
    return trustedServerPath(config.inspect<string>("serverPath"), workspace.isTrusted);
  }

  function getImplicitPathDirs(): string[] {
    const dirs: string[] = [];
    const homeDir = process.env.HOME || process.env.USERPROFILE || "";
    if (homeDir) {
      dirs.push(path.join(homeDir, ".local", "bin"), path.join(homeDir, ".nix-profile", "bin"));
    }

    if (process.platform !== "win32") {
      dirs.push(
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/nix/var/nix/profiles/default/bin",
        "/run/current-system/sw/bin",
      );
    }

    return dirs;
  }

  async function resolveReleaseServerCandidate(
    context: ExtensionContext,
    exeName: string,
    expectedVersion: string,
    generation: number,
  ): Promise<InspectedServerCandidate | undefined> {
    const target = getReleaseServerTarget();
    if (!target) {
      state.outputChannel.appendLine(
        `No Vize release binary is available for ${process.platform}/${process.arch}.`,
      );
      return undefined;
    }

    const installDir = path.join(
      context.globalStorageUri.fsPath,
      "servers",
      expectedVersion,
      target.targetName,
    );
    const serverPath = path.join(installDir, exeName);
    const cachedCandidate = await inspectCachedReleaseServer(serverPath, expectedVersion);
    if (generation !== state.configurationSyncGeneration) return undefined;
    if (cachedCandidate) {
      return cachedCandidate;
    }

    const archivePath = path.join(installDir, target.archiveName);
    const downloadUrl = createReleaseDownloadUrl(expectedVersion, target.archiveName);
    const checksumUrl = `${downloadUrl}.sha256`;
    const checksumPath = `${archivePath}.sha256`;

    try {
      updateStatusBar("starting", `Downloading Vize language server ${expectedVersion}`);
      state.outputChannel.appendLine(
        `Downloading Vize language server ${expectedVersion}: ${downloadUrl}`,
      );
      fs.rmSync(installDir, { force: true, recursive: true });
      await fs.promises.mkdir(installDir, { recursive: true });
      if (generation !== state.configurationSyncGeneration) return undefined;
      await downloadFile(downloadUrl, archivePath);
      if (generation !== state.configurationSyncGeneration) return undefined;

      // Verify SHA-256 sidecars before executing downloaded servers.
      // Missing, malformed, or mismatched checksums fail closed. (#969)
      await downloadFile(checksumUrl, checksumPath);
      if (generation !== state.configurationSyncGeneration) return undefined;
      await verifyArchiveChecksum(archivePath, checksumPath);
      if (generation !== state.configurationSyncGeneration) return undefined;

      await extractReleaseArchive(archivePath, installDir, exeName);
      if (generation !== state.configurationSyncGeneration) return undefined;

      const candidate = await inspectServerCandidate({
        path: serverPath,
        source: "release",
      });
      if (candidate.version === expectedVersion) {
        return candidate;
      }

      state.outputChannel.appendLine(
        `Downloaded Vize language server version ${candidate.version ?? "unknown"} does not match extension ${expectedVersion}.`,
      );
    } catch (error) {
      state.outputChannel.appendLine(`Failed to download Vize language server: ${String(error)}`);
    }

    return undefined;
  }

  async function inspectCachedReleaseServer(
    serverPath: string,
    expectedVersion: string,
  ): Promise<InspectedServerCandidate | undefined> {
    if (!fs.existsSync(serverPath)) {
      return undefined;
    }

    const candidate = await inspectServerCandidate({
      path: serverPath,
      source: "release",
    });
    if (candidate.version === expectedVersion) {
      return candidate;
    }

    state.outputChannel.appendLine(
      `Cached Vize language server ${candidate.version ?? "unknown"} does not match extension ${expectedVersion}. Refreshing cache.`,
    );
    return undefined;
  }

  function getReleaseServerTarget(): ReleaseServerTarget | undefined {
    if (process.platform === "darwin") {
      if (process.arch === "arm64") {
        return releaseTarget("aarch64-apple-darwin", "tar.gz");
      }
      if (process.arch === "x64") {
        return releaseTarget("x86_64-apple-darwin", "tar.gz");
      }
    }

    if (process.platform === "linux") {
      if (process.arch === "arm64") {
        return releaseTarget("aarch64-unknown-linux-gnu", "tar.gz");
      }
      if (process.arch === "x64") {
        return releaseTarget("x86_64-unknown-linux-gnu", "tar.gz");
      }
    }

    if (process.platform === "win32") {
      if (process.arch === "arm64") {
        return releaseTarget("aarch64-pc-windows-msvc", "zip");
      }
      if (process.arch === "x64") {
        return releaseTarget("x86_64-pc-windows-msvc", "zip");
      }
    }

    return undefined;
  }

  function releaseTarget(targetName: string, extension: "tar.gz" | "zip"): ReleaseServerTarget {
    return {
      archiveName: `vize-${targetName}.${extension}`,
      targetName,
    };
  }

  function createReleaseDownloadUrl(version: string, archiveName: string): string {
    return `https://github.com/ubugeeei-prod/vize/releases/download/v${version}/${archiveName}`;
  }

  /// Verify that `archivePath` matches the declared SHA-256 in `checksumPath`.
  /// Accepts bare hex or GNU sha256sum shape; malformed or mismatched files throw. (#969)
  async function verifyArchiveChecksum(archivePath: string, checksumPath: string): Promise<void> {
    const declared = (await fs.promises.readFile(checksumPath, "utf8"))
      .trim()
      .split(/\s+/)[0]
      ?.toLowerCase();
    if (!declared || !/^[0-9a-f]{64}$/.test(declared)) {
      throw new Error(`malformed SHA-256 checksum file at ${checksumPath}`);
    }
    const hash = crypto.createHash("sha256");
    await new Promise<void>((resolve, reject) => {
      const stream = fs.createReadStream(archivePath);
      stream.on("data", (chunk: string | Buffer) => {
        hash.update(chunk);
      });
      stream.on("end", () => resolve());
      stream.on("error", reject);
    });
    const got = hash.digest("hex");
    if (got !== declared) {
      throw new Error(`SHA-256 mismatch for ${archivePath}: declared ${declared}, computed ${got}`);
    }
    state.outputChannel.appendLine(`Verified ${path.basename(archivePath)} (SHA-256 ${got}).`);
  }
  return { findServerPath };
}
