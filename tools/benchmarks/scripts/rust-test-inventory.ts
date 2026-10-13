import fs from "node:fs";
import path from "node:path";
import toml from "@iarna/toml";

export function customRustTestTargets(files) {
  const targets = new Map();
  for (const manifest of files) {
    if (path.basename(manifest) !== "Cargo.toml") continue;
    const content = fs.readFileSync(manifest, "utf8");
    if (!/^\s*\[\[\s*test\s*\]\]/m.test(content)) continue;
    const tests = toml.parse(content).test;
    if (!Array.isArray(tests)) continue;
    for (const target of tests) {
      if (target.harness !== false || typeof target.name !== "string") continue;
      const source = target.path ?? `tests/${target.name}.rs`;
      if (typeof source === "string") {
        targets.set(path.resolve(path.dirname(manifest), source), target.name);
      }
    }
  }
  return targets;
}

export function collectRustTests(root, absolute, customTargets, lineNumberForIndex) {
  const relativePath = path.relative(root, absolute).split(path.sep).join("/");
  const content = fs.readFileSync(absolute, "utf8");
  const customTarget = customTargets.get(absolute);
  if (customTarget != null) {
    const main = /^\s*fn\s+main\s*\(/m.exec(content);
    return main == null
      ? null
      : {
          area: "Rust",
          runner: "cargo test (custom harness)",
          file: relativePath,
          count: 1,
          tests: [{ name: customTarget, line: lineNumberForIndex(content, main.index) }],
        };
  }
  if (!content.includes("#[test") && !content.includes("::test") && !content.includes("#[rstest")) {
    return null;
  }

  const tests = [];
  const pattern =
    /#\[\s*(?:tokio::)?test(?:\s*\([^)]*\))?\s*\][\s\r\n]*(?:#\[[^\]]+\][\s\r\n]*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
  for (const match of content.matchAll(pattern)) {
    tests.push({ name: match[1], line: lineNumberForIndex(content, match.index) });
  }
  return tests.length === 0
    ? null
    : { area: "Rust", runner: "cargo test", file: relativePath, count: tests.length, tests };
}
