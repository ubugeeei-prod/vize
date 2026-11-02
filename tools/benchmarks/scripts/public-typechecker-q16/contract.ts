import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { overrides } from "../../../support/release/public_install/identity.ts";
import { suppliedSubsets } from "./fixtures.ts";
import { declarationData } from "./declaration-data.ts";
import { a65Data } from "./a-data.ts";

export type Subset =
  | "D61"
  | "B60-68"
  | "A59"
  | "A59-65"
  | "C62"
  | "C4"
  | "A64"
  | "E90-91"
  | "F78"
  | "R77"
  | "GArrowTsc";
export type Role = "after";
export interface SourceFile {
  path: string;
  source: string;
}
export interface Expected {
  status: number;
  stdout: string;
  stderr: string;
  stdoutKind?: "OWNED_HEADER_OPAQUE_PRINTER";
  sourceInputSha256?: string;
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
export const afterProfile: Profile | null = {
  subset: "GArrowTsc",
  fix: "ba01953b4429fabdd652d3497d89a8cf1737697b",
  version: "0.652.0",
  source: "ba01953b4429fabdd652d3497d89a8cf1737697b",
  candidate: "6a9b36fbb8c21b0d928583e3732410f5c2afb12a",
  run: "41523763531",
  capsule: "b567cd114991ae7a9d3b3f6a398bf78246e50182",
};
export const sha256 = (raw: string | Uint8Array) => createHash("sha256").update(raw).digest("hex");
const defaultLimitations = {
  scope:
    "Fresh installed AFTER-only selected Q16 subset; retained source Before is not public remeasurement",
  host: "Linux x64 Node26.13.0 adaptation; reporter Windows11/architecture and exact npm version are not claimed",
  manager: "Declared npm11.19.1 control; original npm i and npx checker argv preserved",
  selectedCompilerPid: "UNKNOWN",
  selectedCompilerImage: "UNKNOWN",
  sourceObservationsArePublicOutputs: false,
  wholeNodeModulesFilesetQualified: false,
};
export function limitationsFor(subset: Subset) {
  if (subset === "GArrowTsc")
    return {
      ...defaultLimitations,
      scope:
        "Fresh installed TSC generic-arrow reproducer with authored package/config/argv completion",
      host: "Linux x64 Node26.13.0 adaptation; reported M4/macOS15.6.1/Node22.23.3 is not claimed",
      manager: "npm11.19.1/npm i control; controlled Node invokes the authentic installed bin/tsc",
    };
  return subset === "R77"
    ? {
        ...defaultLimitations,
        scope:
          "Fresh installed AFTER-only static template ref11377; source-native six-state observations are not public outputs",
        host: "Linux x64 Node26.13.0 adaptation; reported macOS26.4.1 arm64 Node26.12.1 is not claimed",
        manager:
          "npm11.19.1/npm i/npx adaptation of reported pnpm12.3.4 add/exec; exactly three reported direct dependencies",
      }
    : defaultLimitations;
}
export const limitations = limitationsFor((afterProfile as Profile | null)?.subset ?? "D61");
export const reportedCommands = {
  install: ["npm", "i"],
  tsrs: ["npx", "tsrs", "--runExternalCode", "--noEmit", "-p", "tsconfig.json"],
  reference: ["npx", "vue-tsc", "--noEmit", "-p", "tsconfig.json"],
} as const;
export function commandsFor(subset: Subset) {
  if (subset === "R77")
    return {
      install: ["npm", "i"],
      tsrs: ["npx", "tsrs", "--runExternalCode", "-p", "tsconfig.json", "--pretty", "false"],
      reference: null,
    } as const;
  if (subset === "E90-91")
    return {
      install: ["npm", "i"],
      tsrs: ["npx", "tsrs", "--fast", "--runExternalCode", "-p", "tsconfig.json"],
      reference: null,
    } as const;
  if (subset === "F78")
    return {
      install: ["npm", "i"],
      tsrs: ["npx", "tsrs", "--runExternalCode", "-p", "tsconfig.json", "--pretty", "false"],
      reference: ["npx", "vue-tsc", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
    } as const;
  return reportedCommands;
}
export function subsetFor(id: string): SuppliedSubset {
  assert(
    ["D61", "B60-68", "A59", "C62", "C4", "E90-91", "F78", "R77"].includes(id),
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
  } else if (profile.subset === "A59-65") {
    subsetFor("A59");
    assert.equal(profile.fix, a65Data.fixMerge);
  } else if (profile.subset === "GArrowTsc") {
    assert.equal(profile.fix, "ba01953b4429fabdd652d3497d89a8cf1737697b");
  } else {
    subsetFor(profile.subset);
    if (profile.subset === "R77")
      assert.equal(profile.fix, "56635d7cfb27718ec8b40e67309c71da5b9a3803");
    if (profile.subset === "F78")
      assert.equal(profile.fix, "d1ef6f2200afaf18922a052c114bb9ed366846e5");
  }
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
    const before =
        profile.subset === "E90-91"
          ? "0.645.0"
          : ["F78", "R77"].includes(profile.subset)
            ? "0.641.0"
            : "0.638.0",
      needle = '"vize": "' + before + '"';
    assert.equal(file.source.split(needle).length, 2);
    return {
      path: file.path,
      source: file.source.replace(needle, '"vize": "' + profile.version + '"'),
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
