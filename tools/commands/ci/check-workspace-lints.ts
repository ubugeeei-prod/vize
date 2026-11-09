import { readFileSync } from "node:fs";
import { isEntrypoint } from "../../support/command-runtime.ts";

const pending: readonly string[] = [];
const upstreamMembers = new Set([
  "upstream/forks/oxc_parser",
  "upstream/forks/oxc_formatter",
  "upstream/forks/oxlint_html_renderer186",
  "upstream/forks/lightningcss",
  "upstream/forks/parcel_sourcemap",
]);

export function workspaceMembers(source: string): string[] {
  const start = source.indexOf("members = [");
  if (start < 0) return [];
  const body = source.slice(start + "members = [".length).split("]", 1)[0] ?? "";
  return body
    .split(",")
    .map((entry) => entry.trim())
    .flatMap((entry) => (entry.startsWith('"') && entry.endsWith('"') ? [entry.slice(1, -1)] : []));
}

export function optsIntoWorkspaceLints(source: string): boolean {
  let inLints = false;
  for (const line of source.split(/\r?\n/u).map((value) => value.trim())) {
    if (line.startsWith("[")) {
      inLints = line === "[lints]";
      continue;
    }
    if (inLints && line.replaceAll(" ", "") === "workspace=true") return true;
  }
  return false;
}

export function checkWorkspaceLints(arguments_: readonly string[]): number {
  const check = arguments_.includes("--check");
  const members = workspaceMembers(readFileSync("Cargo.toml", "utf8")).filter(
    (member) => !upstreamMembers.has(member),
  );
  if (members.length === 0) throw new Error("no workspace members found in Cargo.toml");

  const problems: string[] = [];
  let joined = 0;
  for (const member of members) {
    const optedIn = optsIntoWorkspaceLints(readFileSync(`${member}/Cargo.toml`, "utf8"));
    const isPending = pending.includes(member);
    if (optedIn && isPending) {
      problems.push(`${member} opted into the workspace lints; remove it from PENDING`);
    } else if (!optedIn && !isPending) {
      problems.push(`${member} must add \`[lints] workspace = true\` (or be listed in PENDING)`);
    } else if (optedIn) {
      joined += 1;
    }
  }
  for (const member of pending) {
    if (!members.includes(member))
      problems.push(`${member} is in PENDING but is not a workspace member`);
  }

  console.log(
    `workspace lints: ${joined}/${members.length} members joined, ${members.length - joined} pending`,
  );
  if (problems.length === 0 || !check) {
    for (const problem of problems) console.log(`  ${problem}`);
    return 0;
  }
  console.error(problems.join("\n"));
  return 1;
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = checkWorkspaceLints(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
