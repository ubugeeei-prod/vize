import assert from "node:assert/strict";
import { maskRustNonCode, tokensOf } from "../../../tests/tooling/davinci-storage-rust-syntax.ts";

const builtins = new Set([
  "Debug",
  "Clone",
  "Copy",
  "Eq",
  "PartialEq",
  "Ord",
  "PartialOrd",
  "Hash",
  "Default",
]);
const reserved = new Set([
  ...builtins,
  "Serialize",
  "Deserialize",
  "serde",
  "std",
  "core",
  "test",
  "cfg",
  "path",
  "ignore",
  "derive",
  "expect",
]);

export function pairedTokens(source) {
  const code = maskRustNonCode(source);
  const tokens = tokensOf(code);
  const pairs = new Map();
  const stack = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const value = tokens[index].value;
    if (["{", "[", "("].includes(value)) stack.push(index);
    else if (["}", "]", ")"].includes(value)) {
      const start = stack.pop();
      assert.ok(
        start !== undefined && ["{}", "[]", "()"].includes(tokens[start].value + value),
        "unbalanced Rust registration",
      );
      pairs.set(start, index);
    }
  }
  assert.equal(stack.length, 0, "unclosed Rust registration");
  return { tokens, pairs, code };
}

// Parse authored item boundaries only. Function bodies are never executed or
// fingerprinted; an opaque item cannot establish library-only selection.
export function normalizeAttribute(attribute) {
  return attribute.replace(/"(?:\\.|[^"\\])*"|\s+/gu, (part) => (part.startsWith('"') ? part : ""));
}

function flatten(items) {
  return items.flatMap((item) => [item, ...(item.children ? flatten(item.children) : [])]);
}

