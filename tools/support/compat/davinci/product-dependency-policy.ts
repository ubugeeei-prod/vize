import { execFileSync } from "node:child_process";
import {
  type AllowlistEntry,
  key,
  requireEvidence,
  validateAllowlist,
} from "./level-dependency-policy.ts";

export interface ProductAllowlist {
  readonly schema: "vize.product-dependency-allowlist";
  readonly version: 1;
  readonly baselineRevision: string;
  readonly root: string;
  readonly entries: readonly AllowlistEntry[];
}

export const initialGlyphProductAllowlist: ProductAllowlist = {
  schema: "vize.product-dependency-allowlist",
  version: 1,
  baselineRevision: "d9bab8d2708f66e53606b5ad29e42f2a471de51d",
  root: "vize_glyph",
  entries: [
    {
      from: "vize_glyph",
      to: "vize_atelier_sfc",
      rename: null,
      target: null,
      optional: false,
      workspace: true,
      roots: ["vize_glyph"],
      issue: 6851,
      reason: "Remove Glyph's legacy SFC compiler dependency after its L1 formatting route lands.",
    },
  ],
};

export const initialPatinaProductAllowlist: ProductAllowlist = {
  schema: "vize.product-dependency-allowlist",
  version: 1,
  baselineRevision: "b7515d2e7ccc8589eb6cd8366a21c515c0656f51",
  root: "vize_patina",
  entries: [
    ["vize_canon", "vize_armature", "Remove Patina's inherited Canon-to-Armature path."],
    ["vize_canon", "vize_atelier_core", "Remove Patina's inherited Canon-to-Atelier Core path."],
    ["vize_canon", "vize_atelier_jsx", "Remove Patina's inherited Canon-to-Atelier JSX path."],
    ["vize_canon", "vize_atelier_sfc", "Remove Patina's inherited Canon-to-Atelier SFC path."],
    ["vize_canon", "vize_croquis", "Remove Patina's inherited Canon-to-Croquis path."],
    ["vize_canon", "vize_relief", "Remove Patina's inherited Canon-to-Relief path."],
    ["vize_patina", "vize_armature", "Remove Patina's direct legacy Armature dependency."],
    ["vize_patina", "vize_atelier_jsx", "Remove Patina's direct legacy Atelier JSX dependency."],
    ["vize_patina", "vize_atelier_sfc", "Remove Patina's direct legacy Atelier SFC dependency."],
    ["vize_patina", "vize_croquis", "Remove Patina's direct legacy Croquis dependency."],
    ["vize_patina", "vize_relief", "Remove Patina's direct legacy Relief dependency."],
  ].map(([from, to, reason]) => ({
    from,
    to,
    rename: null,
    target: null,
    optional: false,
    workspace: true,
    roots: ["vize_patina"],
    issue: 6851,
    reason,
  })),
};

export const productPolicies = [
  {
    path: "docs/davinci/plan/product-dependency-allowlists/vize_glyph.json",
    initial: initialGlyphProductAllowlist,
  },
  {
    path: "docs/davinci/plan/product-dependency-allowlists/vize_patina.json",
    initial: initialPatinaProductAllowlist,
  },
] as const;

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export function validateProductAllowlist(value: unknown): ProductAllowlist {
  requireEvidence(
    isRecord(value) && value.schema === "vize.product-dependency-allowlist" && value.version === 1,
    "invalid product dependency allowlist schema",
  );
  requireEvidence(
    Object.keys(value).sort().join(",") === "baselineRevision,entries,root,schema,version",
    "unknown or missing product allowlist fields",
  );
  requireEvidence(
    typeof value.baselineRevision === "string" && /^[0-9a-f]{40}$/u.test(value.baselineRevision),
    "missing product allowlist baseline revision",
  );
  requireEvidence(
    typeof value.root === "string" && /^vize_(?:patina|glyph|maestro|canon)$/u.test(value.root),
    "invalid protected product root",
  );
  const entries = validateAllowlist({
    schema: "vize.level-dependency-allowlist",
    version: 1,
    baselineRevision: value.baselineRevision,
    entries: value.entries,
  }).entries;
  requireEvidence(
    entries.every((entry) => entry.roots.length === 1 && entry.roots[0] === value.root),
    "product permissions must belong only to their protected root",
  );
  return {
    schema: value.schema,
    version: value.version,
    baselineRevision: value.baselineRevision,
    root: value.root,
    entries,
  };
}

export function assertProductAllowlistRatchet(current: unknown, previous: ProductAllowlist): void {
  const checked = validateProductAllowlist(current);
  requireEvidence(
    checked.baselineRevision === previous.baselineRevision && checked.root === previous.root,
    "product dependency baseline and root must stay fixed",
  );
  const prior = new Set(previous.entries.map(key));
  for (const entry of checked.entries)
    requireEvidence(
      prior.has(key(entry)),
      "product dependency allowlist may only shrink: " + key(entry),
    );
}

export function combineProductAllowlists(policies: readonly ProductAllowlist[]): {
  roots: string[];
  entries: AllowlistEntry[];
} {
  const roots = new Set<string>(),
    entries = new Map<string, AllowlistEntry>();
  for (const input of policies) {
    const policy = validateProductAllowlist(input);
    requireEvidence(!roots.has(policy.root), "duplicate product dependency root: " + policy.root);
    roots.add(policy.root);
    for (const entry of policy.entries) {
      const id = key(entry),
        current = entries.get(id);
      if (current)
        entries.set(id, { ...current, roots: [...current.roots, policy.root].sort() });
      else entries.set(id, { ...entry, roots: [policy.root] });
    }
  }
  return { roots: [...roots].sort(), entries: [...entries.values()] };
}

export function readBaseProductAllowlist(
  cwd: string,
  base: string | undefined,
  policyPath: string,
  initial: ProductAllowlist,
): ProductAllowlist {
  requireEvidence(
    base !== undefined && /^[0-9a-f]{40}$/u.test(base),
    "expected full product comparison base SHA",
  );
  requireEvidence(
    policyPath === `docs/davinci/plan/product-dependency-allowlists/${initial.root}.json`,
    "product dependency policy path does not match its root",
  );
  execFileSync("git", ["cat-file", "-e", base + "^{commit}"], { cwd, stdio: "pipe" });
  const present = execFileSync("git", ["ls-tree", "--name-only", base, "--", policyPath], {
    cwd,
    encoding: "utf8",
  }).trim();
  if (!present) return initial;
  return validateProductAllowlist(
    JSON.parse(
      execFileSync("git", ["show", base + ":" + policyPath], { cwd, encoding: "utf8" }),
    ),
  );
}
