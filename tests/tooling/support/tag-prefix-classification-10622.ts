import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { readPinnedArtifact } from "../../differential/harness.mjs";
import { offsetToPosition } from "./lsp/assertions.ts";
import { root } from "./lsp/paths.ts";

export const fixtureDirectory = path.join(
  root,
  "tests/_fixtures/differential/lsp/tag-prefix-classification-10622",
);
export const inputHashes: Record<string, string> = {
  "App.vue.txt": "d7b232bc676a52ae0d4913cf12a96fa08cd63193dad7287e9bf38e91d9dcce6b",
  "Collision.vue.txt": "5a320e1766295bc56296a893d4fec95305c9044fe5704065435208ac0aff2f6b",
  "Child.vue.txt": "9712927066b83ae5504c30aab2ba7f7dd28fc36e8c256afe6a136829a31d2bfb",
  "globals.d.ts.txt": "dd55a47184be69699d01b557e7ee135b4efbf1257b9e73700de000a92d718087",
};
export const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const input = (name: string) =>
  readPinnedArtifact(fixtureDirectory, { path: name, sha256: inputHashes[name] }).toString("utf8");

export type PrefixQuery = {
  name: string;
  needle: string;
  inside?: number;
  requestedBindings: string[];
};
export type PrefixProfile = {
  name: string;
  file: "App.vue.txt" | "Collision.vue.txt";
  newline: "lf" | "crlf";
  vue: boolean;
  historicalCandidates: number;
  queries: PrefixQuery[];
};
const mainQueries: PrefixQuery[] = [
  { name: "pascal-global-overwrite", needle: "<Comp7", requestedBindings: ["Comp7"] },
  { name: "kebab", needle: "<comp7", requestedBindings: ["Comp7"] },
  { name: "false-component", needle: "<Fake", requestedBindings: ["FakeCard"] },
  { name: "global-only", needle: "<Global", requestedBindings: [] },
  { name: "zero-match", needle: "<NoSuch", requestedBindings: [] },
  { name: "other-component", needle: "<Other", requestedBindings: ["OtherCard"] },
  {
    name: "empty-prefix",
    needle: "<Comp7",
    inside: 1,
    requestedBindings: ["Comp7", "FakeCard", "OtherCard"],
  },
  { name: "unchanged-repeat", needle: "<Comp7", requestedBindings: ["Comp7"] },
];
const collisionQueries: PrefixQuery[] = [
  { name: "shared-kebab", needle: "<foo-b", requestedBindings: ["FooBar", "fooBar"] },
  { name: "pascal-identity", needle: "<FooB", requestedBindings: ["FooBar"] },
  { name: "camel-identity", needle: "<fooB", requestedBindings: ["fooBar"] },
  {
    name: "empty-prefix",
    needle: "<FooBar",
    inside: 1,
    requestedBindings: ["FooBar", "fooBar"],
  },
];
export const profiles: PrefixProfile[] = [
  ...(["lf", "crlf"] as const).flatMap((newline) => [
    {
      name: `native-${newline}`,
      file: "App.vue.txt" as const,
      newline,
      vue: true,
      historicalCandidates: 3,
      queries: mainQueries,
    },
    {
      name: `collision-${newline}`,
      file: "Collision.vue.txt" as const,
      newline,
      vue: true,
      historicalCandidates: 2,
      queries: collisionQueries,
    },
  ]),
  {
    name: "missing-vue-default-fallback",
    file: "App.vue.txt",
    newline: "lf",
    vue: false,
    historicalCandidates: 3,
    queries: [mainQueries[0], mainQueries[2], mainQueries[6]],
  },
];

export function preparePrefixWorkspace(
  profile: PrefixProfile,
  runtime: string,
  vuePackage: string,
  evidence: string,
) {
  // A missing-Vue control must not inherit the repository's node_modules.
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tag-prefix-10622-"));
  fs.mkdirSync(path.join(workspace, "src"));
  fs.mkdirSync(evidence, { recursive: true });
  const files: Record<string, string> = {
    "src/App.vue": input(profile.file),
    "src/Child.vue": input("Child.vue.txt"),
    "src/globals.d.ts": input("globals.d.ts.txt"),
    "package.json": '{"private":true,"type":"module"}\n',
    "tsconfig.json":
      JSON.stringify({
        compilerOptions: {
          strict: true,
          module: "ESNext",
          moduleResolution: "bundler",
          target: "ES2022",
          skipLibCheck: true,
          noEmit: true,
        },
        include: ["src/**/*"],
      }) + "\n",
    "vite.config.mjs": `export default ${JSON.stringify({
      vue: {
        languageServer: { lint: false, typecheck: true, editor: true, ecosystem: false },
        typeChecker: { corsaPath: runtime },
      },
    })};\n`,
  };
  if (profile.newline === "crlf") {
    for (const name of Object.keys(files).filter((name) => name.startsWith("src/")))
      files[name] = files[name].replaceAll("\n", "\r\n");
  }
  for (const [name, bytes] of Object.entries(files)) {
    fs.writeFileSync(path.join(workspace, name), bytes);
    const retained = path.join(evidence, "inputs", name);
    fs.mkdirSync(path.dirname(retained), { recursive: true });
    fs.writeFileSync(retained, bytes);
  }
  if (profile.vue) {
    const modules = path.join(workspace, "node_modules");
    fs.mkdirSync(modules);
    const real = fs.realpathSync(vuePackage);
    const kind = process.platform === "win32" ? "junction" : "dir";
    fs.symlinkSync(real, path.join(modules, "vue"), kind);
    fs.symlinkSync(path.join(path.dirname(real), "@vue"), path.join(modules, "@vue"), kind);
  }
  const source = files["src/App.vue"];
  const uri = pathToFileURL(path.join(workspace, "src/App.vue")).href;
  fs.writeFileSync(
    path.join(evidence, "inputs.json"),
    JSON.stringify(
      {
        profile,
        workspace,
        files: Object.entries(files).map(([name, bytes]) => ({
          path: name,
          bytes: Buffer.byteLength(bytes),
          sha256: sha256(bytes),
        })),
      },
      null,
      2,
    ) + "\n",
  );
  return { workspace, source, uri };
}

export function prefixParams(source: string, uri: string, query: PrefixQuery) {
  const at = source.indexOf(query.needle);
  assert.notEqual(at, -1, query.name);
  return {
    textDocument: { uri },
    position: offsetToPosition(source, at + (query.inside ?? query.needle.length)),
  };
}

export type NativeCapture = {
  file: string;
  sha256: string;
  records: Array<{ stage: string; value: unknown }>;
};
export function nativeCaptures(directory: string): NativeCapture[] {
  if (!fs.existsSync(directory)) return [];
  return fs
    .readdirSync(directory)
    .filter((name) => name.endsWith(".json"))
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .map((file) => {
      const bytes = fs.readFileSync(path.join(directory, file));
      const decoded = JSON.parse(bytes.toString("utf8")) as { records: NativeCapture["records"] };
      assert.equal(Array.isArray(decoded.records), true, file);
      return { file, sha256: sha256(bytes), records: decoded.records };
    });
}
