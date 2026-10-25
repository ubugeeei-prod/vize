/** An opt-in producer receipt for the exact image built by build-napi-package. */
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { collectStdout, type CommandEnvironment } from "../command-runtime.ts";

const platformTags: Readonly<Record<string, string>> = {
  "aarch64-apple-darwin": "darwin-arm64",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-pc-windows-msvc": "win32-arm64-msvc",
  "x86_64-pc-windows-msvc": "win32-x64-msvc",
  "aarch64-unknown-linux-gnu": "linux-arm64-gnu",
  "x86_64-unknown-linux-gnu": "linux-x64-gnu",
  "aarch64-unknown-linux-musl": "linux-arm64-musl",
  "x86_64-unknown-linux-musl": "linux-x64-musl",
};

interface ReceiptContext {
  readonly output: string;
  readonly source: string;
  readonly releaseVersion: string;
}

/** Capture the actual checkout before the build, never the dispatch's github.sha. */
export function beginNativeBuildReceipt(
  packageDir: string,
  target: string,
  env: CommandEnvironment,
): ReceiptContext | undefined {
  const output = env.VIZE_NATIVE_BUILD_RECEIPT;
  if (!output) return undefined;
  // A failed producer must not leave a receipt from an earlier build invocation.
  fs.rmSync(output, { force: true });
  if (!Object.hasOwn(platformTags, target))
    throw new Error(`Unsupported receipt target: ${target}`);
  const manifest = JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8"));
  const releaseVersion = env.VIZE_RELEASE_VERSION;
  if (!releaseVersion || manifest.version !== releaseVersion) {
    throw new Error("Native receipt version must match the release package manifest");
  }
  const head = collectStdout("git", ["rev-parse", "HEAD"], { cwd: packageDir, env });
  const source = head.stdout.trim();
  if (head.exitCode !== 0 || !/^[0-9a-f]{40}$/.test(source)) {
    throw new Error("Native receipt requires the actual full checkout commit");
  }
  return { output, source, releaseVersion };
}

/** Bind the output bytes to the fresh token that this same invocation compiled. */
export function finishNativeBuildReceipt(
  receipt: ReceiptContext | undefined,
  packageDir: string,
  crateName: string,
  target: string,
  env: CommandEnvironment,
): void {
  if (!receipt) return;
  const nativeBuildToken = env.VIZE_NATIVE_BUILD_TOKEN;
  if (!nativeBuildToken || !/^[0-9a-f]{32}$/.test(nativeBuildToken)) {
    throw new Error("Native receipt requires this invocation's fresh build token");
  }
  const nodeFile = `${crateName.replaceAll("_", "-")}.${platformTags[target]}.node`;
  const imagePath = path.join(packageDir, nodeFile);
  if (!fs.lstatSync(imagePath).isFile())
    throw new Error(`Native image must be a regular file: ${nodeFile}`);
  const image = fs.readFileSync(imagePath);
  // Cross-built images cannot be loaded on the build host. The Rust producer
  // embeds this token as the getNativeBuildToken return value in every image.
  if (!image.includes(Buffer.from(nativeBuildToken))) {
    throw new Error("Native image does not contain this invocation's build token");
  }
  const currentHead = collectStdout("git", ["rev-parse", "HEAD"], { cwd: packageDir, env });
  if (currentHead.exitCode !== 0 || currentHead.stdout.trim() !== receipt.source) {
    throw new Error("Native checkout changed during the build");
  }
  fs.mkdirSync(path.dirname(receipt.output), { recursive: true });
  fs.writeFileSync(
    receipt.output,
    `${JSON.stringify(
      {
        schema: "vize.nativeBuildReceipt.v1",
        source: receipt.source,
        releaseVersion: receipt.releaseVersion,
        rustTarget: target,
        nodeFile,
        bytes: image.length,
        sha256: createHash("sha256").update(image).digest("hex"),
        nativeBuildToken,
      },
      null,
      2,
    )}\n`,
  );
}
