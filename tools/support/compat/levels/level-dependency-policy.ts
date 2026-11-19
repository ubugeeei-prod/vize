import { execFileSync } from "node:child_process";

export const policyPath = "docs/davinci/plan/level-dependency-allowlist.json";
export interface AllowlistKey {
  readonly from: string;
  readonly to: string;
  readonly rename: string | null;
  readonly target: string | null;
  readonly optional: boolean;
  readonly workspace: boolean;
}
export interface AllowlistEntry extends AllowlistKey {
  readonly roots: readonly string[];
  readonly issue?: number;
  readonly reason?: string;
}
export interface Allowlist {
  readonly schema?: string;
  readonly version?: number;
  readonly baselineRevision?: string;
  readonly entries: readonly AllowlistEntry[];
}

export const forbidden = (name: string): boolean =>
  /^(?:vize_armature|vize_relief|vize_croquis|vize_croquis_cf|vize_atelier_.*)$/u.test(name);
const bootstrapRevision = "f59e69c38ecbead394ba30f0fdacb5f9c1b9fd04";

export const initialEntries: readonly AllowlistEntry[] = [
  {
    from: "vize_l1",
    to: "vize_armature",
    rename: null,
    target: null,
    optional: false,
    workspace: true,
    roots: ["vize_l1", "vize_l1_to_l2"],
  },
  {
    from: "vize_l1",
    to: "vize_relief",
    rename: null,
    target: null,
    optional: false,
    workspace: true,
    roots: ["vize_l1", "vize_l1_to_l2"],
  },
  {
    from: "vize_l1_to_l2",
    to: "vize_relief",
    rename: null,
    target: null,
    optional: false,
    workspace: true,
    roots: ["vize_l1_to_l2"],
  },
];
export const key = ({ from, to, rename, target, optional, workspace }: AllowlistKey): string =>
  JSON.stringify([from, to, rename, target, optional, workspace]);
export function requireEvidence(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export function validateAllowlist(policy: unknown): Allowlist {
  requireEvidence(
    isRecord(policy) && policy.schema === "vize.level-dependency-allowlist" && policy.version === 1,
    "invalid dependency allowlist schema",
  );
  requireEvidence(
    typeof policy.baselineRevision === "string" && /^[0-9a-f]{40}$/u.test(policy.baselineRevision),
    "missing allowlist baseline revision",
  );
  requireEvidence(
    Object.keys(policy).sort().join(",") === "baselineRevision,entries,schema,version",
    "unknown or missing allowlist fields",
  );
  requireEvidence(Array.isArray(policy.entries), "missing dependency allowlist entries");
  const entries: AllowlistEntry[] = [];
  const seen = new Set<string>();
  for (const entry of policy.entries as readonly unknown[]) {
    requireEvidence(isRecord(entry), "invalid dependency allowlist entry");
    requireEvidence(
      Object.keys(entry).sort().join(",") ===
        "from,issue,optional,reason,rename,roots,target,to,workspace",
      "unknown or missing allowlist entry fields",
    );
    requireEvidence(
      typeof entry.from === "string" &&
        entry.from.length > 0 &&
        typeof entry.to === "string" &&
        forbidden(entry.to),
      "invalid allowlist package identity",
    );
    requireEvidence(
      entry.rename === null || (typeof entry.rename === "string" && entry.rename.length > 0),
      "invalid allowlist rename",
    );
    requireEvidence(
      entry.target === null || (typeof entry.target === "string" && entry.target.length > 0),
      "invalid allowlist target",
    );
    requireEvidence(typeof entry.optional === "boolean", "missing allowlist optional flag");
    requireEvidence(
      entry.workspace === true,
      "only measured workspace legacy entries may be allowed",
    );
    requireEvidence(
      Array.isArray(entry.roots) &&
        entry.roots.length > 0 &&
        entry.roots.every((root: unknown) => typeof root === "string" && root.length > 0),
      "invalid allowlist roots",
    );
    requireEvidence(new Set(entry.roots).size === entry.roots.length, "duplicate allowlist root");
    requireEvidence(
      typeof entry.issue === "number" &&
        Number.isSafeInteger(entry.issue) &&
        entry.issue > 0 &&
        typeof entry.reason === "string" &&
        entry.reason.trim().length > 0,
      "missing allowlist removal tracking",
    );
    const typed: AllowlistEntry = {
      from: entry.from,
      to: entry.to,
      rename: entry.rename,
      target: entry.target,
      optional: entry.optional,
      workspace: entry.workspace,
      roots: entry.roots as string[],
      issue: entry.issue,
      reason: entry.reason,
    };
    requireEvidence(!seen.has(key(typed)), "duplicate dependency allowlist entry");
    seen.add(key(typed));
    entries.push(typed);
  }
  return {
    schema: policy.schema,
    version: policy.version,
    baselineRevision: policy.baselineRevision,
    entries,
  };
}

export function assertAllowlistRatchet(current: unknown, previous: Allowlist): void {
  const checked = validateAllowlist(current);
  requireEvidence(
    checked.baselineRevision === (previous.baselineRevision ?? bootstrapRevision),
    "allowlist baseline revision must stay fixed",
  );
  const prior = new Map(previous.entries.map((entry) => [key(entry), entry]));
  for (const entry of checked.entries) {
    const old = prior.get(key(entry));
    requireEvidence(old, "dependency allowlist may only shrink: new entry " + key(entry));
    for (const root of entry.roots)
      requireEvidence(
        old.roots.includes(root),
        "dependency allowlist may only shrink: new root " + root,
      );
  }
}

export function readBaseAllowlist(cwd: string, base: string | undefined): Allowlist {
  requireEvidence(
    base !== undefined && /^[0-9a-f]{40}$/u.test(base),
    "expected full comparison base SHA",
  );
  execFileSync("git", ["cat-file", "-e", base + "^{commit}"], { cwd, stdio: "pipe" });
  const present = execFileSync("git", ["ls-tree", "--name-only", base, "--", policyPath], {
    cwd,
    encoding: "utf8",
  }).trim();
  if (present)
    return validateAllowlist(
      JSON.parse(execFileSync("git", ["show", base + ":" + policyPath], { cwd, encoding: "utf8" })),
    );
  const previousGate = execFileSync(
    "git",
    // The gate moved from .mjs to .ts; either spelling at the base counts.
    [
      "ls-tree",
      "--name-only",
      base,
      "--",
      "tools/support/compat/davinci/level-dependencies.mjs",
      "tools/support/compat/davinci/level-dependencies.ts",
    ],
    { cwd, encoding: "utf8" },
  ).trim();
  requireEvidence(!previousGate, "comparison base gate exists but allowlist is missing");
  // First adoption is bounded by the reviewed release candidate, never by head input.
  return { entries: initialEntries };
}
