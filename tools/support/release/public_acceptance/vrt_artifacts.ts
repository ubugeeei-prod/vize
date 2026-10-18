import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdir, readFile, readdir, lstat, realpath, writeFile } from "node:fs/promises";
import path from "node:path";
import { sha256 } from "./vrt_fixtures.ts";

export interface VrtSharedResult {
  artPath: string;
  variantName: string;
  viewport: string;
  snapshotPath: string;
  currentPath?: string;
  diffPath?: string;
  diffPercentage?: number;
  error?: string;
}
export interface VrtResult extends VrtSharedResult {
  passed: boolean;
  isNew?: boolean;
}
export interface WrittenResult extends VrtSharedResult {
  art: string;
  variant: string;
  diffPixels?: number;
  totalPixels?: number;
  status: "error" | "new" | "passed" | "failed";
}
export interface VrtSummary {
  total: number;
  new: number;
  passed: number;
  failed: number;
  errors: number;
  skipped: number;
  duration: number;
}
export type VrtCounts = Pick<VrtSummary, "new" | "passed" | "failed">;
export interface VrtReport {
  results: WrittenResult[];
  summary: VrtSummary;
  reportOwner?: { version: number; artIdentity: string };
}
export interface ApiCapture {
  success: boolean;
  results: VrtResult[];
  summary: VrtSummary;
  artifacts: { snapshotDir: string; jsonReportPath: string; htmlReportPath: string };
}
export interface RetainedCapture {
  data: ApiCapture;
  json: Buffer;
  html: Buffer;
  png: Buffer;
}
export interface SessionCapture {
  success: boolean;
  results: Array<
    Omit<VrtResult, "snapshotPath" | "currentPath" | "diffPath"> & {
      images: Record<string, string>;
    }
  >;
  summary: VrtSummary;
  reports: { json: string; html: string };
}
export interface Evidence {
  output: string;
  records: unknown[];
  save(name: string, bytes: string | Uint8Array): Promise<void>;
  json(name: string, value: unknown): Promise<void>;
}
export async function evidence(output: string): Promise<Evidence> {
  await mkdir(output);
  const records: unknown[] = [];
  const save = async (name: string, bytes: string | Uint8Array) => {
    assert.match(name, /^[a-zA-Z0-9._/-]+$/);
    assert.ok(!name.split("/").includes(".."));
    const target = path.join(output, name);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, bytes, { flag: "wx" });
    records.push({
      phase: "retained-file",
      file: name,
      bytes: Buffer.byteLength(bytes),
      sha256: sha256(bytes),
    });
  };
  return {
    output,
    records,
    save,
    json: (name, value) => save(name, JSON.stringify(value, null, 2) + "\n"),
  };
}

/** Every cleanup is attempted in order, including when a preceding close or evidence write fails. */
export async function cleanup(
  e: Evidence,
  phase: string,
  steps: Array<{ name: string; run: () => Promise<unknown> }>,
) {
  const failures: unknown[] = [];
  for (const step of steps) {
    try {
      await step.run();
    } catch (error) {
      failures.push(error);
      e.records.push({ phase, operation: step.name, error: String(error) });
    }
  }
  if (failures.length) throw new AggregateError(failures, `${phase} failed`);
}

export function assertVrtCustody(
  vrt: {
    schema?: unknown;
    version?: unknown;
    success?: unknown;
    packages?: unknown;
    loaded?: unknown;
  },
  native: { version: unknown; packages: unknown; loaded: unknown },
) {
  assert.equal(vrt.schema, "vize-public-installed-musea-vrt-v1");
  assert.equal(vrt.success, true);
  assert.equal(vrt.version, native.version);
  assert.deepEqual(vrt.packages, native.packages, "VRT uses the same public installation");
  assert.deepEqual(vrt.loaded, native.loaded, "VRT loaded the same native provider bytes");
}

/** PNG decoding uses the already installed Musea dependency, never a second installation. */
export function pngDetails(bytes: Buffer) {
  const manifest = path.join(process.cwd(), "node_modules/@vizejs/vite-plugin-musea/package.json");
  const require = createRequire(manifest);
  const dependency = require.resolve("pngjs");
  assert.ok(dependency.startsWith(path.join(process.cwd(), "node_modules") + path.sep));
  const { PNG } = require(dependency) as {
    PNG: { sync: { read(bytes: Buffer): { width: number; height: number; data: Buffer } } };
  };
  const png = PNG.sync.read(bytes);
  assert.deepEqual([png.width, png.height], [320, 180]);
  const colors = { blue: 0, green: 0, red: 0 };
  for (let i = 0; i < png.data.length; i += 4) {
    const [r, g, b] = png.data.subarray(i, i + 3);
    if (r === 0 && g === 0 && b === 255) colors.blue++;
    if (r === 0 && g === 255 && b === 0) colors.green++;
    if (r === 255 && g === 0 && b === 0) colors.red++;
  }
  return { width: png.width, height: png.height, colors, sha256: sha256(bytes) };
}

