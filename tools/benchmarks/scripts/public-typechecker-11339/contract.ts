import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { originalFiles, wrappedSupplement } from "./original.ts";
export type Role = "before" | "after";
export interface Profile {
  version: string;
  source: string;
  candidate: string;
  run: string;
  capsule: string;
}
export const beforeProfile: Profile = {
  version: "0.635.0",
  source: "ae65bd3add7768081384a44a72aaad23a9fc2947",
  candidate: "bd2ca765c6a474c6489c6ddf32bfbb36db5f4cd6",
  run: "41322907495",
  capsule: "9b9242d60d7121df1b8454697d1eec37e2d16309",
};
// A genuine containing release must be source-frozen here before feature dispatch.
export const afterProfile: Profile | null = null;
export function profileFor(role: Role): Profile {
  const value = role === "before" ? beforeProfile : afterProfile;
  assert(value, "AFTER release is UNKNOWN; do not execute or guess a candidate");
  return value;
}
export const sha256 = (raw: string | Buffer) => createHash("sha256").update(raw).digest("hex");
export const vueVersions = ["3.4.23", "3.6.0-beta.10"] as const;
export type VueVersion = (typeof vueVersions)[number];
export const cases = ["Original", "Repair", "RestoreOriginal", "WrappedSupplement"] as const;
export type Case = (typeof cases)[number];
export type SourceFile = { path: string; source: string };
export const reporter = {
  issue: 11339,
  login: "dmnlk",
  gitAuthor: "Masaki Kawaguchi <m-kawaguchi@binc.jp>",
  bodySha256: "4511e7cf9df0a09ae43f37eef3e4f02a450334de76bf0a3f8dbf4dc44de56cc0",
  sourceBodies: 3,
  fences: 5,
} as const;
export function input(vue: VueVersion): SourceFile[] {
  assert(vueVersions.includes(vue));
  for (const file of originalFiles) assert.equal(sha256(file.source), file.sha256);
  return [
    {
      path: "package.json",
      source:
        JSON.stringify(
          {
            name: "authored-11339-minimal-consumer",
            private: true,
            type: "module",
            devDependencies: {
              vize: beforeProfile.version,
              vue,
              typescript: "6.0.3",
              "vue-tsc": "3.0.8",
            },
          },
          null,
          2,
        ) + "\n",
    },
    {
      path: "tsconfig.json",
      source:
        JSON.stringify(
          {
            compilerOptions: {
              strict: true,
              target: "ES2022",
              module: "ESNext",
              moduleResolution: "bundler",
              noEmit: true,
            },
            include: ["src/**/*"],
          },
          null,
          2,
        ) + "\n",
    },
    ...originalFiles.map(({ path, source }) => ({ path, source })),
  ];
}
export function sources(original: SourceFile[], role: Role, id: Case): SourceFile[] {
  const version = profileFor(role).version;
  const files = original.map((file) => {
    let source = file.source;
    if (file.path === "package.json") {
      assert.equal(source.split('"vize": "0.635.0"').length, 2);
      source = source.replace('"vize": "0.635.0"', '"vize": "' + version + '"');
    }
    if (file.path === "src/Invalid.vue" && id === "Repair") {
      const invalid =
        '  <p :title="greet(1)">{{ isPC.toFixed() }} {{ currentDevice.toFixed() }} {{ fromBase.toUpperCase() }} {{ missing }}</p>';
      assert.equal(source.split(invalid).length, 2);
      source = source.replace(
        invalid,
        "  <p :title=\"greet('x')\">{{ isPC ? 'pc' : 'sp' }} {{ currentDevice.toUpperCase() }} {{ fromBase.toFixed() }}</p>",
      );
    }
    return { path: file.path, source };
  });
  if (id === "WrappedSupplement")
    files.push({ path: "src/Wrapped.vue", source: wrappedSupplement });
  return files;
}
export const originalFacts = [
  {
    file: "src/Invalid.vue",
    line: 2,
    column: 20,
    code: "TS2345",
    message: "Argument of type 'number' is not assignable to parameter of type 'string'.",
  },
  {
    file: "src/Invalid.vue",
    line: 2,
    column: 32,
    code: "TS2339",
    message: "Property 'toFixed' does not exist on type 'boolean'.",
  },
  {
    file: "src/Invalid.vue",
    line: 2,
    column: 62,
    code: "TS2551",
    message: "Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
  },
  {
    file: "src/Invalid.vue",
    line: 2,
    column: 87,
    code: "TS2339",
    message: "Property 'toUpperCase' does not exist on type 'number'.",
  },
  { file: "src/Invalid.vue", line: 2, column: 107, code: "TS2339", message: null },
] as const;
export const wholePublicOracle = null;
export const limitations = {
  project:
    "AUTHORED_MINIMAL_HARNESS; reporter supplied three source files and prose settings, no full package.json",
  provider: "DEFAULT_INSTALLED_DISCOVERY; no invented native alias/CORSA override",
  selectedCorsaBuild: "UNKNOWN",
  selectedOriginalCliPid: null,
  selectedOriginalBackendPid: null,
  reportedMacNodePnpmEquivalence: false,
  exactReportedCompilerClaimed: false,
  wholePublicOracleOwned: false,
} as const;
