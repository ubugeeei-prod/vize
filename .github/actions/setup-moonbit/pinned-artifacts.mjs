import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";

export const pinnedVersion = "0.10.7+bc794d341";
export const coreSha256 = "06922d35dd94dc0baea7682295fe6084f0d854eed3b497cd6b527f77d899a7f5";
const release =
  "https://github.com/chawyehsu/moonbit-binaries/releases/download/v0.10.7%2Bbc794d341/";
const archives = {
  "linux-x64": [
    "x86_64-unknown-linux.tar.gz",
    94304160,
    "36f5e7cf1545594e17cd3f1c0b757fe6e86ad0218bc96f419369cbb8502e62ba",
  ],
  "linux-arm64": [
    "aarch64-unknown-linux.tar.gz",
    106966613,
    "e191a88a664cf4943736846db89108ac76a3a592b4427a56622180c12a18adc5",
  ],
  "darwin-arm64": [
    "aarch64-apple-darwin.tar.gz",
    83616535,
    "b4781a1e38c800d1fd65693b1970b2d2429faef31d8933d266a1f6e2693a96ef",
  ],
  "win32-x64": [
    "x86_64-pc-windows.zip",
    84486115,
    "c659625f5c3a9fca5d17866bff3de07b6328c59628d74df5d5b4d79b78524880",
  ],
};

export function selectArchive(version, platform, arch) {
  if (version !== pinnedVersion) throw new Error(`Unreviewed MoonBit artifact version: ${version}`);
  // The pinned official Windows installer explicitly supports ARM64 by using
  // this same x86_64 archive; preserve its emulated toolchain selection.
  const selected =
    archives[`${platform}-${platform === "win32" && arch === "arm64" ? "x64" : arch}`];
  if (!selected) throw new Error(`Unsupported pinned MoonBit platform: ${platform}-${arch}`);
  const [name, bytes, sha256] = selected;
  return { url: `${release}moonbit-v0.10.7%2Bbc794d341-${name}`, bytes, sha256 };
}

export function verifyBytes(body, sha256, bytes) {
  const actual = createHash("sha256").update(body).digest("hex");
  if (actual !== sha256 || body.length !== bytes)
    throw new Error("MoonBit artifact hash/size mismatch");
}

function replaceOnce(source, before, after) {
  if (source.split(before).length !== 2)
    throw new Error("MoonBit installer transport boundary changed");
  return source.replace(before, after);
}

// The caller has already authenticated the complete official installer. Only
// these fixed transport expressions change; version, bundling and errors stay.
export function adaptInstaller(source, platform, archive, core) {
  if (platform !== "win32") {
    const uri = (file) => pathToFileURL(file).href.replaceAll("'", "%27");
    source = replaceOnce(
      source,
      'moonbit_uri="$CLI_MOONBIT/binaries/$version/moonbit-$target.tar.gz"',
      `moonbit_uri='${uri(archive)}'`,
    );
    return replaceOnce(
      source,
      'core_uri="$CLI_MOONBIT/cores/core-$version.tar.gz"',
      `core_uri='${uri(core)}'`,
    );
  }
  const literal = (value) => `'${value.replaceAll("'", "''")}'`;
  source = replaceOnce(
    source,
    '  Invoke-WebRequest -Uri $MoonbitUri -OutFile "${HOME}\\moonbit.zip"',
    `  Copy-Item -LiteralPath ${literal(archive)} -Destination "\${HOME}\\moonbit.zip"`,
  );
  return replaceOnce(
    source,
    "  Invoke-WebRequest -Uri $CoreUri -OutFile $MoonHome\\core.zip\n  Expand-Archive $MoonHome\\core.zip -DestinationPath $MoonLib -Force\n  Remove-Item -Force $MoonHome\\core.zip",
    `  New-Item -Path $MoonLib -ItemType Directory -Force | Out-Null\n  tar -xf ${literal(core)} --directory $MoonLib\n  if ($LASTEXITCODE -ne 0) { throw "Failed to extract authenticated MoonBit core" }`,
  );
}

async function downloadArchive(artifact, destination) {
  const response = await fetch(artifact.url);
  if (!response.ok || !response.body)
    throw new Error(`MoonBit archive download failed: HTTP ${response.status}`);
  const file = fs.openSync(destination, "wx");
  let size = 0;
  try {
    for await (const chunk of response.body) {
      size += chunk.length;
      if (size > artifact.bytes) throw new Error("MoonBit archive exceeds pinned size");
      for (let offset = 0; offset < chunk.length;) {
        offset += fs.writeSync(file, chunk, offset, chunk.length - offset);
      }
    }
  } finally {
    fs.closeSync(file);
  }
  verifyBytes(fs.readFileSync(destination), artifact.sha256, artifact.bytes);
}

export async function preparePinnedInstaller(installer, version, runnerTemp) {
  if (process.env.MOONBIT_INSTALL_DEV)
    throw new Error("Unreviewed MoonBit development archive requested");
  const artifact = selectArchive(version, process.platform, process.arch);
  const coreBytes = fs.readFileSync(
    new URL("./assets/core-0.10.7+bc794d341.tar.gz", import.meta.url),
  );
  verifyBytes(coreBytes, coreSha256, 1449345);
  const stage = fs.mkdtempSync(path.join(runnerTemp, "moonbit-pinned-artifacts-"));
  const archive = path.join(
    stage,
    process.platform === "win32" ? "compiler.zip" : "compiler.tar.gz",
  );
  const core = path.join(stage, "core.tar.gz");
  fs.writeFileSync(core, coreBytes, { flag: "wx" });
  await downloadArchive(artifact, archive);
  const original = fs.readFileSync(installer);
  const [sha256, bytes] =
    process.platform === "win32"
      ? ["a5101e91ffa9905fb25cd009b9a4aa942971a294bd055c89836e3af89b710c64", 3218]
      : ["46495f8cdc0050f79b6cb195d66478d101cb3601d68506568fbe377fcdf2a9fe", 6712];
  verifyBytes(original, sha256, bytes);
  const adapted = adaptInstaller(original.toString("utf8"), process.platform, archive, core);
  fs.writeFileSync(installer, adapted);
}
