/** Observational source column; this does not waive the original conservation gate. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { identify, run, save, wrapperSource } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue, packet, type Diagnostic } from "./canon-instance-directives-model.ts";
import { sdk, native, type Row } from "./canon-slot-constructor-audit.ts";
import { boundaryEvidence } from "./canon-slot-framework-boundaries.ts";
type Case = { id: string; component: string; source: string; requiresAuthoredError: boolean };
type Located = { file: string; line: number; column: number; code: number | null };
const key = (row: Located) => JSON.stringify([row.file, row.line, row.column, row.code]);
export function sourceCandidateColumn(
  originalProject: string,
  provider: string,
  binary: string,
  checker: string,
  rows: Case[],
  official: Row[],
  out: string,
) {
  const project = path.join(out, "source-candidate");
  fs.cpSync(originalProject, project, {
    recursive: true,
    filter: (file) => path.basename(file) !== "node_modules",
  });
  sdk(project, provider, binary);
  const sourceImage = identify(binary),
    nativeImage = identify(checker),
    inputs = [];
  for (const row of rows)
    for (const [file, text] of [
      ["component.ts", row.component],
      ["src/Tiny.vue", row.source],
    ]) {
      assert.ok(file && text);
      const relative = path.join(row.id, file),
        copied = path.join(project, relative);
      assert.equal(fs.readFileSync(copied, "utf8"), text);
      inputs.push({ file: relative, ...identify(copied) });
    }
  const manifest = jsonValue(
    fs.readFileSync(path.join(project, "node_modules/vize/package.json"), "utf8"),
  );
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  assert.deepEqual(manifest.typescript.contentMapper.exec, [binary, "content-mapper"]);
  const wrapperRoot = path.join(out, "source-candidate-tool");
  fs.mkdirSync(wrapperRoot);
  const wrapper = path.join(wrapperRoot, "tsgo");
  fs.writeFileSync(wrapper, wrapperSource, { mode: 0o755 });
  const observations: {
    mode: string;
    diagnostics: (Row | Diagnostic)[];
    nativeRequests: string[];
  }[] = [];
  for (const mode of ["cli", "mapper"]) {
    const capture = path.join(out, "source-candidate-native", mode);
    fs.mkdirSync(capture, { recursive: true });
    const env = {
      ...process.env,
      VIZE_DIRECTIVE_NATIVE: checker,
      VIZE_DIRECTIVE_NATIVE_CAPTURE: capture,
    };
    const result =
      mode === "cli"
        ? run(
            out,
            "source-candidate-cli",
            binary,
            [
              "check",
              "--no-config",
              "--format",
              "json",
              "--show-virtual-ts",
              "--tsconfig",
              "tsconfig.json",
              "--corsa-path",
              wrapper,
            ],
            project,
            env,
          )
        : run(
            out,
            "source-candidate-mapper",
            wrapper,
            [
              "--runExternalCode",
              "--checkers",
              "1",
              "--noEmit",
              "--pretty",
              "false",
              "--project",
              "tsconfig.mapper.json",
            ],
            project,
            env,
          );
    const parsed = mode === "cli" ? packet(result.stdout, result.status, project) : null;
    if (parsed)
      for (const row of rows) {
        const file = parsed.result.files.find(
          (file) =>
            path.relative(project, path.resolve(project, file.file)).replaceAll("\\", "/") ===
            row.id + "/src/Tiny.vue",
        );
        assert.ok(file && typeof file.virtualTs === "string" && file.virtualTs.length > 0);
      }
    const diagnostics = parsed?.diagnostics ?? native(result.stdout, result.status, project);
    const requests = fs.readdirSync(capture);
    assert.ok(requests.length > 0);
    observations.push({ mode, diagnostics, nativeRequests: requests });
  }
  const byCase = rows.map((row) => ({
    id: row.id,
    requiresAuthoredError: row.requiresAuthoredError,
    official: official.filter((d) => d.file.startsWith(row.id + "/")),
    routes: observations.map((r) => ({
      mode: r.mode,
      diagnostics: r.diagnostics.filter((d) => d.file.startsWith(row.id + "/")),
    })),
  }));
  const missingAuthored: { id: string; mode: string; missing: Row[] }[] = [];
  for (const row of byCase)
    for (const route of row.routes) {
      const counts = new Map<string, number>();
      for (const d of route.diagnostics) counts.set(key(d), (counts.get(key(d)) ?? 0) + 1);
      const missing = [];
      for (const d of row.official) {
        const n = counts.get(key(d)) ?? 0;
        if (n > 0) counts.set(key(d), n - 1);
        else missing.push(d);
      }
      if (missing.length) missingAuthored.push({ id: row.id, mode: route.mode, missing });
    }
  const boundaries = boundaryEvidence(originalProject, provider, out, rows);
  const report = {
    sourceImage,
    nativeImage,
    sourceSdkManifest: manifest,
    inputs,
    byCase,
    boundaries,
    missingAuthored,
    timingUsable: false,
    releaseQualified: false,
    originalConservationGate: "RED and untouched",
    knownBaselineIssues: [
      "casing-kebab-invalid authored prop is still missed by the old candidate",
      "overload-last-zero has an existing generated TS2322 false positive",
    ],
  };
  save(path.join(out, "source-candidate-summary.json"), report);
  assert.deepEqual(identify(binary), sourceImage);
  assert.deepEqual(identify(checker), nativeImage);
  assert.ok(
    missingAuthored.every((row) => row.id === "casing-kebab-invalid"),
    "all independent authored errors except the separately recorded casing P0 must remain",
  );
  const required = byCase.find((row) => row.id === "required-public-internal");
  assert.ok(required);
  for (const route of required.routes)
    assert.ok(route.diagnostics.some((d) => d.code === 2345 && d.file.endsWith("/src/Tiny.vue")));
  return report;
}
