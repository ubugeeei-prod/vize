import assert from "node:assert/strict";
import { lstatSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import path from "node:path";
import {
  pairedTokens,
  normalizeAttribute,
  validateRegistrationSyntax,
} from "./typechecker-native-canon-lib-syntax.mjs";
export { normalizeAttribute } from "./typechecker-native-canon-lib-syntax.mjs";

const identifier = /^(?:r#)?[A-Za-z_]\w*$/u;
const kinds = new Set([
  "use",
  "const",
  "static",
  "fn",
  "mod",
  "struct",
  "enum",
  "type",
  "impl",
  "trait",
  "extern",
]);
const semicolonItems = new Set(["use", "const", "static", "type"]);
export function physicalSource(root) {
  const absolute = realpathSync(root);
  const checked = (relative) => {
    assert.ok(
      relative &&
        !path.isAbsolute(relative) &&
        !relative.split("/").some((part) => !part || part === "." || part === ".."),
      "unsafe source path",
    );
    let current = absolute;
    for (const part of relative.split("/")) {
      current = path.join(current, part);
      assert.ok(!lstatSync(current).isSymbolicLink(), `symlink source: ${relative}`);
    }
    return current;
  };
  return {
    read(relative) {
      const file = checked(relative);
      assert.ok(lstatSync(file).isFile(), `non-file source: ${relative}`);
      return readFileSync(file, "utf8");
    },
    files(relative) {
      const found = [];
      const visit = (directory) => {
        const absolutePath = checked(directory);
        for (const name of readdirSync(absolutePath).sort()) {
          const child = `${directory}/${name}`;
          const stat = lstatSync(checked(child));
          if (stat.isDirectory()) visit(child);
          else {
            assert.ok(stat.isFile(), `non-file source: ${child}`);
            found.push(child);
          }
        }
      };
      visit(relative);
      return found;
    },
  };
}

export function rustItems(source, { library = false } = {}) {
  const { tokens, pairs, code } = pairedTokens(source);
  const parse = (from, to) => {
    const items = [];
    let index = from;
    while (index < to) {
      if (tokens[index].value === ";") {
        index += 1;
        continue;
      }
      const attrs = [];
      while (tokens[index]?.value === "#") {
        let bracket = index + 1;
        if (tokens[bracket]?.value === "!") bracket += 1;
        assert.equal(tokens[bracket]?.value, "[", "opaque Rust attribute");
        const end = pairs.get(bracket);
        attrs.push(source.slice(tokens[index].start, tokens[end].end));
        index = end + 1;
      }
      if (index === to) {
        assert.ok(
          attrs.every((attr) => attr.startsWith("#!")),
          "orphan outer attribute",
        );
        break;
      }
      const declaration = index;
      while (["pub", "unsafe", "async"].includes(tokens[index].value)) {
        index += 1;
        if (tokens[index]?.value === "(") index = pairs.get(index) + 1;
      }
      if (tokens[index].value === "const" && tokens[index + 1]?.value === "fn") index += 1;
      const kind = tokens[index].value;
      if (!kinds.has(kind) && library) {
        let cursor = index + 1;
        while (cursor < to && !["{", "[", "(", ";"].includes(tokens[cursor].value)) cursor += 1;
        assert.ok(cursor < to, "unterminated unrelated library item");
        items.push({ kind: "opaque", attrs, children: null });
        index = tokens[cursor].value === ";" ? cursor + 1 : pairs.get(cursor) + 1;
        continue;
      }
      assert.ok(kinds.has(kind), `opaque Rust item: ${kind}`);
      let name;
      if (["fn", "mod", "struct", "enum", "trait", "type"].includes(kind)) {
        const token = tokens[index + 1];
        const complete =
          token &&
          identifier.test(token.value) &&
          /^\s*$/u.test(code.slice(tokens[index].end, token.start)) &&
          (code.codePointAt(token.end) ?? 0) <= 0x7f;
        assert.ok(complete || library, "opaque physical Rust item name");
        if (complete) name = token.value.replace(/^r#/u, "");
      }
      let cursor = index + 1;
      let terminator;
      while (cursor < to) {
        const value = tokens[cursor].value;
        if (value === ";" || (value === "{" && !semicolonItems.has(kind))) {
          terminator = cursor;
          break;
        }
        cursor = ["{", "[", "("].includes(value) ? pairs.get(cursor) + 1 : cursor + 1;
      }
      assert.ok(terminator !== undefined, `unterminated Rust item: ${kind}`);
      const end = tokens[terminator].value === "{" ? pairs.get(terminator) : terminator;
      items.push({
        kind,
        name,
        attrs,
        text: source.slice(tokens[declaration].start, tokens[end].end),
        children:
          kind === "mod" && tokens[terminator].value === "{" ? parse(terminator + 1, end) : null,
      });
      index = end + 1;
    }
    return items;
  };
  return parse(0, tokens.length);
}

export function registrations(source, roots, features) {
  const files = new Set();
  const cases = [];
  const edges = [];
  const visit = (file, prefix, defaultDirectory, pathDirectory, ancestors) => {
    assert.ok(!ancestors.includes(file), "cyclic Rust module");
    assert.ok(
      file.startsWith("crates/vize_canon/") && file.endsWith(".rs"),
      "external registration module",
    );
    const text = source.read(file);
    const items = rustItems(text);
    validateRegistrationSyntax(text, items, features);
    files.add(file);
    const walk = (members, names, defaultDir, pathDir) => {
      for (const item of members) {
        if (
          item.kind === "fn" &&
          item.attrs.some((attr) => normalizeAttribute(attr) === "#[test]")
        ) {
          cases.push({ root: ancestors[0] ?? file, file, name: [...names, item.name].join("::") });
        }
        if (item.kind !== "mod") continue;
        const attrs = item.attrs.filter((attr) => attr.replace(/\s+/gu, "").startsWith("#[path"));
        assert.ok(attrs.length <= 1, "ambiguous module path");
        if (item.children) {
          assert.equal(attrs.length, 0, "path on inline module");
          walk(
            item.children,
            [...names, item.name],
            path.posix.join(defaultDir, item.name),
            path.posix.join(defaultDir, item.name),
          );
          continue;
        }
        let destination;
        if (attrs.length) {
          const literal = attrs[0].match(/^#\s*\[\s*path\s*=\s*"([^"\\]+)"\s*\]$/u)?.[1];
          assert.ok(literal && !path.posix.isAbsolute(literal), "opaque module path");
          destination = path.posix.normalize(path.posix.join(pathDir, literal));
        } else {
          const options = [
            `${defaultDir}/${item.name}.rs`,
            `${defaultDir}/${item.name}/mod.rs`,
          ].filter((candidate) => source.paths.has(candidate));
          assert.equal(options.length, 1, "missing or ambiguous module");
          [destination] = options;
        }
        edges.push({ file, destination, name: item.name });
        visit(
          destination,
          [...names, item.name],
          attrs.length || destination.endsWith("/mod.rs")
            ? path.posix.dirname(destination)
            : destination.slice(0, -3),
          path.posix.dirname(destination),
          [...ancestors, file],
        );
      }
    };
    walk(items, prefix, defaultDirectory, pathDirectory);
  };
  for (const root of roots) visit(root, [], path.posix.dirname(root), path.posix.dirname(root), []);
  return { files: [...files].sort((left, right) => left.localeCompare(right)), edges, cases };
}
