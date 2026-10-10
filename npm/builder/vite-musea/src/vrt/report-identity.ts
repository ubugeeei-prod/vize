import { createHash } from "node:crypto";
import { lstat, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { resolveSnapshotIdentity } from "./snapshot-identity.js";

export interface ArtReportTarget {
  identity: string;
  jsonReportPath: string;
  htmlReportPath: string;
}

/** Reserve one qualified namespace and compare legacy names on case-insensitive filesystems. */
function legacyName(identity: string): string | undefined {
  const basename = path.posix.basename(identity, ".art.vue");
  return /^[a-z0-9._-]{1,200}$/i.test(basename) && !/^art-[0-9a-f]{64}$/i.test(basename)
    ? `vrt-${basename}`
    : undefined;
}

/** The complete discovered Art set decides collisions, even for a single-Art request. */
export function resolveArtReportTarget(
  artPath: string,
  artPaths: Iterable<string>,
  projectRoot: string,
  reportDir: string,
): ArtReportTarget {
  const identities = Array.from(artPaths, (item) => resolveSnapshotIdentity(item, projectRoot));
  const identity = resolveSnapshotIdentity(artPath, projectRoot);
  if (!identities.includes(identity)) throw new Error(`Report Art not found: ${artPath}`);
  const candidates = identities.map(legacyName);
  const counts = new Map<string, number>();
  for (const name of candidates) {
    if (name) counts.set(name.toLowerCase(), (counts.get(name.toLowerCase()) ?? 0) + 1);
  }
  const names = identities.map((item, index) => {
    const candidate = candidates[index];
    const colliding = candidate && counts.get(candidate.toLowerCase())! > 1;
    return candidate && !colliding
      ? candidate
      : `vrt-art-${createHash("sha256")
          .update(JSON.stringify([1, item]))
          .digest("hex")}`;
  });
  if (new Set(names.map((name) => name.toLowerCase())).size !== names.length)
    throw new Error("Ambiguous VRT report identities");
  const base = names[identities.indexOf(identity)];
  return {
    identity,
    jsonReportPath: path.join(reportDir, `${base}-report.json`),
    htmlReportPath: path.join(reportDir, `${base}-report.html`),
  };
}

async function existsRegularFile(file: string): Promise<boolean> {
  try {
    if (!(await lstat(file)).isFile()) throw new Error(`VRT report is not a regular file: ${file}`);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
}

/** Fail before capture or writes; legacy adoption requires every recorded result to have this owner. */
export async function assertArtReportOwnership(
  target: ArtReportTarget,
  projectRoot: string,
): Promise<void> {
  const refusal = `Ambiguous VRT report ownership: ${target.jsonReportPath}. Archive or move the existing reports before retrying.`;
  const names = [path.basename(target.jsonReportPath), path.basename(target.htmlReportPath)];
  let existingNames: string[];
  try {
    existingNames = await readdir(path.dirname(target.jsonReportPath));
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
  if (
    existingNames.some((item) =>
      names.some((name) => item !== name && item.toLowerCase() === name.toLowerCase()),
    )
  )
    throw new Error(refusal);
  const jsonExists = await existsRegularFile(target.jsonReportPath);
  const htmlExists = await existsRegularFile(target.htmlReportPath);
  if (!jsonExists && !htmlExists) return;
  if (!jsonExists) throw new Error(refusal);
  try {
    const report: unknown = JSON.parse(await readFile(target.jsonReportPath, "utf8"));
    if (!report || typeof report !== "object" || Array.isArray(report)) throw new Error(refusal);
    if (Object.hasOwn(report, "reportOwner")) {
      const owner = (report as { reportOwner: unknown }).reportOwner;
      if (
        !owner ||
        typeof owner !== "object" ||
        (owner as { version?: unknown }).version !== 1 ||
        (owner as { artIdentity?: unknown }).artIdentity !== target.identity
      )
        throw new Error(refusal);
    } else {
      const results = (report as { results?: unknown }).results;
      if (
        !Array.isArray(results) ||
        !results.length ||
        results.some(
          (item: unknown) =>
            !item ||
            typeof item !== "object" ||
            typeof (item as { artPath?: unknown }).artPath !== "string" ||
            resolveSnapshotIdentity((item as { artPath: string }).artPath, projectRoot) !==
              target.identity,
        )
      )
        throw new Error(refusal);
    }
  } catch {
    throw new Error(refusal);
  }
}

/** Keep ownership portable when a report directory moves to another machine. */
export function attachArtReportOwner(json: string, target: ArtReportTarget): string {
  return JSON.stringify(
    { ...JSON.parse(json), reportOwner: { version: 1, artIdentity: target.identity } },
    null,
    2,
  );
}
