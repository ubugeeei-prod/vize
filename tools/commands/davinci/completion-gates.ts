// Davinci completion gates: the one executable definition of "Davinci complete".
// Definitions: docs/davinci/completion-gates.md. Usage:
//   node tools/commands/davinci/completion-gates.ts [--json] [--issues]
// `--issues` asks GitHub (gh) for issue state; without it issue gates print "unknown".
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type Tier = 0 | 1 | 2 | 3 | 4 | 5 | 6;
export type Status = "pass" | "fail" | "unknown";

export interface Repo {
  readonly paths: readonly string[];
  read(file: string): string | null;
}

export interface Measured {
  readonly pass: boolean;
  readonly detail: string;
}

export interface Gate {
  readonly id: string;
  readonly tier: Tier;
  readonly issues: readonly number[];
  readonly measure?: (repo: Repo) => Measured;
}

export const levelCrates = [
  "vize_l0",
  "vize_l0_derive",
  "vize_l1",
  "vize_l1_to_l2",
  "vize_l2",
  "vize_l2_to_l3",
  "vize_l3",
  "vize_l4",
] as const;

export const legacyCrates = [
  "vize_armature",
  "vize_relief",
  "vize_croquis",
  "vize_croquis_cf",
  "vize_atelier_core",
  "vize_atelier_dom",
  "vize_atelier_ssr",
  "vize_atelier_vapor",
  "vize_atelier_sfc",
  "vize_atelier_jsx",
] as const;

export const codenames = [
  "sinopia",
  "disegno",
  "ricalco",
  "impeto",
  "stesura",
  "folio",
  "spolvero",
  "davinci",
] as const;

const codenamePattern = new RegExp(codenames.join("|"), "gi");
// Captured output of a past run is evidence, not a name the code produces.
const capturePattern =
  /\.(log|jsonl|stderr|stdout)$|(^|\/)observed-[^/]*\.json$|\.observation\.json$/;

// The gate's definition must spell the names it bans.
const toolRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const gateFiles = new Set([
  path
    .relative(toolRoot, fileURLToPath(import.meta.url))
    .split(path.sep)
    .join("/"),
  "tools/support/levels/rename-program-codename.ts",
]);

const exemptPath = (file: string) =>
  file.startsWith("docs/davinci/") || gateFiles.has(file) || capturePattern.test(file);

// Other docs may keep the level codenames as aliases, but not the program name.
const counts = (file: string, name: string) =>
  !file.startsWith("docs/") || name.toLowerCase() === "davinci";

export interface CodenameReport {
  readonly paths: number;
  readonly lines: number;
  readonly byName: Readonly<Record<string, number>>;
}

export const scanCodenames = (repo: Repo): CodenameReport => {
  const byName: Record<string, number> = {};
  const hit = (name: string) => {
    const key = name.toLowerCase();
    byName[key] = (byName[key] ?? 0) + 1;
  };
  let paths = 0,
    lines = 0;
  const matches = (file: string, text: string) =>
    (text.match(codenamePattern) ?? []).filter((name) => counts(file, name));
  for (const file of repo.paths) {
    if (exemptPath(file)) continue;
    const named = matches(file, file);
    if (named.length) {
      paths++;
      named.forEach(hit);
    }
    const text = repo.read(file);
    if (text === null) continue;
    for (const line of text.split("\n")) {
      // `//! L3 - reactivity IR (codename: Impeto)` is the allowed alias form for level
      // codenames; the program name has none.
      const alias = /codename/i.test(line);
      const found = matches(file, line).filter(
        (name) => !alias || name.toLowerCase() === "davinci",
      );
      if (found.length) {
        lines++;
        found.forEach(hit);
      }
    }
  }
  return { paths, lines, byName };
};

