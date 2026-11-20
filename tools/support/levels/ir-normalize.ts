// Normalizes rustc's optimized LLVM IR so a pure move or rename compares equal.
//
// A function is keyed by its demangled Rust path (Rust v0 mangling, crate
// disambiguators dropped) and its body is kept with every name that a move or a
// rename can change replaced by a stable spelling: symbol references become
// demangled paths, anonymous globals (`@alloc_…`, `@anon.…`, `@vtable.…`) and
// metadata ids become placeholders, attribute-group ids become their contents,
// and named struct types lose their numeric suffixes. Paths are then relocated
// by the caller's module moves and identifier renames, so a base built before
// the move spells every path as the head does.

import { createHash } from "node:crypto";

/** Rust v0 symbol demangling (RFC 2603), enough to name every IR function. */
export function demangleV0(symbol: string): string | null {
  const start = symbol.startsWith("_R") ? 2 : symbol.startsWith("__R") ? 3 : -1;
  if (start < 0) return null;
  const text = symbol.slice(start);
  let at = 0;
  const memo = new Map<number, string>();
  const fail = (): never => {
    throw new Error(`malformed v0 symbol ${symbol}`);
  };
  const peek = () => text[at] ?? "";
  const eat = (char: string) => {
    if (text[at] === char) {
      at++;
      return true;
    }
    return false;
  };
  const decimal = (): number => {
    const begin = at;
    // `0` stands alone: closures nest as `…00` (two empty identifiers).
    if (eat("0")) return 0;
    while (/[0-9]/u.test(peek())) at++;
    if (at === begin) fail();
    return Number(text.slice(begin, at));
  };
  const base62 = (): number => {
    if (eat("_")) return 0;
    let value = 0;
    while (!eat("_")) {
      const char = peek();
      if (!char) fail();
      const digit = /[0-9]/u.test(char)
        ? char.charCodeAt(0) - 48
        : /[a-z]/u.test(char)
          ? char.charCodeAt(0) - 87
          : /[A-Z]/u.test(char)
            ? char.charCodeAt(0) - 29
            : fail();
      value = value * 62 + digit;
      at++;
    }
    return value + 1;
  };
  const disambiguator = () => {
    if (eat("s")) base62();
  };
  const identifier = (): string => {
    disambiguator();
    eat("u");
    const length = decimal();
    eat("_");
    const name = text.slice(at, at + length);
    at += length;
    return name;
  };
  const backref = (parse: () => string): string => {
    const target = base62();
    const saved = at;
    at = target;
    const cached = memo.get(target);
    const value = cached ?? parse();
    memo.set(target, value);
    at = saved;
    return value;
  };
  const genericArgs = (): string => {
    const args: string[] = [];
    while (!eat("E")) {
      if (eat("L")) {
        base62();
        args.push("'_");
      } else if (eat("K")) args.push(constant());
      else args.push(type());
    }
    return args.length ? `<${args.join(", ")}>` : "";
  };
  const path = (): string => {
    const begin = at;
    const char = text[at++] ?? "";
    let value: string;
    switch (char) {
      case "C":
        value = identifier();
        break;
      case "M": {
        disambiguator();
        path();
        value = `<${type()}>`;
        break;
      }
      case "X": {
        disambiguator();
        path();
        const self = type();
        value = `<${self} as ${path()}>`;
        break;
      }
      case "Y": {
        const self = type();
        value = `<${self} as ${path()}>`;
        break;
      }
      case "N": {
        const namespace = text[at++] ?? fail();
        const parent = path();
        const name = identifier();
        value = /[a-z]/u.test(namespace)
          ? `${parent}::${name}`
          : `${parent}::{${namespace === "C" ? "closure" : namespace === "S" ? "shim" : namespace}${name ? `:${name}` : ""}}`;
        break;
      }
      case "I": {
        const base = path();
        value = base + genericArgs();
        break;
      }
      case "B":
        value = backref(path);
        break;
      default:
        value = fail();
    }
    memo.set(begin, value);
    return value;
  };
  const basic: Record<string, string> = {
    a: "i8",
    b: "bool",
    c: "char",
    d: "f64",
    e: "str",
    f: "f32",
    h: "u8",
    i: "isize",
    j: "usize",
    l: "i32",
    m: "u32",
    n: "i128",
    o: "u128",
    s: "i16",
    t: "u16",
    u: "()",
    v: "...",
    x: "i64",
    y: "u64",
    z: "!",
    p: "_",
  };
  const binder = () => {
    if (eat("G")) base62();
  };
  const lifetime = () => {
    if (eat("L")) base62();
  };
  const type = (): string => {
    const begin = at;
    const char = peek();
    if (basic[char] !== undefined) {
      at++;
      return basic[char];
    }
    let value: string;
    switch (char) {
      case "A": {
        at++;
        const element = type();
        value = `[${element}; ${constant()}]`;
        break;
      }
      case "S":
        at++;
        value = `[${type()}]`;
        break;
      case "T": {
        at++;
        const items: string[] = [];
        while (!eat("E")) items.push(type());
        value = `(${items.join(", ")})`;
        break;
      }
      case "R":
      case "Q":
        at++;
        lifetime();
        value = `&${char === "Q" ? "mut " : ""}${type()}`;
        break;
      case "P":
      case "O":
        at++;
        value = `*${char === "O" ? "mut" : "const"} ${type()}`;
        break;
      case "F": {
        at++;
        binder();
        const unsafe = eat("U");
        if (eat("K")) {
          if (!eat("C")) identifier();
        }
        const params: string[] = [];
        while (!eat("E")) params.push(type());
        value = `${unsafe ? "unsafe " : ""}fn(${params.join(", ")}) -> ${type()}`;
        break;
      }
      case "D": {
        at++;
        binder();
        const traits: string[] = [];
        while (!eat("E")) {
          let bound = path();
          while (eat("p")) {
            const name = identifier();
            bound += `<${name} = ${type()}>`;
          }
          traits.push(bound);
        }
        lifetime();
        value = `dyn ${traits.join(" + ")}`;
        break;
      }
      case "B":
        at++;
        value = backref(type);
        break;
      default:
        value = path();
    }
    memo.set(begin, value);
    return value;
  };
  const constant = (): string => {
    if (eat("p")) return "_";
    if (eat("B")) return backref(constant);
    const ty = type();
    const negative = eat("n");
    const begin = at;
    while (!eat("_")) {
      if (!peek()) fail();
      at++;
    }
    return `${negative ? "-" : ""}0x${text.slice(begin, at - 1)}${ty}`;
  };
  try {
    if (/[0-9]/u.test(peek())) decimal();
    const value = path();
    return value;
  } catch {
    return null;
  }
}