export function assertSummary(report: VrtReport | ApiCapture, total: number, counts: VrtCounts) {
  assert.equal(report.summary.total, total);
  assert.equal(report.summary.errors, 0);
  assert.equal(report.summary.skipped, 0);
  assert.ok(Number.isFinite(report.summary.duration) && report.summary.duration >= 0);
  for (const [key, value] of Object.entries(counts))
    assert.equal(Reflect.get(report.summary, key), value, key);
  assert.equal(report.results.length, total);
  const fromApi = "success" in report;
  if (fromApi) assert.equal(report.success, true);
  const statuses = report.results.map((result) => {
    assert.equal(result.error, undefined);
    assert.equal(result.variantName, "Default");
    assert.equal(result.viewport, "authored-compact");
    if (fromApi) {
      const actual = result as VrtResult;
      assert.equal(Object.hasOwn(actual, "passed"), true);
      assert.equal(typeof actual.passed, "boolean");
      if (actual.isNew === true) assert.equal(actual.passed, true);
      else assert.equal(Object.hasOwn(actual, "isNew"), false);
      const status = actual.isNew ? "new" : actual.passed ? "passed" : "failed";
      if (status === "new") assert.equal(actual.diffPercentage, undefined);
      return status;
    }
    const written = result as WrittenResult;
    assert.equal(Object.hasOwn(written, "passed"), false);
    assert.equal(Object.hasOwn(written, "isNew"), false);
    assert.ok(["new", "passed", "failed"].includes(written.status));
    if (written.status === "new") {
      assert.equal(written.diffPercentage, undefined);
      assert.equal(written.diffPixels, undefined);
      assert.equal(written.totalPixels, undefined);
    }
    return written.status;
  });
  for (const status of ["new", "passed", "failed"] as const)
    assert.equal(statuses.filter((actual) => actual === status).length, counts[status]);
}

/** Bind whole physical JSON data to the actual parsed HTTP packet without normalizing either. */
export function assertApiReport(
  report: VrtReport,
  packet: Pick<ApiCapture, "results" | "summary">,
  width: number,
  height: number,
) {
  assert.deepEqual(report.summary, packet.summary);
  assert.equal(report.results.length, packet.results.length);
  const shared = [
    "artPath",
    "variantName",
    "viewport",
    "snapshotPath",
    "currentPath",
    "diffPath",
    "diffPercentage",
    "error",
  ];
  for (const [index, written] of report.results.entries()) {
    const actual = packet.results[index];
    for (const key of shared) {
      assert.equal(Object.hasOwn(written, key), Object.hasOwn(actual, key), key);
      assert.deepEqual(Reflect.get(written, key), Reflect.get(actual, key), key);
    }
    assert.equal(typeof actual.viewport, "string");
    assert.equal(Object.hasOwn(actual, "passed"), true);
    assert.equal(typeof actual.passed, "boolean");
    assert.equal(Object.hasOwn(written, "passed"), false);
    assert.equal(Object.hasOwn(written, "isNew"), false);
    assert.equal(written.art, path.basename(actual.artPath, ".art.vue"));
    assert.equal(written.variant, actual.variantName);
    const status = actual.error
      ? "error"
      : actual.isNew
        ? "new"
        : actual.passed
          ? "passed"
          : "failed";
    assert.equal(written.status, status);
    if (status === "new") {
      assert.equal(actual.passed, true);
      assert.equal(actual.isNew, true);
    } else assert.equal(Object.hasOwn(actual, "isNew"), false);
    if (status === "error") assert.equal(actual.passed, false);
    if (status === "new" || status === "error") {
      assert.equal(written.diffPixels, undefined);
      assert.equal(written.totalPixels, undefined);
      assert.equal(written.diffPercentage, undefined);
    } else {
      assert.equal(written.totalPixels, width * height);
      assert.ok(
        Number.isSafeInteger(written.diffPixels) &&
          written.diffPixels! >= 0 &&
          written.diffPixels! <= written.totalPixels!,
      );
      assert.equal(written.diffPercentage, (written.diffPixels! / written.totalPixels!) * 100);
    }
  }
}

