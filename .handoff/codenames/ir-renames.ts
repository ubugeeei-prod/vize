// usage: node ir-renames.ts <repo> <base>
// Prints the ir-identity arguments for the rename: one --crate per package whose library sources
// changed, and one --rename per codename-bearing Rust identifier in the base's library sources.
import { execFileSync } from "node:child_process";
import path from "node:path";
import { pathToFileURL } from "node:url";

const [repo, base] = process.argv.slice(2);
const rename = await import(
  pathToFileURL(path.join(repo, "tools/support/levels/rename-codenames.ts")).href
);
const git = (...args: string[]) =>
  execFileSync("git", args, { cwd: repo, encoding: "utf8", maxBuffer: 1 << 30 });
const files = git("ls-tree", "-r", "--name-only", base)
  .split("\n")
  .filter((file) => /^(?:crates|davinci)\/[^/]+\/src\/.+\.rs$/u.test(file))
  .filter((file) => !/(?:^|\/)tests?(?:\.rs|\/)|_tests?\.rs$/u.test(file));
const codename = /davinci|folio|impeto|disegno|ricalco|sinopia|stesura|spolvero/iu;
const pairs = new Map<string, string>();
for (const file of files) {
  const text = git("show", `${base}:${file}`);
  for (const [word] of text.matchAll(/[A-Za-z_][A-Za-z0-9_]*/gu)) {
    if (!codename.test(word) || pairs.has(word)) continue;
    if (rename.levelHits(word).length === 0 && !/davinci/iu.test(word)) continue;
    const next = rename.renameWord(word);
    if (next !== word) pairs.set(word, next);
  }
}
const crates = new Set<string>();
for (const file of git("diff", "--name-only", base, "HEAD").split("\n")) {
  const match = /^((?:crates|levels)\/[^/]+)\/src\//u.exec(file);
  if (!match) continue;
  const manifest = git("show", `HEAD:${match[1]}/Cargo.toml`);
  const name = /^name\s*=\s*"([^"]+)"/mu.exec(manifest)?.[1];
  if (name) crates.add(name);
}
const args = [
  ...[...crates].sort().map((name) => `--crate ${name}`),
  ...[...pairs].sort().map(([from, to]) => `--rename ${from}=${to}`),
];
process.stdout.write(args.join(" ") + "\n");
process.stderr.write(`${crates.size} crates, ${pairs.size} identifier renames\n`);
