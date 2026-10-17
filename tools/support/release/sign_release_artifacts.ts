// Sign the existing release asset set without serializing independent cosign calls.
import { spawn, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { isEntrypoint } from "../command-runtime.ts";
import { cliReleasePlatforms, parseReleaseVersion } from "../github/release-platforms.ts";

const CONCURRENCY = 4;
const SUFFIXES = [".cosign.bundle", ".pem", ".sig"];
const EDITORS = ["emacs", "helix", "nvim", "vim", "zed"];

function regularFile(file: string): Buffer {
  const stat = lstatSync(file);
  if (!stat.isFile() || stat.size === 0) throw new Error(`Expected nonempty regular file: ${file}`);
  return readFileSync(file);
}

function digest(file: string): string {
  return createHash("sha256").update(regularFile(file)).digest("hex");
}

function assetPaths(tag: string): string[] {
  parseReleaseVersion(tag);
  if (!tag.startsWith("v")) throw new Error("Expected a v-prefixed release tag");
  return [
    ...cliReleasePlatforms.map(({ archive }) => `cli-artifacts/${archive}`),
    "editor-extension-artifacts/editors/vscode/dist/vize.vsix",
    ...EDITORS.map((editor) => `editor-extension-artifacts/${editor}-vize-extension.tar.gz`),
    `vize-${tag}-cyclonedx.sbom.json`,
    `vize-${tag}-spdx.sbom.json`,
  ];
}

function treeFiles(root: string, directory: string): string[] {
  if (!lstatSync(path.join(root, directory)).isDirectory()) {
    throw new Error(`Expected regular directory: ${directory}`);
  }
  return readdirSync(path.join(root, directory)).flatMap((name) => {
    const file = `${directory}/${name}`;
    const stat = lstatSync(path.join(root, file));
    if (stat.isDirectory()) return treeFiles(root, file);
    if (!stat.isFile()) throw new Error(`Refusing non-regular release entry: ${file}`);
    return [file];
  });
}

function assertClosure(root: string, inputs: string[], signed: boolean): void {
  const checksums = inputs
    .filter((file) => !file.endsWith(".sbom.json"))
    .map((file) => `${file}.sha256`);
  const expected = [
    ...inputs,
    ...checksums,
    ...(signed ? inputs.flatMap((file) => SUFFIXES.map((suffix) => file + suffix)) : []),
  ].sort((left, right) => (left < right ? -1 : Number(left > right)));
  const sboms = readdirSync(root).filter((file) =>
    inputs.some(
      (input) => input.endsWith(".sbom.json") && (file === input || file.startsWith(`${input}.`)),
    ),
  );
  const actual = [
    ...treeFiles(root, "cli-artifacts"),
    ...treeFiles(root, "editor-extension-artifacts"),
    ...sboms,
  ].sort((left, right) => (left < right ? -1 : Number(left > right)));
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `Release asset/output closure differs: ${JSON.stringify({ expected, actual })}`,
    );
  }
  for (const file of expected) regularFile(path.join(root, file));
}

function signalGroup(child: ChildProcess): void {
  if (child.pid === undefined) return;
  try {
    process.kill(-child.pid, "SIGKILL");
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "ESRCH")) throw error;
  }
}

function groupExists(pid: number): boolean {
  try {
    process.kill(-pid, 0);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ESRCH") return false;
    throw error;
  }
}