export function validateScopeAuthority(items, { library = false } = {}) {
  const flat = flatten(items);
  for (const item of flat) {
    assert.notEqual(item.kind, "opaque", "opaque ancestral registration authority");
    if (library) {
      for (const attr of item.attrs) {
        const normalized = normalizeAttribute(attr);
        if (
          /^#\[cfg\([\s\S]+\)\]$/u.test(normalized) ||
          normalized === "#[test]" ||
          /^#\[expect\([\s\S]+\)\]$/u.test(normalized)
        )
          continue;
        const derives = normalized.match(/^#\[derive\(([^()]*)\)\]$/u)?.[1].split(",");
        assert.ok(
          derives?.length && derives.every((name) => builtins.has(name)),
          "opaque ancestral attribute authority",
        );
      }
    }
    assert.ok(!reserved.has(item.name), `shadowed registration authority: ${item.name}`);
    assert.notEqual(item.kind, "extern", "opaque external registration authority");
    if (item.kind !== "use") continue;
    const useSource = maskRustNonCode(item.text);
    const useCode = useSource.replace(/\s+/gu, "");
    if (useCode.includes("*"))
      assert.match(
        useCode,
        /^(?:pub(?:\([^)]*\))?)?use(?:super|self|crate)::\*;$/u,
        "opaque wildcard provider",
      );
    const words = tokensOf(useSource).map((token) => token.value.replace(/^r#/u, ""));
    assert.ok(
      !words.some((word, index) => word === "as" && reserved.has(words[index + 1])),
      "shadowed attribute or derive provider",
    );
    if (!words.some((word) => reserved.has(word))) continue;
    assert.ok(
      /^(?:pub(?:\([^)]*\))?)?use(?:std|core)::/u.test(useCode) ||
        /^useserde::(?:Serialize|Deserialize|\{(?:Serialize|Deserialize)(?:,(?:Serialize|Deserialize))*[,]?\});$/u.test(
          useCode,
        ),
      "shadowed attribute or derive provider",
    );
  }
  return flat;
}

function validateCfg(attribute, features) {
  const condition = attribute.match(/^#\s*!?\s*\[\s*cfg\s*\(([\s\S]*)\)\s*\]$/u)?.[1];
  assert.ok(condition, "opaque conditional registration");
  let cursor = 0;
  const next = () => {
    while (/\s/u.test(condition[cursor] ?? "")) cursor += 1;
    return condition[cursor];
  };
  const predicate = () => {
    next();
    const name = condition.slice(cursor).match(/^[A-Za-z_]\w*/u)?.[0];
    assert.ok(name, "opaque conditional registration");
    cursor += name.length;
    if (["all", "any", "not"].includes(name)) {
      assert.equal(next(), "(", "opaque conditional registration");
      cursor += 1;
      let count = 0;
      while (next() !== ")") {
        predicate();
        count += 1;
        if (next() !== ",") break;
        cursor += 1;
      }
      assert.equal(next(), ")", "opaque conditional registration");
      cursor += 1;
      if (name === "not") assert.equal(count, 1, "opaque conditional registration");
      return;
    }
    if (["test", "unix", "windows"].includes(name)) return;
    assert.ok(["feature", "target_os"].includes(name), "opaque conditional registration");
    assert.equal(next(), "=", "opaque conditional registration");
    cursor += 1;
    next();
    const literal = condition.slice(cursor).match(/^"([^"\\]*)"/u);
    assert.ok(literal, "opaque conditional registration");
    const value = literal[1];
    cursor += literal[0].length;
    if (name === "feature") assert.ok(features.has(value), "undeclared Cargo cfg feature");
    else assert.equal(value, "linux", "opaque conditional registration");
  };
  // Validate declarations only: every recognized branch remains in the union,
  // including disabled features and contradictory predicates.
  predicate();
  next();
  assert.equal(cursor, condition.length, "opaque conditional registration");
}

export function validateRegistrationSyntax(source, items, features) {
  const code = maskRustNonCode(source);
  assert.doesNotMatch(code, /\b(?:macro_rules\s*!|include\s*!)/u, "opaque Rust generator");
  const flat = validateScopeAuthority(items);
  const { tokens, pairs } = pairedTokens(source);
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index].value !== "#") continue;
    let bracket = index + 1;
    if (tokens[bracket]?.value === "!") bracket += 1;
    assert.equal(tokens[bracket]?.value, "[", "opaque attribute syntax");
    const end = pairs.get(bracket);
    const attr = source.slice(tokens[index].start, tokens[end].end);
    const key = tokens[bracket + 1]?.value;
    assert.ok(
      ["(", "=", "]"].includes(tokens[bracket + 2]?.value),
      "qualified or opaque attribute head",
    );
    assert.ok(
      ["test", "path", "cfg", "derive", "serde", "expect", "ignore"].includes(key),
      `opaque registration attribute: ${key}`,
    );
    const normalized = normalizeAttribute(attr);
    if (key === "test") assert.equal(normalized, "#[test]", "opaque test attribute shape");
    if (key === "ignore")
      assert.match(
        normalized,
        /^#\[ignore(?:="(?:\\.|[^"\\])*")?\]$/u,
        "opaque ignore attribute shape",
      );
    if (key === "path")
      assert.match(normalized, /^#\[path="[^"\\]+"\]$/u, "opaque path attribute shape");
    if (["serde", "expect"].includes(key))
      assert.match(
        normalized,
        new RegExp(`^#${key === "expect" ? "!?" : ""}\\[${key}\\([\\s\\S]+\\)\\]$`, "u"),
        "opaque helper attribute shape",
      );
    if (key === "cfg") validateCfg(attr, features);
    if (key === "derive") {
      const names = attr
        .match(/^#\[derive\(([^()]*)\)\]$/u)?.[1]
        .split(",")
        .map((name) => name.trim())
        .filter(Boolean);
      assert.ok(names?.length, "opaque derive arguments");
      for (const name of names) {
        if (builtins.has(name)) continue;
        assert.ok(
          ["Serialize", "Deserialize", "serde::Serialize", "serde::Deserialize"].includes(name),
          `opaque derive: ${name}`,
        );
        if (!name.startsWith("serde::"))
          assert.ok(
            flat.some(
              (item) =>
                item.kind === "use" &&
                new RegExp(
                  `^use\\s+serde::(?:${name}|\\{[^{}]*\\b${name}\\b[^{}]*\\})\\s*;?$`,
                  "u",
                ).test(item.text),
            ),
            "unbound serde derive",
          );
      }
    }
    index = end;
  }
}
