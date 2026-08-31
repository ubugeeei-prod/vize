import { Buffer } from "node:buffer";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  mkdtempSync,
  openSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";

const artifactDownloadTimeoutMs = 120_000;
const artifactMaxBytes = 512 * 1024 * 1024;
const artifactMaxEntries = 4_096;
const artifactMaxUncompressedBytes = 512 * 1024 * 1024;

export async function downloadArtifactEntries({
  artifact,
  token,
  fetchImpl = globalThis.fetch,
  limits = {},
}) {
  const url = artifact.archive_download_url;
  if (typeof url !== "string" || url.length === 0) {
    throw new Error(`Real Project Matrix artifact ${String(artifact.name)} has no download URL`);
  }
  const response = await fetchImpl(url, {
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${token}`,
      "x-github-api-version": "2022-11-28",
    },
    signal: AbortSignal.timeout(artifactDownloadTimeoutMs),
  });
  if (!response.ok) {
    throw new Error(
      `Failed to download Real Project Matrix artifact ${String(artifact.name)}: ${response.status} ${response.statusText}`,
    );
  }
  const maxBytes = limits.maxBytes ?? artifactMaxBytes;
  const maxEntries = limits.maxEntries ?? artifactMaxEntries;
  const maxUncompressedBytes = limits.maxUncompressedBytes ?? artifactMaxUncompressedBytes;
  const label = String(artifact.name);
  const scratch = mkdtempSync(join(tmpdir(), "vize-release-matrix-artifact-"));
  try {
    const archive = join(scratch, "artifact.zip");
    const output = join(scratch, "out");
    await writeBoundedArchive({ response, archive, label, maxBytes });
    assertArchiveWithinLimits({ archive, label, maxEntries, maxUncompressedBytes });
    const unzip = spawnSync("unzip", ["-q", archive, "-d", output], {
      encoding: "utf8",
      timeout: 30_000,
    });
    if (unzip.error != null || unzip.status !== 0) {
      const detail = [unzip.stdout, unzip.stderr].filter(Boolean).join("\n").trim();
      throw new Error(
        `Failed to unpack Real Project Matrix artifact ${label}${detail === "" ? "" : `:\n${detail}`}`,
      );
    }
    return collectTextEntries(output, output, new Map(), {
      label,
      maxEntries,
      maxUncompressedBytes,
      totalBytes: 0,
    });
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

export function exactMatchingEntries(entries, pattern, label) {
  const matches = entryNames(entries)
    .filter((name) => pattern.test(name))
    .sort((left, right) => left.localeCompare(right))
    .map((name) => ({ name, text: readTextEntry(entries, name, label) }));
  if (matches.length !== 1) {
    throw new Error(`${label} must be present exactly once; found ${matches.length}`);
  }
  return matches;
}

export function readJsonEntry(entries, name, label) {
  return parseJsonText(readTextEntry(entries, name, label), name);
}

export function parseJsonText(text, name) {
  try {
    return JSON.parse(text);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid release evidence JSON ${name}: ${detail}`, { cause: error });
  }
}

export function readTextEntry(entries, name, label) {
  const value =
    entries instanceof Map
      ? entries.get(name)
      : entries != null && typeof entries === "object"
        ? entries[name]
        : undefined;
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} is missing ${name}`);
  }
  return value;
}

export function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function entryNames(entries) {
  if (entries instanceof Map) return [...entries.keys()];
  if (entries != null && typeof entries === "object") return Object.keys(entries);
  throw new Error("Real Project Matrix artifact entries must be a map or object");
}

async function writeBoundedArchive({ response, archive, label, maxBytes }) {
  const handle = openSync(archive, "w");
  let downloaded = 0;
  try {
    for await (const chunk of responseChunks(response, label)) {
      downloaded += chunk.byteLength;
      if (downloaded > maxBytes) {
        throw new Error(`Real Project Matrix artifact ${label} exceeds ${maxBytes} bytes`);
      }
      writeSync(handle, chunk);
    }
  } finally {
    closeSync(handle);
  }
  if (downloaded === 0) {
    throw new Error(`Real Project Matrix artifact ${label} downloaded no bytes`);
  }
}

async function* responseChunks(response, label) {
  const body = response.body;
  if (body != null && typeof body[Symbol.asyncIterator] === "function") {
    for await (const chunk of body) yield Buffer.from(chunk);
    return;
  }
  if (body != null && typeof body.getReader === "function") {
    const reader = body.getReader();
    for (;;) {
      const { done, value } = await reader.read();
      if (done) return;
      if (value != null) yield Buffer.from(value);
    }
  }
  throw new Error(`Real Project Matrix artifact ${label} response is not streamable`);
}

// The archive is untrusted until its central directory proves it stays inside the
// entry-count and uncompressed-size budgets, so read the totals before extraction
// instead of letting `unzip` fill the scratch directory first.
function assertArchiveWithinLimits({ archive, label, maxEntries, maxUncompressedBytes }) {
  const listing = spawnSync("unzip", ["-Z", "-t", archive], {
    encoding: "utf8",
    timeout: 30_000,
  });
  if (listing.error != null || listing.status !== 0) {
    const detail = [listing.stdout, listing.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      `Failed to inspect Real Project Matrix artifact ${label}${detail === "" ? "" : `:\n${detail}`}`,
    );
  }
  const totals = /^\s*(\d+)\s+files?,\s+(\d+)\s+bytes uncompressed/m.exec(listing.stdout ?? "");
  if (totals == null) {
    throw new Error(`Real Project Matrix artifact ${label} has an unreadable archive listing`);
  }
  const entryCount = Number(totals[1]);
  const uncompressedBytes = Number(totals[2]);
  if (entryCount === 0) {
    throw new Error(`Real Project Matrix artifact ${label} is empty`);
  }
  if (entryCount > maxEntries) {
    throw new Error(
      `Real Project Matrix artifact ${label} declares ${entryCount} entries; the limit is ${maxEntries}`,
    );
  }
  if (uncompressedBytes > maxUncompressedBytes) {
    throw new Error(
      `Real Project Matrix artifact ${label} declares ${uncompressedBytes} uncompressed bytes; the limit is ${maxUncompressedBytes}`,
    );
  }
}

function collectTextEntries(root, directory = root, entries = new Map(), budget = null) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolute = join(directory, entry.name);
    if (entry.isDirectory()) {
      collectTextEntries(root, absolute, entries, budget);
    } else if (entry.isFile()) {
      const payload = readFileSync(absolute);
      if (budget != null) {
        budget.totalBytes += payload.byteLength;
        if (entries.size + 1 > budget.maxEntries) {
          throw new Error(
            `Real Project Matrix artifact ${budget.label} extracted more than ${budget.maxEntries} entries`,
          );
        }
        if (budget.totalBytes > budget.maxUncompressedBytes) {
          throw new Error(
            `Real Project Matrix artifact ${budget.label} extracted more than ${budget.maxUncompressedBytes} bytes`,
          );
        }
      }
      entries.set(relative(root, absolute).replaceAll("\\", "/"), payload.toString("utf8"));
    } else {
      throw new Error(`Real Project Matrix artifact contains unsupported entry: ${entry.name}`);
    }
  }
  if (directory === root && entries.size === 0 && statSync(root).isDirectory()) {
    throw new Error("Real Project Matrix artifact is empty");
  }
  return entries;
}