/** A path rewrite from the base spelling to the head spelling. */
export type Relocate = (demangled: string) => string;

export interface NormalizedModule {
  /** Normalized function name → normalized bodies (a multiset). */
  readonly functions: ReadonlyMap<string, readonly string[]>;
}

const symbolPattern = /@("?)([A-Za-z_$.][\w$.]*)\1/gu;
const mangledToken = /(?<![\w$])_R[0-9A-Za-z_]+/gu;
const anonymousGlobal =
  /^(?:alloc_[0-9a-f]+|anon\.[\w.]+|__unnamed_\d+|vtable\.[\w.]+|str\.[\w.]*|switch\.table\..*|\.str(?:\.\d+)?)$/u;
const namedType = /%"([^"]*)"/gu;

/** Demangle and relocate every Rust symbol spelled inside local names. */
function demangleTokens(text: string, relocate: Relocate): string {
  return text.replace(mangledToken, (token) => {
    const demangled = demangleV0(token);
    return demangled === null ? token : `{${relocate(demangled)}}`;
  });
}

function relocateTypes(text: string, relocate: Relocate): string {
  return text.replace(
    namedType,
    (_whole, type: string) => `%"${relocate(type.replace(/\.\d+$/u, ""))}"`,
  );
}

function namedSymbol(raw: string, relocate: Relocate): string {
  const stripped = raw.replace(/\.llvm\.\d+$/u, "");
  const demangled = demangleV0(stripped);
  return demangled === null ? stripped : relocate(demangled);
}

/**
 * Anonymous globals (string bytes, vtables, switch tables) are named by a hash
 * of their normalized initializer, so a renumbering compares equal while a
 * reference to different contents does not.
 */
function anonymousNames(lines: readonly string[], relocate: Relocate): Map<string, string> {
  const names = new Map<string, string>();
  for (const line of lines) {
    const match = /^@("?)([^"\s]+)\1 = (.*)$/u.exec(line);
    if (!match || !anonymousGlobal.test(match[2])) continue;
    const references = match[3]
      .replace(/!\d+/gu, "!M")
      .replace(symbolPattern, (_whole, _quote: string, raw: string) =>
        anonymousGlobal.test(raw) ? "@{G}" : `@{${namedSymbol(raw, relocate)}}`,
      );
    const content = relocateTypes(demangleTokens(references, relocate), relocate);
    names.set(match[2], `G:${createHash("sha256").update(content).digest("hex").slice(0, 16)}`);
  }
  return names;
}

