#!/usr/bin/env node
/** Replay the #6832 removal of the stage feed codename from paths, names and wire strings. */
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
// Spelled in parts so this script never matches itself.
const old = ["spol", "vero"].join("");
const Old = ["Spol", "vero"].join("");
const schema = "docs/davinci/plan/stage-ladder-feed.schema.json";

export const moves: readonly (readonly [string, string])[] = [
  [`crates/vize_curator/tests/${old}_ladder.rs`, "crates/vize_curator/tests/stage_ladder.rs"],
  [
    `crates/vize_curator/tests/${old}_payload.rs`,
    "crates/vize_curator/tests/stage_feed_payload.rs",
  ],
  [
    `crates/vize_curator/tests/${old}_timing.rs`,
    "crates/vize_curator/tests/stage_ladder_timing.rs",
  ],
  [
    `crates/vize_vitrine/src/wasm/tests_${old}.rs`,
    "crates/vize_vitrine/src/wasm/tests_stage_feed.rs",
  ],
  [
    `crates/vize_vitrine/src/wasm/tests_${old}_profile.rs`,
    "crates/vize_vitrine/src/wasm/tests_stage_profile.rs",
  ],
  [`docs/davinci/plan/${old}-feed.schema.json`, schema],
];

type Rule = readonly [string | RegExp, string];

// Paths, identifiers and the profile span key; longer spellings first.
const names: readonly Rule[] = [
  [`${old}-feed.schema.json`, "stage-ladder-feed.schema.json"],
  [`levels.${old}.step`, "stages.step"],
  [`${old.toUpperCase()}_FEED_SCHEMA_VERSION`, "STAGE_FEED_SCHEMA_VERSION"],
  [`negotiate${Old}Feed`, "negotiateStageFeed"],
  [`tests_${old}_profile`, "tests_stage_profile"],
  [`tests_${old}`, "tests_stage_feed"],
  [`${old}_value_with_remarks`, "stage_feed_value_with_remarks"],
  [`${old}_value`, "stage_feed_value"],
  [`payload_${old}`, "payload_stage_feed"],
  [`${old}_members`, "stage_members"],
  [`include_${old}_remarks`, "include_stage_remarks"],
  [`${old}_profile`, "stage_profile"],
  [`${old}_ladder`, "stage_ladder"],
  [`${old}_payload`, "stage_feed_payload"],
  [`${old}_timing`, "stage_ladder_timing"],
  [`${old}Profile`, "stageProfile"],
];

// The JSON member, where quoted or URL-encoded, then prose.
const wire: readonly Rule[] = [
  [`%22${old}%22`, "%22stageFeed%22"],
  [`"${old}"`, '"stageFeed"'],
  [`\`${old}\` feed`, "`stageFeed` member"],
  [`\`${old}\``, "`stageFeed`"],
  [`\`(${old}, `, "`(stageFeed, "],
  [`[^\\n]*${Old} feed payload/`, "[^\\n]*--test stage_feed_payload/"],
  [`"title": "Levels ${Old} feed"`, '"title": "Levels stage ladder feed"'],
  [`schema_version ${Old} negotiates`, "schema_version the stage view negotiates"],
  [
    new RegExp(
      `Producers: \`level-opt --dump-dir\` writes it as \`${old}\\.json\` beside the pages \\(${Old}Feed::of_dump[^)]*\\), the inspector payload embeds it under \`stageFeed\`, and the wasm \`analyzeSfc\` result carries`,
      "u",
    ),
    "Producers: the inspector payload embeds it under `stageFeed`, and the wasm `analyzeSfc` result carries",
  ],
  [`${Old} stage-ladder feed`, "Stage-ladder feed"],
  [`${Old} stage ladder`, "stage ladder"],
  [`${Old} ladder`, "stage ladder"],
  [`${Old} members`, "stage members"],
  [`${Old} consumers`, "stage-view consumers"],
  [`${Old}'s timing`, "The stage view's timing"],
  [`${Old} in the browser`, "the stage view in the browser"],
  [`JSON ${Old} and agents`, "JSON the stage view and agents"],
  [`LSP and ${Old}`, "LSP and the stage view"],
  [`artifact ${Old} reads`, "artifact the stage view reads"],
  [`${Old} renders`, "the stage view renders"],
  [`a ${old} transfer`, "a pounced transfer"],
  [`preset's\n// ${Old} feed`, "preset's\n// stage feed"],
  [new RegExp(`(["\`]|^//!? )${Old} feed`, "gmu"), "$1Stage feed"],
  [`${Old} feed`, "stage feed"],
];