/** Same Linux release job, same cosign arguments and identity; only scheduling changes. */
export async function signReleaseArtifacts(
  root: string,
  tag: string,
  signal?: AbortSignal,
): Promise<void> {
  if (process.platform === "win32")
    throw new Error("Release signing requires POSIX process groups");
  const inputs = assetPaths(tag);
  assertClosure(root, inputs, false);
  const before = new Map(inputs.map((file) => [file, digest(path.join(root, file))]));
  for (const [file, hash] of before) {
    if (file.endsWith(".sbom.json")) continue;
    const expected = `${hash}  ${path.basename(file)}\n`;
    if (regularFile(path.join(root, `${file}.sha256`)).toString() !== expected) {
      throw new Error(`Release checksum does not match input: ${file}`);
    }
  }
  const active = new Set<ChildProcess>();
  let failure: Error | undefined;
  function stop(error: Error): void {
    failure ??= error;
    for (const child of active) {
      try {
        signalGroup(child);
      } catch (killError) {
        // Keep the original refusal; the writer barrier still requires
        // positive disappearance, so a failed kill never permits return.
        failure ??= new Error(`Failed to stop signer group: ${String(killError)}`);
      }
    }
  }
  const abort = () => stop(new Error("Release signing aborted"));
  signal?.addEventListener("abort", abort, { once: true });
  let next = 0;
  try {
    if (signal?.aborted) abort();
    async function sign(file: string): Promise<void> {
      if (failure) return;
      process.stdout.write(`Signing ${file}\n`);
      const child = spawn(
        "cosign",
        [
          "sign-blob",
          "--yes",
          "--bundle",
          `${file}.cosign.bundle`,
          "--output-certificate",
          `${file}.pem`,
          "--output-signature",
          `${file}.sig`,
          file,
        ],
        { cwd: root, detached: true, shell: false, stdio: "inherit" },
      );
      active.add(child);
      await new Promise<void>((resolve) => {
        child.once("error", (error) => stop(error));
        child.once("exit", (code, childSignal) => {
          if (code !== 0)
            stop(new Error(`Signer failed for ${file}: ${String(code ?? childSignal)}`));
          try {
            if (child.pid !== undefined && groupExists(child.pid))
              stop(new Error(`Signer left descendants for ${file}`));
          } catch (error) {
            stop(new Error(`Failed to inspect signer group: ${String(error)}`));
          }
        });
        child.once("close", () => resolve());
      });
      // Reap all leaders, kill any descendant groups, then establish the writer barrier.
      if (child.pid !== undefined) {
        for (;;) {
          try {
            signalGroup(child);
            if (!groupExists(child.pid)) break;
          } catch (error) {
            stop(error instanceof Error ? error : new Error(String(error)));
          }
          // Never return, including failure/abort, while an owned group could
          // write. The unchanged workflow timeout bounds unexpected OS failures.
          await new Promise((resolve) => setTimeout(resolve, 10));
        }
      }
      active.delete(child);
    }
    await Promise.all(
      Array.from({ length: CONCURRENCY }, async () => {
        try {
          while (!failure && next < inputs.length) await sign(inputs[next++]);
        } catch (error) {
          stop(error instanceof Error ? error : new Error(String(error)));
        }
      }),
    );
  } finally {
    signal?.removeEventListener("abort", abort);
  }
  if (failure) throw failure;
  if (active.size !== 0) throw new Error("Signer child remains before output verification");
  assertClosure(root, inputs, true);
  for (const [file, hash] of before) {
    if (digest(path.join(root, file)) !== hash)
      throw new Error(`Release input changed during signing: ${file}`);
    if (
      !file.endsWith(".sbom.json") &&
      regularFile(path.join(root, `${file}.sha256`)).toString() !==
        `${hash}  ${path.basename(file)}\n`
    ) {
      throw new Error(`Release checksum changed during signing: ${file}`);
    }
  }
  process.stdout.write(
    `${JSON.stringify({ schema: "vize-release-signing-v1", concurrency: CONCURRENCY, inputs: Object.fromEntries(before), outputs: Object.fromEntries(inputs.flatMap((file) => SUFFIXES.map((suffix) => [file + suffix, digest(path.join(root, file + suffix))]))) })}\n`,
  );
}

if (isEntrypoint(import.meta.url)) {
  const controller = new AbortController();
  let interrupted = 0;
  const interrupt = (signal: NodeJS.Signals) => {
    interrupted = signal === "SIGINT" ? 130 : 143;
    controller.abort();
  };
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", interrupt);
  try {
    if (process.argv.length !== 3)
      throw new Error("Usage: node tools/support/release/sign_release_artifacts.ts <tag>");
    await signReleaseArtifacts(process.cwd(), process.argv[2], controller.signal);
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = interrupted || 1;
  } finally {
    process.removeListener("SIGINT", interrupt);
    process.removeListener("SIGTERM", interrupt);
  }
}
