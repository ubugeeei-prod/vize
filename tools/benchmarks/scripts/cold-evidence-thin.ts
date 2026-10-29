/** Bounded metadata-only extraction; captured executables are never run. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, posix, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { evidenceTar, type EvidenceTarMember } from "./cold-evidence-tar.ts";
import {
  integerEvidence,
  parseEvidenceJson,
  recordEvidence,
  stringifyEvidenceJson,
} from "./cold-evidence-json.ts";

const perFile = 8 * 1024 * 1024,
  totalLimit = 32 * 1024 * 1024;
const roots = new Set([
  "baseline-selection.json",
  "base.sha",
  "head.sha",
  "package.json",
  "package-lock.json",
  "addr2line-version.txt",
]);
const measures = new Set([
  "measurement.json",
  "corpus.json",
  "stdout.txt",
  "stderr.txt",
  "profile.json",
]);
const required = [
  "base.sha",
  "head.sha",
  "builds/normal-base.json",
  "builds/normal-head.json",
  "builds/normal-pair.json",
  "builds/profile-base.json",
  "builds/profile-head.json",
  "builds/profile-pair.json",
  "profile-base/profile.json",
  "profile-head/profile.json",
  "pmu-controls/calibration.json",
];
export function selectedEvidence(name: string) {
  const clean = name.startsWith("./") ? name.slice(2) : name,
    original = clean.split("/");
  if (posix.isAbsolute(clean) || original.includes("..") || clean.includes("\\")) return null;
  const parts = posix.normalize(clean).replace(/\/+$/u, "").split("/");
  if (parts.shift() !== "cold-scale" || parts.length === 0) return null;
  if (parts.some((p) => ["project", "node_modules", "tmp", "xdg-cache", "binaries"].includes(p)))
    return null;
  const relative = parts.join("/"),
    first = parts[0] ?? "";
  if (parts.length === 1) return roots.has(relative) ? relative : null;
  if (first === "builds" && parts.length === 2 && relative.endsWith(".json")) return relative;
  if (
    /^(?:(?:base|head)-[123]|mapper|vue-tsc|control-(?:cli|mapper|vue-tsc)|profile-(?:base|head))$/u.test(
      first,
    ) &&
    parts.length === 2 &&
    measures.has(parts[1] ?? "")
  )
    return relative;
  if (
    /^(?:pmu-controls|stat-(?:core|cache)-(?:base|head|mapper)|record-(?:head|mapper))$/u.test(
      first,
    ) &&
    /\.(?:json|txt|csv|log)$/u.test(relative)
  )
    return relative;
  return null;
}
async function hashFile(file: string) {
  const hash = createHash("sha256");
  for await (const part of createReadStream(file)) hash.update(part);
  return hash.digest("hex");
}
export function coldEvidenceCollector(
  write: (path: string, bytes: Buffer) => void,
  requiredPaths: readonly string[] = required,
) {
  const selected: { path: string; bytes: number; sha256: string }[] = [],
    errors: { path: string; reason: string }[] = [];
  const seen = new Set<string>();
  let totalBytes = 0;
  const retain = (member: { name: string; kind: string; size: bigint }) => {
    const path = selectedEvidence(member.name);
    return (
      path !== null &&
      member.kind === "file" &&
      !seen.has(path) &&
      member.size <= BigInt(perFile) &&
      member.size + BigInt(totalBytes) <= BigInt(totalLimit)
    );
  };
  const consume = (member: EvidenceTarMember) => {
    const path = selectedEvidence(member.name);
    if (path === null) return;
    if (member.kind !== "file" || seen.has(path) || member.size > BigInt(perFile)) {
      errors.push({ path, reason: "nonregular, duplicate, or oversized" });
      return;
    }
    seen.add(path);
    if (member.size + BigInt(totalBytes) > BigInt(totalLimit)) {
      errors.push({ path, reason: "total metadata limit" });
      return;
    }
    assert.ok(member.data);
    assert.equal(BigInt(member.data.length), member.size);
    new TextDecoder("utf-8", { fatal: true }).decode(member.data);
    write(path, member.data);
    totalBytes += member.data.length;
    selected.push({
      path,
      bytes: member.data.length,
      sha256: createHash("sha256").update(member.data).digest("hex"),
    });
  };
  const result = () => {
    const copied = new Set(selected.map((v) => v.path)),
      requiredMissing = requiredPaths.filter((v) => !copied.has(v)).sort();
    return { selected, errors, totalBytes, requiredMissing };
  };
  return { retain, consume, result };
}
export async function extractColdEvidence(
  archive: string,
  out: string,
  run: string,
  artifact: string,
  apiFile: string,
) {
  assert.match(run, /^\d+$/u);
  assert.match(artifact, /^\d+$/u);
  const api = recordEvidence(parseEvidenceJson(readFileSync(apiFile, "utf8")));
  assert.equal(String(api.id), artifact);
  assert.equal(String(recordEvidence(api.workflow_run).id), run);
  assert.equal(api.expired, false);
  assert.ok(integerEvidence(api.size_in_bytes) <= 300n * 1024n * 1024n);
  assert.ok(!existsSync(out));
  mkdirSync(out);
  const collector = coldEvidenceCollector((path, bytes) => {
    const target = join(out, "cold-scale", path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
  });
  for await (const member of evidenceTar(archive, collector.retain, perFile))
    collector.consume(member);
  const { selected, errors, totalBytes, requiredMissing } = collector.result();
  const manifest = {
    selected,
    errors,
    totalBytes,
    requiredMissing,
    schemaVersion: 1,
    sourceRun: run,
    sourceArtifact: artifact,
    sourceArtifactApi: api,
    downloadedTarSha256: await hashFile(archive),
    driverSource: process.env.GITHUB_SHA ?? null,
    analysisPerformed: false,
    newBuild: false,
    newCapture: false,
    newCheckExecution: false,
    coldQualified: false,
    scope:
      "verbatim metadata copy only; declared ZIP digest is from API, tar digest independently computed",
  };
  writeFileSync(join(out, "manifest.json"), stringifyEvidenceJson(manifest, true) + "\n");
  assert.equal(errors.length, 0);
  assert.equal(requiredMissing.length, 0);
  console.log(
    stringifyEvidenceJson({
      selectedFiles: selected.length,
      bytes: totalBytes,
      sourceRun: run,
      sourceArtifact: artifact,
      newExecution: false,
    }),
  );
  return manifest;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [archive, out, run, artifact, api] = process.argv.slice(2);
  assert.ok(archive && out && run && artifact && api);
  await extractColdEvidence(archive, out, run, artifact, api);
}