/** Normalize one `.ll` module for comparison. */
export function normalizeModule(ir: string, relocate: Relocate = (name) => name): NormalizedModule {
  const attributes = new Map<string, string>();
  for (const match of ir.matchAll(/^attributes #(\d+) = \{ (.*) \}$/gmu))
    attributes.set(match[1], match[2]);
  const lines = ir.split("\n");
  const anonymous = anonymousNames(lines, relocate);
  const symbol = (raw: string) => anonymous.get(raw) ?? namedSymbol(raw, relocate);
  const functions = new Map<string, string[]>();
  for (let index = 0; index < lines.length; index++) {
    const head = lines[index];
    if (!head.startsWith("define ")) continue;
    const name = /@("?)([^"(\s]+)\1\(/u.exec(head);
    if (!name) continue;
    const body = [head];
    while (index + 1 < lines.length && lines[index].trim() !== "}") body.push(lines[++index]);
    const joined = body
      // rustc annotates calls with demangled comments (`; call a::b::f`).
      .map((line) => line.replace(/\s*;.*$/u, ""))
      .filter((line) => line !== "")
      .join("\n")
      .replace(/ ?!(?:[A-Za-z_][\w.]*\s+)?!\d+/gu, " !M")
      .replace(/!\d+/gu, "!M")
      .replace(/#(\d+)/gu, (whole, id: string) =>
        attributes.has(id) ? `#{${attributes.get(id)}}` : whole,
      )
      .replace(symbolPattern, (_whole, _quote: string, raw: string) => `@{${symbol(raw)}}`)
      .replace(/%([A-Za-z_][\w:<>]*)\.\d+\b/gu, "%$1");
    // Inlined callees name their blocks after themselves (`%_RNv….exit`).
    const text = relocateTypes(demangleTokens(joined, relocate), relocate);
    const key = symbol(name[2]);
    const bodies = functions.get(key) ?? [];
    bodies.push(text);
    functions.set(key, bodies);
  }
  for (const bodies of functions.values()) bodies.sort();
  return { functions };
}

export interface ModuleDifference {
  /** Functions only the base defines (after relocation). */
  readonly onlyBase: readonly string[];
  /** Functions only the head defines. */
  readonly onlyHead: readonly string[];
  /** Functions both define whose bodies differ. */
  readonly changed: readonly string[];
  /** Functions with identical bodies. */
  readonly identical: number;
}

export function compareModules(base: NormalizedModule, head: NormalizedModule): ModuleDifference {
  const onlyBase: string[] = [];
  const onlyHead: string[] = [];
  const changed: string[] = [];
  let identical = 0;
  for (const [name, bodies] of base.functions) {
    const other = head.functions.get(name);
    if (!other) onlyBase.push(name);
    else if (other.length !== bodies.length || other.some((body, i) => body !== bodies[i]))
      changed.push(name);
    else identical += bodies.length;
  }
  for (const name of head.functions.keys()) if (!base.functions.has(name)) onlyHead.push(name);
  return {
    onlyBase: onlyBase.sort(),
    onlyHead: onlyHead.sort(),
    changed: changed.sort(),
    identical,
  };
}

/** The first `limit` differing normalized lines of one changed function. */
export function bodyDiff(
  base: NormalizedModule,
  head: NormalizedModule,
  name: string,
  limit = 4,
): string[] {
  const before = base.functions.get(name) ?? [];
  const after = head.functions.get(name) ?? [];
  if (before.length !== after.length)
    return [`${before.length} bodies in base, ${after.length} in head`];
  const out: string[] = [];
  for (let body = 0; body < before.length && out.length < limit * 2; body++) {
    const left = before[body].split("\n");
    const right = after[body].split("\n");
    for (let line = 0; line < Math.max(left.length, right.length); line++) {
      if (left[line] === right[line]) continue;
      out.push(`- ${left[line] ?? ""}`, `+ ${right[line] ?? ""}`);
      if (out.length >= limit * 2) break;
    }
  }
  return out;
}

export function isIdentical(difference: ModuleDifference): boolean {
  return !difference.onlyBase.length && !difference.onlyHead.length && !difference.changed.length;
}