/** The public session exposes opaque artifacts, never the private API's filesystem paths. */
export async function assertSessionReport(input: {
  report: { data: VrtReport; json: Buffer; html: Buffer };
  packet: SessionCapture;
  root: string;
  width: number;
  height: number;
  fetch: (route: string, type: string) => Promise<Buffer>;
}) {
  const { report, packet, root } = input;
  assert.deepEqual(report.data, JSON.parse(report.json.toString()));
  assert.deepEqual(Object.keys(packet).sort(), ["reports", "results", "success", "summary"]);
  assert.equal(packet.success, true);
  assert.deepEqual(report.data.summary, packet.summary);
  assertSummary(report.data, packet.results.length, packet.summary);
  assert.deepEqual(Object.keys(packet.reports).sort(), ["html", "json"]);
  assert.equal(path.resolve(root), root);
  const physicalRoot = await realpath(root);
  const bodies = new Map<string, Buffer>();
  const acquire = async (route: string, type: string, expected: Buffer) => {
    assert.equal(typeof route, "string");
    assert.match(
      route,
      /^\/artifacts\/[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );
    assert.equal(bodies.has(route), false, "distinct artifact registrations must not alias");
    const bytes = await input.fetch(route, type);
    assert.deepEqual(bytes, expected, "opaque artifact preserves whole physical bytes");
    bodies.set(route, bytes);
  };
  for (const [index, written] of report.data.results.entries()) {
    const actual = packet.results[index];
    const { images, ...metadata } = actual;
    assert.ok(images && typeof images === "object" && !Array.isArray(images));
    const expected = {
      artPath: written.artPath,
      variantName: written.variantName,
      viewport: written.viewport,
      passed: written.status === "new" || written.status === "passed",
      ...(written.status === "new" ? { isNew: true } : {}),
      ...(Object.hasOwn(written, "diffPercentage")
        ? { diffPercentage: written.diffPercentage }
        : {}),
      ...(Object.hasOwn(written, "error") ? { error: written.error } : {}),
    };
    assert.deepEqual(
      metadata,
      expected,
      "exact public projection, including absent private fields",
    );
    assert.equal(written.art, path.basename(written.artPath, ".art.vue"));
    assert.equal(written.variant, written.variantName);
    if (written.status !== "new") {
      assert.equal(written.totalPixels, input.width * input.height);
      assert.ok(
        Number.isSafeInteger(written.diffPixels) &&
          written.diffPixels! >= 0 &&
          written.diffPixels! <= written.totalPixels!,
      );
      assert.equal(written.diffPercentage, (written.diffPixels! / written.totalPixels!) * 100);
    }
    assert.equal(typeof written.snapshotPath, "string");
    assert.equal(typeof written.currentPath, "string");
    const candidates: Array<[string, string | undefined]> = [
      ["snapshot", written.snapshotPath],
      ["current", written.currentPath],
      ["diff", written.diffPath],
    ];
    const files = candidates.filter((entry) => entry[1] !== undefined);
    assert.deepEqual(Object.keys(images).sort(), files.map(([kind]) => kind).sort());
    for (const [kind, file] of files) {
      assert.ok(file && path.isAbsolute(file) && file.startsWith(root + path.sep));
      const info = await lstat(file);
      assert.ok(info.isFile() && !info.isSymbolicLink());
      assert.ok((await realpath(file)).startsWith(physicalRoot + path.sep));
      await acquire(images[kind], "image/png", await readFile(file));
    }
  }
  await acquire(packet.reports.json, "application/json", report.json);
  await acquire(packet.reports.html, "text/html", report.html);
  return bodies;
}

export async function retainReport(
  e: Evidence,
  phase: string,
  jsonFile: string,
  htmlFile?: string,
) {
  const json = await readFile(jsonFile);
  await e.save(`${phase}/report.json`, json);
  const data = JSON.parse(json.toString()) as VrtReport;
  if (htmlFile) await e.save(`${phase}/report.html`, await readFile(htmlFile));
  for (const [index, result] of data.results.entries()) {
    for (const [kind, file] of [
      ["snapshot", result.snapshotPath],
      ["current", result.currentPath],
      ["diff", result.diffPath],
    ] as const) {
      if (!file) continue;
      const bytes = await readFile(file);
      await e.save(`${phase}/${index}-${kind}.png`, bytes);
      e.records.push({ phase, index, kind, file, ...pngDetails(bytes) });
    }
  }
  return { data, json };
}

export async function inventory(
  root: string,
  current = root,
): Promise<Array<{ path: string; sha256: string; size: string; mtimeNs: string }>> {
  const result: Awaited<ReturnType<typeof inventory>> = [];
  for (const entry of await readdir(current, { withFileTypes: true })) {
    const file = path.join(current, entry.name);
    const info = await lstat(file, { bigint: true });
    assert.equal(info.isSymbolicLink(), false, file);
    if (entry.isDirectory()) result.push(...(await inventory(root, file)));
    else
      result.push({
        path: path.relative(root, file),
        sha256: sha256(await readFile(file)),
        size: String(info.size),
        mtimeNs: String(info.mtimeNs),
      });
  }
  return result.sort((a, b) => a.path.localeCompare(b.path));
}

export async function ownershipIndex(snapshotDir: string) {
  const bytes = await readFile(path.join(snapshotDir, "identities.json"));
  const data = JSON.parse(bytes.toString()) as { version: number; owners: Record<string, string> };
  assert.equal(data.version, 1);
  assert.equal(Object.keys(data.owners).length, 2);
  assert.equal(new Set(Object.values(data.owners)).size, 2);
  assert.deepEqual(
    Object.values(data.owners)
      .map((id) => JSON.parse(id))
      .sort((a, b) => a[1].localeCompare(b[1])),
    ["left", "right"].map((side) => [
      1,
      `src/${side}/Button.art.vue`,
      "Default",
      "authored-compact",
      320,
      180,
      1,
    ]),
  );
  await assert.rejects(lstat(path.join(snapshotDir, "identities.lock")), { code: "ENOENT" });
  return { bytes, data };
}
