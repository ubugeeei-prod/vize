import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";
import { collectUnitEvidence } from "./compare-unused-bindings-profiles.ts";

type HtmlFile = { name: string; size: number; mtimeNs: string };
const save = (file: string, value: unknown) =>
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
const digest = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const inside = (root: string, file: string) => {
  const name = relative(root, realpathSync(file));
  return name !== ".." && !name.startsWith("../") && !isAbsolute(name);
};

function htmlFiles(target: string) {
  const files: HtmlFile[] = [];
  const gaps: { name: string; reason: string }[] = [];
  const directory = join(target, "cargo-timings");
  if (!existsSync(directory)) return { files, gaps };
  if (!inside(realpathSync(target), directory))
    throw new Error("Cargo timing directory is outside the observed target");
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (!/^cargo-timing(?:-[A-Za-z0-9_.-]+)?\.html$/.test(entry.name)) continue;
    const file = join(directory, entry.name);
    if ((!entry.isFile() && !entry.isSymbolicLink()) || !inside(realpathSync(directory), file)) {
      gaps.push({ name: entry.name, reason: "not a timing file inside its directory" });
      continue;
    }
    const stat = statSync(file, { bigint: true });
    files.push({ name: entry.name, size: Number(stat.size), mtimeNs: String(stat.mtimeNs) });
  }
  return { files: files.sort((a, b) => a.name.localeCompare(b.name)), gaps };
}

export function prepareBuildEvidence(output: string, target: string) {
  mkdirSync(target, { recursive: true });
  mkdirSync(output, { recursive: true });
  save(join(output, "html-before.json"), htmlFiles(target));
  // Cargo1.99.1 writes stamped HTML then link_or_copy of this alias. Remove
  // only the alias so a failed build cannot qualify a cached report as fresh.
  // https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/compiler/timings/mod.rs#L274-L283
  rmSync(join(target, "cargo-timings", "cargo-timing.html"), { force: true });
}

export function finishBuildEvidence(output: string, target: string, context: object) {
  mkdirSync(output, { recursive: true });
  const beforePath = join(output, "html-before.json");
  const before: { files: HtmlFile[] } = existsSync(beforePath)
    ? JSON.parse(readFileSync(beforePath, "utf8"))
    : { files: [] };
  const after = htmlFiles(target);
  const old = new Map(before.files.map((file) => [file.name, file]));
  const retained: { name: string; sha256: string }[] = [];
  const gaps = [...after.gaps];
  for (const file of after.files) {
    const previous = old.get(file.name);
    if (previous?.mtimeNs === file.mtimeNs && previous.size === file.size) continue;
    if (!existsSync(beforePath)) continue; // No authenticated pre-build boundary.
    const source = join(target, "cargo-timings", file.name);
    mkdirSync(join(output, "html"), { recursive: true });
    const destination = join(output, "html", file.name);
    copyFileSync(source, destination);
    retained.push({ name: file.name, sha256: digest(destination) });
  }
  if (!existsSync(beforePath))
    gaps.push({ name: "html-before.json", reason: "prepare did not run" });
  if (retained.length === 0)
    gaps.push({
      name: "cargo-timing.html",
      reason: "no fresh HTML observed; cached HTML excluded",
    });
  save(join(output, "html-after.json"), { ...after, retained, gaps });
  // Same bounded emitted-hash lookup as #9289; no objects or whole target copy.
  return collectUnitEvidence(output, target, context);
}

if (import.meta.main) {
  const [operation, output] = process.argv.slice(2);
  if (!output || !["before", "collect"].includes(operation))
    throw new Error("usage: rust-build-evidence.ts before|collect OUTPUT");
  const target = resolve("target");
  if (operation === "before") prepareBuildEvidence(output, target);
  else {
    const command = (name: string, args: string[]) =>
      execFileSync(name, args, { encoding: "utf8" }).trim();
    finishBuildEvidence(output, target, {
      source: command("git", ["rev-parse", "HEAD"]),
      tree: command("git", ["rev-parse", "HEAD^{tree}"]),
      rustc: command("rustc", ["-Vv"]),
      plan: JSON.parse(process.env.RUST_PLAN_JSON || "null"),
      env: Object.fromEntries(
        [
          "COHORT",
          "NEXTEST_PROFILE",
          "NEXTEST_FILTER",
          "CARGO_BUILD_JOBS",
          "CARGO_INCREMENTAL",
          "RUST_TEST_THREADS",
          "VIZE_NUXT_CONFIG_ITERATIONS",
          "VIZE_TEST_DISABLE_TSGO",
          "VIZE_TEST_REQUIRE_TSGO",
          "RUSTFLAGS",
          "CARGO_ENCODED_RUSTFLAGS",
          "CARGO_TARGET_DIR",
        ].map((key) => [key, process.env[key] ?? null]),
      ),
    });
  }
}