export const workspaceMembers = (cargoToml: string): string[] => {
  const block = /\bmembers\s*=\s*\[([^\]]*)\]/.exec(cargoToml);
  if (!block) throw new Error("Cargo.toml has no workspace members");
  return [...block[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
};

const requireText = (repo: Repo, file: string) => {
  const text = repo.read(file);
  if (text === null) throw new Error("missing " + file);
  return text;
};

const members = (repo: Repo) => workspaceMembers(requireText(repo, "Cargo.toml"));

const allowlistEntries = (repo: Repo, file: string): number => {
  const value: unknown = JSON.parse(requireText(repo, file));
  const entries = (value as { entries?: unknown }).entries;
  if (!Array.isArray(entries)) throw new Error(file + " has no entries array");
  return entries.length;
};

const measureLevelCrates = (repo: Repo): Measured => {
  const all = members(repo);
  const home = all.find((member) => path.basename(member) === "vize_l0");
  if (!home) throw new Error("no vize_l0 workspace member");
  const found = all
    .filter((member) => path.dirname(member) === path.dirname(home))
    .map((member) => path.basename(member))
    .sort();
  const expected = [...levelCrates].sort();
  const extra = found.filter((name) => !expected.includes(name as never));
  const missing = expected.filter((name) => !found.includes(name));
  return {
    pass: extra.length === 0 && missing.length === 0,
    detail: `extra [${extra.join(", ")}], missing [${missing.join(", ")}]`,
  };
};

const measureLevelDependencies = (repo: Repo): Measured => {
  const count = allowlistEntries(repo, "docs/davinci/plan/level-dependency-allowlist.json");
  return { pass: count === 0, detail: `${count} allowlisted legacy edges` };
};

const measureSkeleton = (repo: Repo): Measured => {
  let modules = 0;
  for (const file of repo.paths) {
    if (!file.endsWith(".rs") || file.startsWith("tools/")) continue;
    if (/reason\s*=\s*"skeleton:/.test(repo.read(file) ?? "")) modules++;
  }
  return { pass: modules === 0, detail: `${modules} skeleton todo!() modules` };
};

const measureCodenames = (repo: Repo): Measured => {
  const report = scanCodenames(repo);
  const names = Object.entries(report.byName)
    .sort(([, a], [, b]) => b - a)
    .map(([name, count]) => `${name} ${count}`)
    .join(", ");
  return {
    pass: report.paths + report.lines === 0,
    detail: `${report.paths} paths, ${report.lines} lines (${names})`,
  };
};

const measureRustOnlyFormal = (repo: Repo): Measured => {
  const lean = repo.paths.filter(
    (file) =>
      file.endsWith(".lean") ||
      /(^|\/)(lakefile\.(lean|toml)|lean-toolchain|lake-manifest\.json)$/.test(file) ||
      (file.startsWith(".github/workflows/") && /(^|[-_.])lean([-_.]|$)/.test(path.basename(file))),
  );
  return { pass: lean.length === 0, detail: `${lean.length} Lean files or workflows` };
};

const productAllowlistDir = "docs/davinci/plan/product-dependency-allowlists/";

const measureProductDependencies = (repo: Repo): Measured => {
  const files = repo.paths.filter(
    (file) => file.startsWith(productAllowlistDir) && file.endsWith(".json"),
  );
  if (files.length === 0) throw new Error("no product dependency allowlists");
  const counts = files.map(
    (file) => `${path.basename(file, ".json")} ${allowlistEntries(repo, file)}`,
  );
  const total = files.reduce((sum, file) => sum + allowlistEntries(repo, file), 0);
  return { pass: total === 0, detail: `${total} legacy edges (${counts.join(", ")})` };
};

const measureStabilityPolicy = (repo: Repo): Measured => {
  const policy = JSON.parse(
    requireText(repo, "docs/davinci/plan/deletion-readiness-policy.json"),
  ) as { minimumStableDays?: unknown; maximumGapDays?: unknown };
  const positive = (value: unknown) =>
    typeof value === "number" && Number.isInteger(value) && value > 0;
  const { minimumStableDays: days, maximumGapDays: gap } = policy;
  return {
    pass: positive(days) && positive(gap),
    detail: `minimumStableDays ${String(days)}, maximumGapDays ${String(gap)}`,
  };
};

const measureLegacyDeleted = (repo: Repo): Measured => {
  const left = members(repo)
    .map((member) => path.basename(member))
    .filter((name) => legacyCrates.includes(name as never));
  return { pass: left.length === 0, detail: `${left.length} legacy crates: ${left.join(", ")}` };
};

// Tier order is the dependency order of the work; see the doc's "Order of work".
export const gates: readonly Gate[] = [
  { id: "levels.crates", tier: 0, issues: [6833], measure: measureLevelCrates },
  { id: "levels.dependencies", tier: 0, issues: [6831], measure: measureLevelDependencies },
  { id: "levels.skeleton", tier: 0, issues: [6826], measure: measureSkeleton },
  { id: "levels.codenames", tier: 0, issues: [6832], measure: measureCodenames },
  { id: "formal.rust-only", tier: 0, issues: [10825], measure: measureRustOnlyFormal },
  { id: "ci.tiers", tier: 0, issues: [6830] },
  { id: "harness.differential", tier: 0, issues: [6891, 6892, 6893, 6884, 6890] },
  { id: "history.fixtures", tier: 0, issues: [6879, 6880, 6881, 6883] },
  { id: "levels.markup", tier: 1, issues: [6835, 6843] },
  { id: "levels.container", tier: 1, issues: [6837] },
  { id: "levels.dialects", tier: 1, issues: [6841, 6842] },
  { id: "levels.l2", tier: 2, issues: [6838] },
  { id: "levels.query", tier: 2, issues: [6871] },
  { id: "levels.script", tier: 2, issues: [6844] },
  { id: "levels.jsx", tier: 2, issues: [6885] },
  { id: "levels.l3", tier: 3, issues: [6839] },
  { id: "levels.l4", tier: 3, issues: [6840] },
  { id: "products.formatter", tier: 4, issues: [6847] },
  { id: "products.linter", tier: 4, issues: [6848] },
  { id: "products.typechecker", tier: 4, issues: [6849] },
  { id: "products.lsp", tier: 4, issues: [6850, 6872] },
  { id: "products.project", tier: 4, issues: [6874] },
  { id: "products.fixes", tier: 4, issues: [6876] },
  { id: "products.dependencies", tier: 4, issues: [6851], measure: measureProductDependencies },
  { id: "inputs.pug", tier: 4, issues: [8908] },
  { id: "inputs.sass", tier: 4, issues: [11550] },
  { id: "inputs.scss", tier: 4, issues: [11551] },
  { id: "inputs.less", tier: 4, issues: [11552] },
  { id: "inputs.stylus", tier: 4, issues: [11553] },
  { id: "inputs.jsx", tier: 4, issues: [11554] },
  { id: "inputs.tsx", tier: 4, issues: [11555] },
  { id: "inputs.vapor", tier: 4, issues: [11556] },
  { id: "legacy.zero-fallback", tier: 5, issues: [6854] },
  { id: "legacy.acceptance", tier: 5, issues: [6853] },
  { id: "legacy.stability", tier: 5, issues: [6828], measure: measureStabilityPolicy },
  { id: "legacy.deleted", tier: 5, issues: [6828], measure: measureLegacyDeleted },
  { id: "inputs.tsrx", tier: 6, issues: [6858] },
  { id: "poc.svelte", tier: 6, issues: [11540] },
  { id: "poc.astro", tier: 6, issues: [11557] },
  { id: "poc.angular", tier: 6, issues: [11558] },
  { id: "poc.solid", tier: 6, issues: [6859] },
  { id: "poc.moonbit", tier: 6, issues: [11559] },
  { id: "poc.coffeescript", tier: 6, issues: [11560] },
  { id: "poc.python", tier: 6, issues: [11561] },
];

const issueList = (issues: readonly number[]) => issues.map((issue) => "#" + issue).join(" ");

export interface Result {
  readonly id: string;
  readonly tier: Tier;
  readonly status: Status;
  readonly detail: string;
  readonly openIssues: readonly number[] | null;
}

export const evaluate = (
  repo: Repo,
  issueStates: ReadonlyMap<number, "OPEN" | "CLOSED"> | null,
  selected: readonly Gate[] = gates,
): Result[] =>
  selected.map((gate) => {
    const measured = gate.measure?.(repo) ?? null;
    const openIssues = issueStates
      ? gate.issues.filter((issue) => {
          const state = issueStates.get(issue);
          if (!state) throw new Error(`no state for #${issue}`);
          return state === "OPEN";
        })
      : null;
    // A measured gate passes on its measurement; an issue gate when its issues are closed.
    const status: Status = measured
      ? measured.pass
        ? "pass"
        : "fail"
      : openIssues === null
        ? "unknown"
        : openIssues.length === 0
          ? "pass"
          : "fail";
    return {
      id: gate.id,
      tier: gate.tier,
      status,
      detail:
        measured?.detail ??
        (openIssues === null
          ? "issues " + issueList(gate.issues)
          : openIssues.length === 0
            ? "issues closed"
            : "open " + issueList(openIssues)),
      openIssues,
    };
  });

export const gitRepo = (root: string): Repo => {
  const paths = execFileSync("git", ["ls-files", "-z"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 1 << 28,
  })
    .split("\0")
    .filter(Boolean);
  return {
    paths,
    read(file) {
      let bytes: Buffer;
      try {
        bytes = fs.readFileSync(path.join(root, file));
      } catch {
        return null; // deleted in the worktree or a submodule directory
      }
      return bytes.subarray(0, 8000).includes(0) ? null : bytes.toString("utf8");
    },
  };
};

const fetchIssueStates = (root: string): Map<number, "OPEN" | "CLOSED"> => {
  const numbers = [...new Set(gates.flatMap((gate) => gate.issues))];
  const fields = numbers.map((n) => `i${n}: issue(number: ${n}) { state }`).join(" ");
  const query = `{ repository(owner: "ubugeeei-prod", name: "vize") { ${fields} } }`;
  const raw = execFileSync("gh", ["api", "graphql", "-f", "query=" + query], {
    cwd: root,
    encoding: "utf8",
  });
  const data = (JSON.parse(raw) as { data: { repository: Record<string, { state: string }> } }).data
    .repository;
  return new Map(
    numbers.map((n) => {
      const state = data["i" + n]?.state;
      if (state !== "OPEN" && state !== "CLOSED") throw new Error(`#${n}: state ${state}`);
      return [n, state];
    }),
  );
};

if (import.meta.main) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
  const args = new Set(process.argv.slice(2));
  const results = evaluate(gitRepo(root), args.has("--issues") ? fetchIssueStates(root) : null);
  if (args.has("--json")) {
    process.stdout.write(JSON.stringify(results, null, 2) + "\n");
  } else {
    for (const result of results) {
      process.stdout.write(
        `T${result.tier} ${result.status.padEnd(7)} ${result.id.padEnd(22)} ${result.detail}\n`,
      );
    }
    const passed = results.filter((result) => result.status === "pass").length;
    process.stdout.write(`${passed}/${results.length} gates pass\n`);
  }
}
