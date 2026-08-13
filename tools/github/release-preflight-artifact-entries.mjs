import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";

export async function downloadArtifactEntries({ artifact, token, fetchImpl = globalThis.fetch }) {
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
  });
  if (!response.ok) {
    throw new Error(
      `Failed to download Real Project Matrix artifact ${String(artifact.name)}: ${response.status} ${response.statusText}`,
    );
  }
  const scratch = mkdtempSync(join(tmpdir(), "vize-release-matrix-artifact-"));
  try {
    const archive = join(scratch, "artifact.zip");
    const output = join(scratch, "out");
    writeFileSync(archive, Buffer.from(await response.arrayBuffer()));
    const unzip = spawnSync("unzip", ["-q", archive, "-d", output], {
      encoding: "utf8",
      timeout: 30_000,
    });
    if (unzip.error != null || unzip.status !== 0) {
      const detail = [unzip.stdout, unzip.stderr].filter(Boolean).join("\n").trim();
      throw new Error(
        `Failed to unpack Real Project Matrix artifact ${String(artifact.name)}${detail === "" ? "" : `:\n${detail}`}`,
      );
    }
    return collectTextEntries(output);
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

function collectTextEntries(root, directory = root, entries = new Map()) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolute = join(directory, entry.name);
    if (entry.isDirectory()) {
      collectTextEntries(root, absolute, entries);
    } else if (entry.isFile()) {
      entries.set(relative(root, absolute).replaceAll("\\", "/"), readFileSync(absolute, "utf8"));
    } else {
      throw new Error(`Real Project Matrix artifact contains unsupported entry: ${entry.name}`);
    }
  }
  if (directory === root && entries.size === 0 && statSync(root).isDirectory()) {
    throw new Error("Real Project Matrix artifact is empty");
  }
  return entries;
}
