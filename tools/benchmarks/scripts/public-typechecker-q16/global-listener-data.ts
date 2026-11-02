import { boundedBytes } from "../../../support/release/release/quota/http.ts";
import { publicUrl, sriSha512 } from "../../../support/release/public_install/registry.ts";
import type { Recorder } from "./execute.ts";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString, type JsonRecord } from "../../../support/json-value.ts";
import { sha256, type Profile, type SourceFile } from "./contract.ts";

export const globalListenerFileNames = [
  "global-listener-files.json",
  "global-listener-controls.json",
];
export const globalListenerFix = "044768e5e35cedf030da532d78fe3c8f8a056310";
export const globalListenerArgs = ["--runExternalCode", "-p", "tsconfig.json", "--pretty", "false"];
export const globalListenerReferenceArgs = ["--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
export interface ListenerDiagnostic {
  file: string;
  code: number;
  sourceNeedle: string;
  message: string | null;
  missingRequiredProperty: string | null;
}
export interface ListenerState {
  id: string;
  files: SourceFile[];
  status: number;
  diagnostics: ListenerDiagnostic[];
}
function resource(name: string, expected: string) {
  const raw = readFileSync(new URL(name, import.meta.url));
  assert.equal(sha256(raw), expected, "source-owned listener data differs");
  return asRecord(JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)));
}
export function globalListenerData() {
  const files = resource(
      globalListenerFileNames[0]!,
      "b1ad879fa39f205930bc193a288ea524b3afd4221bcdfbf484da1859bb694bec",
    ),
    controls = resource(
      globalListenerFileNames[1]!,
      "6c9c81af01bd72cb78e9c3fa2eea51e782ecca0aec23f13939a08c0a9decbda6",
    );
  assert.equal(files.schema, "vize.globalListener11373.files.v1");
  assert.equal(controls.schema, "vize.globalListener11373.controls.v1");
  assert.equal(controls.fix, globalListenerFix);
  const images = new Map<string, string>();
  for (const value of asArray(files.files)) {
    const image = asRecord(value),
      path = asString(image.path),
      text = asString(image.utf8);
    assert(!images.has(path));
    assert.equal(Buffer.byteLength(text), image.bytes);
    assert.equal(sha256(text), image.sha256);
    images.set(path, text);
  }
  assert.equal(images.size, 22);
  const source = (name: unknown) => {
    const text = images.get(asString(name));
    assert(text !== undefined);
    return text;
  };
  const baseline: SourceFile[] = asArray(controls.baseline).map((value) => {
    const input = asRecord(value);
    return { path: asString(input.file), source: source(input.source) };
  });
  assert.deepEqual(
    baseline.map((f) => f.path),
    [
      "package.json",
      "tsconfig.json",
      "src/global-components.d.ts",
      "src/GlobalComponentListener.vue",
      "src/PascalListener.vue",
      "src/NoListener.vue",
    ],
  );
  const states: ListenerState[] = asArray(controls.controls).map((value) => {
    const row = asRecord(value),
      inputs = new Map(baseline.map((f) => [f.path, f.source]));
    for (const item of asArray(row.overrides)) {
      const edit = asRecord(item);
      inputs.set(asString(edit.file), source(edit.source));
    }
    const diagnostics = asArray(row.diagnostics).map((value) => {
      const d = asRecord(value);
      assert([2322, 2339, 2345].includes(Number(d.code)));
      return {
        file: asString(d.file),
        code: Number(d.code),
        sourceNeedle: asString(d.sourceNeedle),
        message: d.message === undefined ? null : asString(d.message),
        missingRequiredProperty:
          d.missingRequiredProperty === undefined ? null : asString(d.missingRequiredProperty),
      };
    });
    assert.equal(diagnostics.length, row.success === true ? 0 : 1);
    for (const d of diagnostics)
      if (d.missingRequiredProperty !== null) {
        assert.equal(row.id, "missing-required-global-prop");
        assert.equal(d.code, 2345);
        assert.equal(d.missingRequiredProperty, "to");
        assert.equal(d.message, null);
      } else assert(d.message !== null);
    return {
      id: asString(row.id),
      files: [...inputs].map(([path, source]) => ({ path, source })),
      status: row.success === true ? 0 : 2,
      diagnostics,
    };
  });
  assert.deepEqual(
    states.map((s) => s.id),
    [
      "original-three-components",
      "wrong-global-prop",
      "missing-required-global-prop",
      "typed-event-bad",
      "typed-event-good",
      "local-owned-number",
      "local-owned-string-rejected",
      "unregistered-default",
      "unregistered-explicit-check",
      "listener-absent-original-control",
      "listener-restored-exact-original",
      "authored-expect-error-marked",
      "quoted-marker-unmarked",
      "authored-expect-error-sibling",
    ],
  );
  assert.deepEqual(states[0]!.files, states[10]!.files);
  return { states, sourceAuthority: controls.sourceAuthority, images };
}
export function globalListenerFiles(state: ListenerState, profile: Profile) {
  assert.equal(profile.subset, "GlobalListener11373");
  return state.files.map((file) => {
    if (file.path !== "package.json") return { ...file };
    const needle = '"vize": "0.640.0"';
    assert.equal(file.source.split(needle).length, 2);
    return {
      path: file.path,
      source: file.source.replace(needle, '"vize": "' + profile.version + '"'),
    };
  });
}
export function listenerNeedle(state: ListenerState, diagnostic: ListenerDiagnostic) {
  const file = state.files.find((f) => f.path === diagnostic.file);
  assert(file);
  const start = file.source.indexOf(diagnostic.sourceNeedle);
  assert(start >= 0);
  assert.equal(file.source.indexOf(diagnostic.sourceNeedle, start + 1), -1);
  assert(!diagnostic.sourceNeedle.includes("\n"));
  const prefix = file.source.slice(0, start),
    line = prefix.split("\n").length,
    column = prefix.length - prefix.lastIndexOf("\n");
  return { line, startColumn: column, endColumnExclusive: column + diagnostic.sourceNeedle.length };
}

