import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { readTar } from "../../../archive/tar.ts";
import { boundedBytes } from "./http.ts";
import type { OwnedCrateSourceProof } from "./types.ts";

function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
/** Verify actual public package bytes and Cargo's committed source identity. Never extract. */
export async function readOwnedCrateSource(
  crate: string,
  version: string,
  checksum: string,
  candidate: string,
  fetchImpl: typeof fetch = globalThis.fetch,
): Promise<OwnedCrateSourceProof> {
  requireValue(
    /^[a-zA-Z0-9_-]+$/.test(crate) && /^\d+\.\d+\.\d+$/.test(version),
    "crate archive identity required",
  );
  requireValue(
    /^[a-f0-9]{64}$/.test(checksum) && /^[a-f0-9]{40}$/.test(candidate),
    "expected crate checksum and candidate required",
  );
  const url = `https://static.crates.io/crates/${crate}/${crate}-${version}.crate`;
  const response = await fetchImpl(url, { redirect: "error", signal: AbortSignal.timeout(15_000) });
  requireValue(response.status === 200, `owned crate archive HTTP ${response.status}`);
  const archive = await boundedBytes(response, 32 * 1024 * 1024);
  const archiveSha256 = createHash("sha256").update(archive).digest("hex");
  requireValue(
    archiveSha256 === checksum,
    "owned crate archive checksum differs from registry proof",
  );
  const tar = gunzipSync(archive, { maxOutputLength: 128 * 1024 * 1024 });
  const members = readTar(tar),
    expected = crate + "-" + version + "/.cargo_vcs_info.json";
  const matches = members.filter((member) => member.name === expected);
  requireValue(matches.length === 1, "owned crate must contain one Cargo VCS proof");
  const member = matches[0];
  requireValue(
    member !== undefined && member.kind === "file" && member.data.length <= 16 * 1024,
    "bounded regular Cargo VCS proof required",
  );
  const raw = member.data,
    value: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw));
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "Cargo VCS object required",
  );
  const git = (value as Record<string, unknown>).git;
  requireValue(
    git !== null &&
      typeof git === "object" &&
      !Array.isArray(git) &&
      (git as Record<string, unknown>).sha1 === candidate,
    "owned crate Cargo VCS source is not the intended produced candidate",
  );
  return {
    candidateSha: candidate,
    archiveUrl: url,
    archiveBytes: archive.length,
    archiveSha256,
    vcsPath: expected,
    vcsBytes: raw.length,
    vcsSha256: createHash("sha256").update(raw).digest("hex"),
    vcsRawBase64: raw.toString("base64"),
  };
}
