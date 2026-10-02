import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import {
  ident,
  maskRustNonCode,
  maskRustSource,
  tokensOf,
} from "../davinci-storage-rust-syntax.ts";

type ModuleEdge = { file: string; testOnly: boolean };

function rustFiles(directory: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    assert.equal(entry.isSymbolicLink(), false, `${file}: unsupported symbolic module routing`);
    if (entry.isDirectory()) files.push(...rustFiles(file));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(file);
  }
  return files;
}

function moduleDirectory(file: string): string {
  return path.basename(file) === "mod.rs"
    ? path.dirname(file)
    : path.join(path.dirname(file), path.basename(file, ".rs"));
}

function moduleEdges(
  file: string,
  source: string,
  files: ReadonlyMap<string, string>,
): ModuleEdge[] {
  const code = maskRustNonCode(source);
  const production = maskRustSource(source);
  const tokens = tokensOf(code);
  // Path metadata (including cfg_attr) and includes cannot establish an exemption.
  for (const [index, token] of tokens.entries()) {
    if (ident(token) === "include" && tokens[index + 1]?.value === "!") {
      assert.fail(`${file}: unsupported module routing`);
    }
    if (token.value !== "#") continue;
    const start = tokens[index + 1]?.value === "!" ? index + 2 : index + 1;
    if (tokens[start]?.value !== "[") continue;
    let depth = 0;
    for (let cursor = start; cursor < tokens.length; cursor += 1) {
      const part = tokens[cursor];
      if (part.value === "[") depth += 1;
      else if (part.value === "]" && --depth === 0) break;
      else if (ident(part) === "path" && tokens[cursor + 1]?.value === "=") {
        assert.fail(`${file}: unsupported module routing`);
      }
    }
  }
  const scopes = [{ directory: moduleDirectory(file), depth: 0 }];
  const edges: ModuleEdge[] = [];
  let depth = 0;
  let parens = 0;
  let brackets = 0;
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const scope = scopes.at(-1)!;
    const name = ident(tokens[index + 1]);
    const delimiter = tokens[index + 2];
    if (
      token.value === "mod" &&
      depth === scope.depth &&
      parens === 0 &&
      brackets === 0 &&
      name &&
      delimiter
    ) {
      if (delimiter.value === "{") {
        depth += 1;
        scopes.push({ directory: path.join(scope.directory, name), depth });
        index += 2;
        continue;
      }
      if (delimiter.value === ";") {
        const candidates = [
          path.join(scope.directory, `${name}.rs`),
          path.join(scope.directory, name, "mod.rs"),
        ].filter((candidate) => files.has(candidate));
        assert.equal(candidates.length, 1, `${file}: module ${name} must have one ordinary source`);
        edges.push({
          file: candidates[0],
          testOnly: production.slice(token.start, delimiter.end).trim() === "",
        });
      }
    }
    if (token.value === "{") depth += 1;
    else if (token.value === "}") {
      if (scope.depth === depth && scopes.length > 1) scopes.pop();
      depth -= 1;
    }
    if (token.value === "(") parens += 1;
    else if (token.value === ")") parens -= 1;
    else if (token.value === "[") brackets += 1;
    else if (token.value === "]") brackets -= 1;
  }
  return edges;
}

/** Exempt only sources exclusively reachable through real cfg(test) module edges. */
export function productionRustModuleSources(entry: string): Map<string, string> {
  const root = path.resolve(entry);
  assert.equal(
    fs.lstatSync(root).isSymbolicLink(),
    false,
    `${root}: unsupported symbolic module routing`,
  );
  const files = new Map(
    [root, ...rustFiles(moduleDirectory(root)).sort()].map((file) => [
      file,
      fs.readFileSync(file, "utf8"),
    ]),
  );
  const graph = new Map(
    [...files].map(([file, source]) => [file, moduleEdges(file, source, files)]),
  );
  const production = new Set<string>();
  const tests = new Set<string>();
  const pending = [{ file: root, testOnly: false }];
  while (pending.length > 0) {
    const item = pending.pop()!;
    const seen = item.testOnly ? tests : production;
    if (seen.has(item.file)) continue;
    seen.add(item.file);
    for (const edge of graph.get(item.file) ?? []) {
      pending.push({ file: edge.file, testOnly: item.testOnly || edge.testOnly });
    }
  }
  return new Map(
    [...files]
      .filter(([file]) => production.has(file) || !tests.has(file))
      .map(([file, source]) => [file, maskRustSource(source)]),
  );
}
