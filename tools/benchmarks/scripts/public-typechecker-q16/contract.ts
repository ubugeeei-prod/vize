import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { overrides } from "../../../support/release/public_install/identity.ts";
import { suppliedSubsets } from "./fixtures.ts";
import { declarationData } from "./declaration-data.ts";

export type Subset = "D61" | "B60-68" | "A59" | "C62" | "C4" | "A64";
export type Role = "after";
export interface SourceFile {
  path: string;
  source: string;
}
export interface Expected {
  status: number;
  stdout: string;
  stderr: string;
}
export interface State {
  id: string;
  phase: string;
  files: SourceFile[];
  expectedTsrs: Expected;
}
export interface Project {
  id: string;
  issue: number;
  states: State[];
}
export interface SuppliedSubset {
  id: Subset;
  projects: Project[];
  sourceAuthority: unknown;
}
interface PublishedProfile {
  version: string;
  source: string;
  candidate: string;
  run: string;
  capsule: string;
}
export type RegressionProfile = PublishedProfile & { subset: Exclude<Subset, "A64">; fix: string };
export type DeclarationProfile = PublishedProfile & {
  subset: "A64";
  knownGoodSource: string;
  causalFix: null;
};
export type Profile = RegressionProfile | DeclarationProfile;
// Only the parent can bind a genuine published containing release in a narrow source successor.
export const afterProfile: Profile | null = null;
export const sha256 = (raw: string | Uint8Array) => createHash("sha256").update(raw).digest("hex");
export const limitations = {
  scope:
    "Fresh installed AFTER-only selected Q16 subset; retained source Before is not public remeasurement",
  host: "Linux x64 Node26.13.0 adaptation; reporter Windows11/architecture and exact npm version are not claimed",
  manager: "Declared npm11.19.1 control; original npm i and npx checker argv preserved",
  selectedCompilerPid: "UNKNOWN",
  selectedCompilerImage: "UNKNOWN",
  sourceObservationsArePublicOutputs: false,
  wholeNodeModulesFilesetQualified: false,
};
export const reportedCommands = {
  install: ["npm", "i"],
  tsrs: ["npx", "tsrs", "--runExternalCode", "--noEmit", "-p", "tsconfig.json"],
  reference: ["npx", "vue-tsc", "--noEmit", "-p", "tsconfig.json"],
} as const;
export function subsetFor(id: string): SuppliedSubset {
  assert(
    ["D61", "B60-68", "A59", "C62", "C4"].includes(id),
    "closed source-qualified subset required",
  );
  const rows: SuppliedSubset[] = suppliedSubsets;
  const matches = rows.filter((row) => row.id === id);
  assert.equal(matches.length, 1, "subset source data has not been supplied");
  return matches[0]!;
}
export function requireProfile(profile: Profile | null): Profile {
  assert(profile !== null, "genuine containing AFTER release is UNKNOWN");
  if (profile.subset === "A64") {
    assert.equal(profile.causalFix, null);
    assert(!Object.hasOwn(profile, "fix"), "A64 has no causal fix claim");
    assert.equal(profile.knownGoodSource, declarationData.knownGoodSource);
  } else subsetFor(profile.subset);
  for (const sha of [
    profile.subset === "A64" ? profile.knownGoodSource : profile.fix,
    profile.source,
    profile.candidate,
    profile.capsule,
  ])
    assert(/^[a-f0-9]{40}$/u.test(sha), "literal source binding required");
  assert(/^0\.\d+\.0$/u.test(profile.version) && /^\d+$/u.test(profile.run));
  return profile;
}
export function profileFor(_role: Role = "after"): Profile {
  return requireProfile(afterProfile);
}
export function filesFor(state: State, profile: Profile): SourceFile[] {
  return state.files.map((file) => {
    assert(/^(package\.json|tsconfig\.json|src\/[A-Za-z0-9_.-]+)$/u.test(file.path));
    if (file.path !== "package.json") return { ...file };
    assert.equal(file.source.split('"vize": "0.638.0"').length, 2);
    return {
      path: file.path,
      source: file.source.replace('"vize": "0.638.0"', '"vize": "' + profile.version + '"'),
    };
  });
}
export function requireEnvironment(
  environment: NodeJS.ProcessEnv,
  record: (rows: { name: string; present: boolean; empty: boolean }[]) => void = () => {},
) {
  const names = [
    ...new Set([
      ...overrides,
      "TSRS_BINARY",
      "TSRS_PATH",
      "TSRS_EXECUTABLE",
      "VIZE_PUBLIC_NATIVE_CUSTODY",
      "VIZE_OXLINT_NATIVE_CUSTODY",
      "NODE_PATH",
    ]),
  ];
  const rows = names.map((name) => ({
    name,
    present: Object.hasOwn(environment, name),
    empty: environment[name] === "",
  }));
  record(rows);
  assert(
    rows.every((row) => !row.present),
    "initial runtime override presence refuses without value egress",
  );
  return rows;
}
