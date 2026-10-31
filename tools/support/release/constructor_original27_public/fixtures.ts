import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord, readJsonRecord } from "../../json-value.ts";
import { fileImage, sha256 } from "../computed_event_custody.ts";

export const fixtureDriver = "a8ce1ec166a53b380e74dbf7189cc8578c80c676";
export const fixturePins = {
  "tools/support/compat/content-mapper/constructor-overload-original27.ts":
    "82cc3a035d24680de2b6e1ba376f42ac69f071dfc6aa1d77a040bee640ea6e30",
  "tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/original-cases.ts":
    "07bf14866c4debd560d5e928d46069d843a17a3b0f3b27ec27f78466768de232",
  "tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/historical-public-summary.json":
    "85fe4aefd51ee457e93628b6f6c2e4c9a34979086eab861610da5b9acd0d943e",
  "tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/provider-package.json":
    "57e15c4d13056f3d059a818d36df2ae0d23f6ca35c46bf29b656f62cdb2b18cc",
} as const;
export const fixturePath =
  "tests/_fixtures/differential/typechecker/constructor-overload-11023-original27";
export type OriginalRow = {
  id: string;
  component: string;
  source: string;
  requiresAuthoredError: boolean;
};
export type Inputs = {
  rows: OriginalRow[];
  files: Map<string, string>;
  manifest: { file: string; bytes: number; sha256: string }[];
  historical: {
    identities: { file: string; bytes: number; sha256: string }[];
    checker: { bytes: number; sha256: string };
    stock: { bytes: number; sha256: string };
    vueTsc: { bytes: number; sha256: string };
  };
};
export async function loadOriginal27(driverRoot: string) {
  const root = realpathSync(driverRoot);
  const custody = Object.entries(fixturePins).map(([file, expected]) => {
    const image = fileImage(join(root, file));
    assert.equal(image.sha256, expected, file);
    return image;
  });
  const module = asRecord(
    await import(pathToFileURL(join(root, Object.keys(fixturePins)[0])).href),
  );
  assert.equal(typeof module.original27Inputs, "function");
  const inputs = Reflect.apply(
    module.original27Inputs as () => Inputs,
    undefined,
    [],
  );
  assert.equal(inputs.rows.length, 27);
  assert.equal(inputs.files.size, 58);
  assert.equal(
    inputs.rows.filter((row) => row.requiresAuthoredError).length,
    14,
  );
  const provider = readJsonRecord(
    join(root, fixturePath, "provider-package.json"),
  );
  return { root, custody, inputs, provider };
}
export function installedManifest(
  provider: Record<string, unknown>,
  version: string,
) {
  const manifest = structuredClone(provider);
  const dependencies = asRecord(manifest.dependencies);
  assert.equal(dependencies.vize, "0.602.0");
  assert.equal(dependencies["@vizejs/native"], "0.602.0");
  dependencies.vize = version;
  dependencies["@vizejs/native"] = version;
  return manifest;
}
export function assertInputs(project: string, files: Map<string, string>) {
  return [...files].map(([file, expected]) => {
    const raw = readFileSync(join(project, file));
    assert.equal(raw.toString("utf8"), expected, file);
    return { file, bytes: raw.length, sha256: sha256(raw) };
  });
}
