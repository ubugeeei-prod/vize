/** Exact original #11023 inputs; distinct from the later 26-case campaign. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { slotCases } from "../../../../tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/original-cases.ts";

export const original27Root = fileURLToPath(
  new URL(
    "../../../../tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/",
    import.meta.url,
  ),
);
export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
const json = (value: unknown) => JSON.stringify(value, null, 2) + "\n";

export function original27Inputs() {
  const driver = fs.readFileSync(path.join(original27Root, "original-cases.ts"));
  assert.equal(driver.length, 5155);
  assert.equal(sha256(driver), "07bf14866c4debd560d5e928d46069d843a17a3b0f3b27ec27f78466768de232");
  assert.equal(
    createHash("sha1").update(`blob ${driver.length}\0`).update(driver).digest("hex"),
    "d34ce4bf37ea26ac8a9120c8bd6f66c79346d86f",
  );
  const historicalBytes = fs.readFileSync(
    path.join(original27Root, "historical-public-summary.json"),
  );
  assert.equal(
    sha256(historicalBytes),
    "85fe4aefd51ee457e93628b6f6c2e4c9a34979086eab861610da5b9acd0d943e",
  );
  const historical = JSON.parse(historicalBytes.toString()) as {
    manifest: { file: string; bytes: number; sha256: string }[];
    byCase: {
      id: string;
      requiresAuthoredError: boolean;
      official: unknown[];
      mapper: unknown[];
      cli: unknown[];
    }[];
    identities: { file: string; bytes: number; sha256: string }[];
    hDiagnostics: unknown[];
    checker: { bytes: number; sha256: string };
    stock: { bytes: number; sha256: string };
    vueTsc: { bytes: number; sha256: string };
    hSource: string;
  };
  const rows = slotCases();
  assert.equal(rows.length, 20);
  const zero = rows.find((row) => row.id === "zero")!;
  rows.push({
    id: "required-public-internal",
    component: zero.component.replace(
      "props: { value: { type: Number, required: true },",
      "props: { internal: { type: String, required: true }, value: { type: Number, required: true },",
    ),
    source: zero.source,
    requiresAuthoredError: true,
  });
  const nativeComponent = zero.component.replace(
    "export declare const WitnessChild: { new(): Instance };",
    "export const WitnessChild = VueChild;",
  );
  for (const [id, binding, tag] of [
    ["pascal", "WitnessChild", "WitnessChild"],
    ["camel", "witnessChild", "witnessChild"],
    ["kebab", "witnessChild", "witness-child"],
  ]) {
    for (const stage of ["valid", "invalid"]) {
      let source = zero.source
        .replace("import WitnessChild from", "import " + binding + " from")
        .replaceAll("<WitnessChild", "<" + tag)
        .replaceAll("</WitnessChild", "</" + tag);
      if (stage === "invalid") source = source.replace(':value="1"', ":value=\"'bad'\"");
      rows.push({
        id: `casing-${id}-${stage}`,
        component: nativeComponent,
        source,
        requiresAuthoredError: stage === "invalid",
      });
    }
  }
  const files = new Map<string, string>();
  for (const row of rows) {
    files.set(`${row.id}/component.ts`, row.component);
    files.set(`${row.id}/src/Tiny.vue`, row.source);
  }
  const config = {
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ES2024",
      lib: ["ES2024", "DOM"],
      types: [],
      incremental: false,
    },
    vueCompilerOptions: {
      strictTemplates: true,
      checkUnknownProps: true,
      checkUnknownComponents: false,
    },
    include: ["**/*.vue", "**/component.ts"],
  };
  const h = ["import { h, type Component } from 'vue';"];
  for (const [index, row] of rows.entries()) {
    if (
      ![
        "zero",
        "constructor-public-props-discrepancy",
        "constructor-public-type-discrepancy",
        "required-public-internal",
      ].includes(row.id)
    )
      continue;
    h.push(
      `import C${index} from './${row.id}/component';`,
      `const component${index}:Component=C${index};`,
      `const props${index}:InstanceType<typeof C${index}>['$props']={value:1};`,
      `h(C${index}, props${index});`,
      `void component${index};`,
    );
  }
  files.set("authored_h.ts", h.join("\n") + "\n");
  files.set("tsconfig.json", json(config));
  files.set(
    "tsconfig.mapper.json",
    json({ ...config, contentMappers: [{ package: "vize", extensions: [".vue"] }] }),
  );
  files.set("tsconfig.h.json", json({ ...config, include: ["authored_h.ts", "**/component.ts"] }));
  const manifest = [...files].map(([file, body]) => ({
    file,
    bytes: Buffer.byteLength(body),
    sha256: sha256(body),
  }));
  assert.equal(rows.length, 27);
  assert.equal(files.size, 58);
  assert.deepEqual(manifest, historical.manifest);
  assert.deepEqual(
    rows.map(({ id, requiresAuthoredError }) => ({ id, requiresAuthoredError })),
    historical.byCase.map(({ id, requiresAuthoredError }) => ({ id, requiresAuthoredError })),
  );
  assert.equal(files.get("authored_h.ts"), historical.hSource);
  return { rows, files, manifest, historical, historicalSummarySha256: sha256(historicalBytes) };
}

export function writeOriginal27(project: string) {
  const inputs = original27Inputs();
  assert.equal(fs.existsSync(project), false);
  for (const [file, body] of inputs.files) {
    const target = path.join(project, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, body);
  }
  return inputs;
}
