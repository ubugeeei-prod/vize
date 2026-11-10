/** Full observations first; cross-engine finding equality is never an accuracy waiver. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
export const source = "ea22f2cd2aa661b9bdab940e25464eff692b935f";
export const misskeyRevision = "810faa8e5db4bbf9d6d408a61df148559340ee99";
export const providerPins = {
  package: "bb62ff5452cbfd7866d232c6e1a78d1d7b6d82ec5f6cbf67e08aa6deff9c324e",
  lock: "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6",
  native: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
};
export const orders = [
  ["vize", "vue-tsc"],
  ["vue-tsc", "vize"],
  ["vue-tsc", "vize"],
  ["vize", "vue-tsc"],
] as const;
export const hash = (body: string | Buffer) => createHash("sha256").update(body).digest("hex");
export function save(file: string, value: unknown) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function identity(file: string) {
  const real = fs.realpathSync(file),
    body = fs.readFileSync(real);
  return { path: real, bytes: body.length, sha256: hash(body) };
}
export type Finding = {
  file: string | null;
  line: number | null;
  column: number | null;
  code: number;
  severity: string;
  message: string;
  end: null;
  related: null;
};
export function diagnostics(tool: string, stdout: string, cwd: string) {
  const findings: Finding[] = [],
    unclassified: string[] = [];
  let packet: unknown = null;
  if (tool === "vize") {
    packet = JSON.parse(stdout);
    assert(packet && typeof packet === "object" && !Array.isArray(packet));
    const p = packet as {
      files: { file: string; diagnostics: string[] }[];
      errorCount: number;
      warningCount: number;
      fileCount: number;
      programs: unknown[];
    };
    assert(Array.isArray(p.files) && Array.isArray(p.programs));
    assert(Number.isSafeInteger(p.fileCount) && p.fileCount > 0);
    for (const file of p.files) {
      assert.equal(typeof file.file, "string");
      assert(Array.isArray(file.diagnostics));
      for (const text of file.diagnostics) {
        const m = /^(error|warning|info):(\d+):(\d+) \[TS(\d+)\] ([\s\S]*)$/u.exec(text);
        if (!m) {
          unclassified.push(JSON.stringify({ file: file.file, text }));
          continue;
        }
        findings.push({
          file: path.resolve(cwd, file.file),
          severity: m[1],
          line: Number(m[2]),
          column: Number(m[3]),
          code: Number(m[4]),
          message: m[5],
          end: null,
          related: null,
        });
      }
    }
    assert.equal(findings.filter((f) => f.severity === "error").length, p.errorCount);
    assert.equal(findings.filter((f) => f.severity === "warning").length, p.warningCount);
  } else {
    let current: Finding | undefined;
    for (const text of stdout.split(/\r?\n/u)) {
      const m = /^(.*?)\((\d+),(\d+)\): (error|warning|info) TS(\d+): ([\s\S]*)$/u.exec(text);
      const global = /^(error|warning|info) TS(\d+): ([\s\S]*)$/u.exec(text);
      if (m || global) {
        current = m
          ? {
              file: path.resolve(cwd, m[1]),
              line: Number(m[2]),
              column: Number(m[3]),
              severity: m[4],
              code: Number(m[5]),
              message: m[6],
              end: null,
              related: null,
            }
          : {
              file: null,
              line: null,
              column: null,
              severity: global![1],
              code: Number(global![2]),
              message: global![3],
              end: null,
              related: null,
            };
        findings.push(current);
      } else if (/^\s/u.test(text) && current) current.message += "\n" + text;
      else if (text) {
        unclassified.push(text);
        current = undefined;
      }
    }
  }
  return {
    packet,
    findings,
    unclassified,
    endAndStructuredRelated:
      "not exposed by these CLI formats; complete raw streams/whole Vize packet retained",
  };
}
export function compareFindings(a: Finding[], b: Finding[]) {
  const remaining = b.map((finding, index) => ({ finding, index })),
    matched: number[] = [],
    unmatched = [];
  for (const finding of a) {
    const at = remaining.findIndex((row) => isDeepStrictEqual(row.finding, finding));
    if (at >= 0) matched.push(remaining.splice(at, 1)[0].index);
    else
      unmatched.push({
        tool: "vize",
        finding,
        classification: "UNRESOLVED_ENGINE_OR_MAPPING_OR_MISSED_WORK",
      });
  }
  unmatched.push(
    ...remaining.map((row) => ({
      tool: "vue-tsc",
      finding: row.finding,
      classification: "UNRESOLVED_ENGINE_OR_MAPPING_OR_MISSED_WORK",
    })),
  );
  return {
    matched: matched.length,
    unmatched,
    qualified: unmatched.length === 0,
    registryToleranceApplied: false,
  };
}
export function gnuTime(text: string) {
  const value = (label: string, pattern: RegExp) => {
    const rows = text.split("\n").filter((line) => line.trimStart().startsWith(label + ":"));
    assert.equal(rows.length, 1, "unique GNU time field: " + label);
    const token = rows[0].slice(rows[0].indexOf(":") + 1).trim();
    assert(pattern.test(token), "malformed GNU time field: " + label);
    return token;
  };
  const user = value("User time (seconds)", /^\d+\.\d{2}$/u),
    system = value("System time (seconds)", /^\d+\.\d{2}$/u);
  const rss = value("Maximum resident set size (kbytes)", /^\d+$/u);
  const cpuMs = Math.round((Number(user) + Number(system)) * 1000);
  assert(Number.isSafeInteger(cpuMs) && cpuMs > 0 && Number(rss) > 0);
  return {
    user,
    system,
    cpuMs,
    resolutionMs: 10,
    childHighWaterKiB: Number(rss),
    rssScope: "GNU time child high-water; not process-tree simultaneous peak",
  };
}

/** Exact cold-run spans that attribute end-to-end batch-check work. */
export function profileAttribution(report: unknown) {
  assert(report && typeof report === "object" && !Array.isArray(report));
  const value = report as {
    schema_version?: unknown;
    truncation?: { dropped_spans?: unknown; dropped_counters?: unknown };
    spans?: unknown;
  };
  assert.equal(value.schema_version, 1);
  assert.deepEqual(value.truncation, { dropped_spans: 0, dropped_counters: 0 });
  assert(Array.isArray(value.spans));
  const selected = value.spans
    .map((entry) => {
      assert(entry && typeof entry === "object" && !Array.isArray(entry));
      const span = entry as {
        key?: unknown;
        count?: unknown;
        wall_ns?: { total?: unknown; self?: unknown; min?: unknown; max?: unknown };
      };
      assert.equal(typeof span.key, "string");
      const key = span.key as string;
      assert(Number.isSafeInteger(span.count) && Number(span.count) >= 0);
      assert(span.wall_ns && typeof span.wall_ns === "object");
      for (const field of ["total", "self", "min", "max"] as const)
        assert(
          Number.isSafeInteger(span.wall_ns[field]) && Number(span.wall_ns[field]) >= 0,
          `${key}.wall_ns.${field}`,
        );
      return {
        key,
        count: span.count as number,
        wallNs: {
          total: span.wall_ns.total as number,
          self: span.wall_ns.self as number,
          min: span.wall_ns.min as number,
          max: span.wall_ns.max as number,
        },
      };
    })
    .filter(
      (span) =>
        span.key.startsWith("cli.check.collect.") ||
        span.key.startsWith("cli.check.imports.") ||
        span.key === "canon.vue.virtual_ts" ||
        span.key.startsWith("canon.executor.") ||
        span.key.startsWith("canon.corsa."),
    )
    .toSorted(
      (left, right) => right.wallNs.total - left.wallNs.total || left.key.localeCompare(right.key),
    );
  for (const key of [
    "cli.check.collect.imports",
    "canon.vue.virtual_ts",
    "canon.executor.materialize",
    "canon.corsa.cli",
  ])
    assert(
      selected.some((span) => span.key === key),
      `missing required profile stage ${key}`,
    );
  return { schemaVersion: 1, truncation: value.truncation, spans: selected };
}
/** The retained raw profile file and its exact batch-stage attribution. */
export function coldProfile(command: string, file: string) {
  return {
    command,
    raw: identity(file),
    attribution: profileAttribution(JSON.parse(fs.readFileSync(file, "utf8"))),
  };
}
export function inventory(root: string, exclude: string[] = []) {
  const rows: Record<string, unknown>[] = [];
  function visit(dir: string) {
    for (const name of fs.readdirSync(dir).sort()) {
      if (exclude.includes(name)) continue;
      const file = path.join(dir, name),
        relative = path.relative(root, file),
        s = fs.lstatSync(file);
      if (s.isSymbolicLink())
        rows.push({
          path: relative,
          kind: "symlink",
          target: fs.readlinkSync(file),
          physical: fs.realpathSync(file),
        });
      else if (s.isDirectory()) visit(file);
      else {
        assert(s.isFile());
        rows.push({
          path: relative,
          kind: "file",
          bytes: s.size,
          mode: s.mode,
          sha256: hash(fs.readFileSync(file)),
        });
      }
    }
  }
  visit(root);
  return { root, rows, sha256: hash(JSON.stringify(rows)) };
}

export function statusConsistent(status: number | null, parsed: ReturnType<typeof diagnostics>) {
  const errors = parsed.findings.filter((row) => row.severity === "error").length;
  return (
    parsed.unclassified.length === 0 && (errors > 0 ? status === 1 || status === 2 : status === 0)
  );
}

export function assertStatusStable(before: number | null, after: number | null) {
  assert.equal(after, before, "same-tool timed status drift");
}
export function assessStderr(stderr: string) {
  return {
    empty: stderr.length === 0,
    unresolvedLines: stderr.split(/\r?\n/u).filter(Boolean),
    classification: "all nonempty stderr unresolved; raw retained, no word-based clean inference",
  };
}