export const p73FullMetadataUrl = "https://registry.npmjs.org/vize";
export const p73FullMetadataAccept = "application/json";
export function usesP73FullPackument(
  profile: Profile,
  name: string,
  version: string,
  capsuleEntry: JsonRecord,
) {
  return (
    profile.subset === "GlobalListener11373" &&
    profile.version === "0.653.0" &&
    name === "vize" &&
    version === "0.653.0" &&
    capsuleEntry.metadataOrigin === p73FullMetadataUrl
  );
}
export function selectP73FullMetadata(value: unknown, lock: JsonRecord, capsuleEntry: JsonRecord) {
  const packument = asRecord(value);
  assert.equal(packument.name, "vize");
  const versions = asRecord(packument.versions);
  assert(Object.hasOwn(versions, "0.653.0"));
  const selected = asRecord(versions["0.653.0"]);
  assert.equal(selected.name, "vize");
  assert.equal(selected.version, "0.653.0");
  assert.equal(lock.version, "0.653.0");
  assert.equal(capsuleEntry.metadataOrigin, p73FullMetadataUrl);
  const dist = asRecord(selected.dist);
  sriSha512(dist.integrity);
  publicUrl(asString(dist.tarball));
  assert.equal(dist.integrity, lock.integrity);
  assert.equal(dist.integrity, capsuleEntry.integrity);
  assert.equal(dist.tarball, lock.resolved);
  assert.equal(dist.tarball, capsuleEntry.tarball);
  assert.equal(
    publicUrl(asString(asRecord(dist.attestations).url)),
    "https://registry.npmjs.org/-/npm/v1/attestations/vize@0.653.0",
  );
  return selected;
}
export async function readP73FullMetadata(
  profile: Profile,
  lock: JsonRecord,
  capsuleEntry: JsonRecord,
  record: Pick<Recorder, "raw" | "json">,
  cache: Map<string, Buffer>,
  fetchImpl: typeof fetch = globalThis.fetch,
) {
  assert(usesP73FullPackument(profile, "vize", asString(lock.version), capsuleEntry));
  const key = "P73_653_FULL_PACKUMENT_RAW:" + p73FullMetadataUrl;
  let raw = cache.get(key);
  const cached = raw !== undefined;
  if (raw)
    record.json("vize-full-packument.http.json", {
      url: p73FullMetadataUrl,
      accept: p73FullMetadataAccept,
      source: "SAME_RUN_SHARED_RAW_CACHE",
    });
  else {
    const response = await fetchImpl(p73FullMetadataUrl, {
      redirect: "error",
      signal: AbortSignal.timeout(15_000),
      headers: {
        Accept: p73FullMetadataAccept,
        "User-Agent": "VizeReleaseVerification/1 (https://github.com/ubugeeei-prod/vize)",
      },
    });
    record.json("vize-full-packument.http.json", {
      url: response.url,
      requestedUrl: p73FullMetadataUrl,
      accept: p73FullMetadataAccept,
      status: response.status,
      date: response.headers.get("date"),
      contentType: response.headers.get("content-type"),
      source: "ACTUAL_HTTP_RESPONSE",
    });
    raw = await boundedBytes(response, 4 * 1024 * 1024);
    record.raw("vize-full-packument.raw.json", raw);
    assert.equal(response.url, p73FullMetadataUrl);
    assert.equal(response.status, 200);
    cache.set(key, raw);
  }
  assert(raw.length <= 4 * 1024 * 1024);
  if (cached) record.raw("vize-full-packument.raw.json", raw);
  const selected = selectP73FullMetadata(
    JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)),
    lock,
    capsuleEntry,
  );
  record.json("vize-full-packument-selection.json", {
    url: p73FullMetadataUrl,
    accept: p73FullMetadataAccept,
    rawBytes: raw.length,
    rawSha256: sha256(raw),
    selectedVersion: "0.653.0",
    selectedEntry: selected,
    representation: "EXACT_FULL_PACKUMENT_MEMBER",
    canonicalVersionHttpStatusClaimed: false,
  });
  return selected;
}
