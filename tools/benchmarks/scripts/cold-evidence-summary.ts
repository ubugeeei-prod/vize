/** Verify complete raw custody before deriving a bounded, lossless summary. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import {
  arrayEvidence as arr,
  integerEvidence as integer,
  isRecord,
  parseEvidenceJson as parse,
  recordEvidence as rec,
  stringifyEvidenceJson as stringify,
} from "./cold-evidence-json.ts";
const sha = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const text = (value: unknown) => {
  assert.equal(typeof value, "string");
  assert.ok(typeof value === "string");
  return value;
};
const pick = (value: Record<string, unknown>, keys: readonly string[]) =>
  Object.fromEntries(keys.filter((k) => Object.hasOwn(value, k)).map((k) => [k, value[k]]));
const metric = (value: Record<string, unknown>, group: string, field: string) =>
  integer(rec(value[group] ?? {})[field] ?? 0);
export function summarizeColdEvidence(root: string) {
  root = realpathSync(root);
  const manifestBytes = readFileSync(join(root, "manifest.json")),
    manifest = rec(parse(manifestBytes.toString("utf8")));
  assert.equal(arr(manifest.errors).length, 0);
  assert.equal(arr(manifest.requiredMissing).length, 0);
  const files = new Map<string, Buffer>(),
    identities = new Map<string, { bytes: number; sha256: string }>();
  for (const value of arr(manifest.selected)) {
    const entry = rec(value),
      path = text(entry.path);
    assert.ok(!path.startsWith("/") && !path.includes("\\") && !path.split("/").includes(".."));
    const file = join(root, "cold-scale", path);
    assert.ok(lstatSync(file).isFile() && !lstatSync(file).isSymbolicLink());
    assert.ok(realpathSync(file).startsWith(join(root, "cold-scale") + sep));
    const bytes = readFileSync(file);
    assert.equal(BigInt(bytes.length), integer(entry.bytes));
    assert.equal(sha(bytes), entry.sha256);
    files.set(path, bytes);
    identities.set(path, { bytes: bytes.length, sha256: sha(bytes) });
  }
  const builds: unknown[] = [],
    measurements: unknown[] = [],
    profiles: unknown[] = [],
    pmu: unknown[] = [];
  for (const [path, body] of files) {
    if (path.startsWith("builds/") && path.endsWith(".json")) {
      const value = rec(parse(body.toString("utf8"))),
        receipt = pick(value, [
          "role",
          "sourceSha",
          "treeSha",
          "binary",
          "binarySha256",
          "binaryBytes",
          "allocation",
          "targetDir",
          "targetStartedEmpty",
          "profile",
          "compilerEnvironment",
          "qualified",
          "reason",
          "base",
          "head",
        ]);
      for (const side of ["base", "head"])
        if (isRecord(receipt[side]))
          receipt[side] = pick(receipt[side], [
            "role",
            "sourceSha",
            "binarySha256",
            "allocation",
            "targetStartedEmpty",
            "qualified",
          ]);
      receipt.cargoArtifact = value.cargoArtifact ?? null;
      builds.push({ path, identity: identities.get(path), receipt });
    }
    if (/^profile-(?:base|head)\/profile.json$/u.test(path)) {
      const value = rec(parse(body.toString("utf8"))),
        spans = arr(value.spans).map(rec),
        chosen = new Map<string, Record<string, unknown>>();
      const selectors = [
        (v: Record<string, unknown>) => metric(v, "alloc", "self_calls"),
        (v: Record<string, unknown>) => metric(v, "alloc", "bytes"),
        (v: Record<string, unknown>) => metric(v, "wall_ns", "self"),
        (v: Record<string, unknown>) => integer(v.count ?? 0),
      ];
      for (const select of selectors)
        for (const span of [...spans]
          .sort((a, b) => (select(a) > select(b) ? -1 : select(a) < select(b) ? 1 : 0))
          .slice(0, 8))
          chosen.set(text(span.key), span);
      const counters = arr(value.counters).map(rec),
        selected = counters.filter((v) =>
          /route|workspace|hash|alloc|source|function|call|read|stat|realpath|parse|analyze|type.query/iu.test(
            text(v.key),
          ),
        );
      const allocation = value.allocation ?? null;
      const peak = isRecord(allocation)
        ? (["peak_live_bytes", "peakLiveBytes", "peak_live"]
            .map((k) => allocation[k])
            .find((v) => v !== undefined) ?? null)
        : null;
      profiles.push({
        path,
        identity: identities.get(path),
        tool: value.tool,
        version: value.tool_version,
        allocation,
        peakLiveBytes: peak,
        truncation: value.truncation,
        spans: [...chosen.values()],
        spanCount: spans.length,
        counters: selected.slice(0, 80),
        selectedCounterCount: selected.length,
        counterSummaryTruncated: selected.length > 80,
        hashCountersPresent: counters.some((v) => text(v.key).toLowerCase().includes("hash")),
        scope:
          "profiling-build requested allocation volume and instrumented spans; span entries are not universal physical calls; wall spans are not normal latency",
      });
    }
    if (path.endsWith("/measurement.json")) {
      const value = rec(parse(body.toString("utf8"))),
        stage = path.slice(0, path.lastIndexOf("/")),
        stdout = files.get(stage + "/stdout.txt");
      let packet: unknown = null;
      if (stdout) {
        try {
          packet = parse(stdout.toString("utf8"));
        } catch {
          packet = null;
        }
      }
      let packetCounts: unknown = null;
      if (isRecord(packet) && Array.isArray(packet.files)) {
        const rows = packet.files,
          names = rows.map((v) => rec(v).file);
        packetCounts = {
          arrayLength: rows.length,
          fileCount: packet.fileCount,
          uniqueNames: new Set(names).size,
          vue: names.filter((n) => typeof n === "string" && n.endsWith(".vue")).length,
          typescript: names.filter((n) => typeof n === "string" && /\.[cm]?tsx?$/u.test(n)).length,
        };
      }
      const corpusPath = stage + "/corpus.json",
        corpusBody = files.get(corpusPath);
      let corpus: unknown = null;
      if (corpusBody) {
        const raw = rec(parse(corpusBody.toString("utf8"))),
          rows = arr(raw.entries ?? raw.files).map(rec);
        const names = rows.map((v) => text(v.path ?? v.file ?? v.relativePath)),
          sourceBytes = rows.reduce((sum, v) => sum + integer(v.bytes), 0n);
        const corpusHash = sha(stringify(rows));
        assert.equal(sourceBytes, integer(raw.sourceBytes));
        assert.equal(corpusHash, raw.corpusSha256);
        assert.equal(new Set(names).size, names.length);
        corpus = {
          identity: identities.get(corpusPath),
          arrayLength: rows.length,
          computedSourceBytes: sourceBytes,
          computedCorpusSha256: corpusHash,
          uniqueNames: new Set(names).size,
          vue: names.filter((n) => n.endsWith(".vue")).length,
          typescript: names.filter((n) => /\.[cm]?tsx?$/u.test(n)).length,
          rootScalars: Object.fromEntries(
            Object.entries(raw).filter(([, v]) => !isRecord(v) && !Array.isArray(v)),
          ),
        };
      }
      const stderr = files.get(stage + "/stderr.txt")?.toString("utf8") ?? "",
        rss = /Maximum resident set size \(kbytes\):\s*(\d+)/u.exec(stderr);
      const receipt = pick(value, [
        "tool",
        "version",
        "files",
        "sfcFiles",
        "sourceBytes",
        "corpusSha256",
        "wallMs",
        "status",
        "signal",
        "executableSha256",
        "backendSha256",
        "args",
        "cold",
        "meetsColdTarget",
        "correctness",
      ]);
      if (isRecord(receipt.correctness) && Object.hasOwn(receipt.correctness, "diagnostics")) {
        const correctness = { ...receipt.correctness },
          diagnostics = correctness.diagnostics;
        delete correctness.diagnostics;
        correctness.diagnosticArrayLength = Array.isArray(diagnostics) ? diagnostics.length : null;
        receipt.correctness = correctness;
      }
      measurements.push({
        path,
        identity: identities.get(path),
        receipt,
        actualOutputCounts: packetCounts,
        actualCorpusCounts: corpus,
        maxProcessOrWaitedChildRssKib: rss?.[1] ? BigInt(rss[1]) : null,
        rssScope: "GNU time maximum process/waited child, not simultaneous process-tree sum",
      });
    }
    if (path.endsWith("/calibration.json")) {
      const value = rec(parse(body.toString("utf8")));
      pmu.push({
        path,
        identity: identities.get(path),
        qualified: value.qualified,
        groups: value.groups,
        reason: value.reason ?? null,
        hardwareCounterAcceptance: value.qualified ? "inspect individual counter receipts" : null,
      });
    }
    if (/^stat-(?:core|cache)-(?:base|head|mapper)\/.*\/receipt.json$/u.test(path)) {
      const value = rec(parse(body.toString("utf8")));
      pmu.push({
        path,
        identity: identities.get(path),
        qualified: value.qualified,
        collection: value.collection,
        metric: value.metric,
        capability: value.capability ?? null,
        calibrationQualified: value.calibrationQualified ?? null,
        acceptedCounters: value.qualified ? (value.counters ?? null) : null,
      });
    }
  }
  return {
    schemaVersion: 1,
    sourceRun: manifest.sourceRun,
    sourceArtifact: manifest.sourceArtifact,
    sourceArtifactApi: manifest.sourceArtifactApi,
    downloadedTarSha256: manifest.downloadedTarSha256,
    manifestSha256: sha(manifestBytes),
    verifiedMembers: files.size,
    verifiedBytes: manifest.totalBytes,
    builds,
    measurements,
    profiles,
    pmu,
    parseErrors: [],
    newBuild: false,
    newCapture: false,
    newCheckExecution: false,
    coldQualified: false,
    hardwareCounterAcceptance: null,
    integerEncoding:
      "JSON integer lexemes are preserved exactly; unsafe integers require a lossless consumer",
  };
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [source, destination] = process.argv.slice(2);
  assert.ok(source && destination);
  const result = summarizeColdEvidence(source),
    body = stringify(result) + "\n";
  assert.ok(
    Buffer.byteLength(body) <= 100 * 1024,
    "bounded summary exceeded 100KiB; raw custody stays remote",
  );
  writeFileSync(destination, body);
  console.log(
    stringify({
      summaryBytes: Buffer.byteLength(body),
      verifiedMembers: result.verifiedMembers,
      sourceRun: result.sourceRun,
      sourceArtifact: result.sourceArtifact,
    }),
  );
}