// Frozen #10900 custody (sha-pinned before-bytes and the projection that reads them)
// and the completion gate and codename rename, which must spell the names they ban.
const frozen = new Set([
  "tools/support/cross_file/async_component/public-before.json.txt",
  "tools/support/cross_file/async_component/verify.ts",
  "tools/commands/levels/completion-gates.ts",
  "tools/support/levels/rename-codenames.ts",
]);
// Live indexes in docs that cite the moved files; other docs keep historical bytes.
const indexes = new Set([
  "docs/davinci/plan/phase-2-records.md",
  "docs/davinci/plan/test-suites.md",
  schema,
]);

/** Docs and captured fixtures keep their historical bytes. */
export function eligible(relative: string): boolean {
  const parts = relative.split("/");
  if (parts.some((part) => part === "fixtures" || part === "_fixtures")) return false;
  return !frozen.has(relative);
}

function apply(source: string, rules: readonly Rule[]): string {
  return rules.reduce(
    (text, [from, to]) =>
      typeof from === "string" ? text.split(from).join(to) : text.replace(from, to),
    source,
  );
}

export function rewrite(relative: string, source: string): string {
  if (relative.startsWith("docs/")) {
    if (!indexes.has(relative)) return apply(source, names.slice(0, 1));
    return apply(source, relative === schema ? [...names, ...wire] : names);
  }
  const text = apply(source, [...names, ...wire]);
  const bare = new RegExp(`\\b${old}\\b`, "gu");
  if (relative.endsWith(".rs")) return text.replace(bare, "stage_feed");
  if (/\.(?:ts|vue)$/u.test(relative)) return text.replace(bare, "stageFeed");
  return text;
}

function grep(cwd: string, needle: string, scope: string[]): string[] {
  try {
    return execFileSync("git", ["grep", "-lIiz", "-F", needle, "--", ...scope], {
      cwd,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    })
      .split("\0")
      .filter(Boolean);
  } catch {
    return [];
  }
}

/** `check` lists files the rewrite would change; residue fails loudly in both modes. */
export function run(mode: string, cwd = root): string[] {
  if (!["moves", "rewrite", "check"].includes(mode)) {
    throw new Error("usage: rename-stage-feed.ts moves|rewrite|check");
  }
  for (const [from, to] of moves) {
    const hasOld = existsSync(path.join(cwd, from));
    // A moved file may be deleted later (#11545); only the old path must stay gone.
    if (mode === "moves" && hasOld === existsSync(path.join(cwd, to))) {
      throw new Error(`Expected exactly one of ${from} and ${to}`);
    }
    if (mode !== "moves" && hasOld) throw new Error(`Unmoved: ${from}`);
  }
  if (mode === "moves") {
    const pending = moves.filter(([from]) => existsSync(path.join(cwd, from)));
    for (const [from, to] of pending) execFileSync("git", ["mv", from, to], { cwd });
    return pending.map(([from]) => from);
  }
  const files = [
    ...grep(cwd, old, [".", ":!docs"]).filter(eligible),
    ...grep(cwd, `${old}-feed.schema.json`, ["docs"]),
    ...[...indexes].filter((relative) => grep(cwd, old, [relative]).length > 0),
  ];
  const stale: string[] = [];
  const residue: string[] = [];
  for (const relative of new Set(files)) {
    const file = path.join(cwd, relative);
    const before = readFileSync(file, "utf8");
    const after = rewrite(relative, before);
    if (after !== before) stale.push(relative);
    const live = !relative.startsWith("docs/") || relative === schema;
    if (live && after.toLowerCase().includes(old)) residue.push(relative);
    if (mode === "rewrite" && after !== before) writeFileSync(file, after);
  }
  if (residue.length > 0) throw new Error(`Unmapped spellings remain in:\n${residue.join("\n")}`);
  if (mode === "rewrite") format(cwd, stale);
  return stale;
}

/** Renamed spellings change line and table widths; reflow only the rewritten files. */
function format(cwd: string, files: string[]): void {
  const rust = files.filter((file) => file.endsWith(".rs"));
  if (rust.length > 0) {
    execFileSync(process.env.RUSTFMT ?? "rustfmt", ["--edition", "2024", ...rust], { cwd });
  }
  const web = files.filter((file) => /\.(?:css|json|md|ts|vue)$/u.test(file));
  if (web.length > 0) {
    execFileSync(path.join(cwd, "node_modules/.bin/vp"), ["fmt", ...web], { cwd, stdio: "ignore" });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const mode = process.argv[2] ?? "";
  const changed = run(mode);
  process.stdout.write(`${changed.length} files ${mode === "check" ? "stale" : "updated"}\n`);
  if (mode === "check" && changed.length > 0) process.exitCode = 1;
}
