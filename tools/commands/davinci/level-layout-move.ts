// Scripted, replayable module moves for the input-neutral level layout
// (docs/davinci/decisions/2026-11-17-input-neutral-levels.md, "Migration").
//
//   node tools/commands/davinci/level-layout-move.ts <step> [--dry-run]
//
// A step runs `git mv` for its files and rewrites the paths that name them:
// the `mod` declarations, `crate::`/`self::`/`super::` paths and `use` groups
// inside the crate, other crates' `<crate>::` paths, and repository path
// strings (tooling, docs, fixtures). It makes no other edit. A branch made
// before a step lands rebases by re-running the step on its own tree; a step
// that already ran finds nothing to move.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

/** One module move inside a crate, as module paths. */
export interface ModuleMove {
  readonly from: readonly string[];
  readonly to: readonly string[];
}

export interface Step {
  /** The crate directory, relative to the repository root. */
  readonly crate: string;
  /** The crate's name in other crates' paths. */
  readonly name: string;
  readonly moves: readonly ModuleMove[];
  /** New parent modules the step creates, with their documentation. */
  readonly parents: Readonly<Record<string, string>>;
  /** Moved modules whose name the old parent also re-exports as an item: a path
   *  ending in one names the item and stays. */
  readonly items: readonly string[];
  /** Rust identifiers renamed in every tracked `.rs` file, word for word. */
  readonly identifiers?: Readonly<Record<string, string>>;
}

const move = (from: string, to: string): ModuleMove => ({
  from: from.split("::"),
  to: to.split("::"),
});

export const steps: Readonly<Record<string, Step>> = {
  // Step 2: one sibling per markup grammar in L1.
  "l1-markup": {
    crate: "davinci/vize_l1",
    name: "vize_l1",
    moves: [
      move("pug", "markup::pug"),
      move("build", "markup::htmlish::build"),
      move("event", "markup::htmlish::event"),
      move("markup::document", "markup::htmlish::document"),
      move("markup::lex", "markup::htmlish::lex"),
      move("markup::native", "markup::htmlish::native"),
      move("markup::parse", "markup::htmlish::parse"),
      move("markup::render", "markup::htmlish::render"),
      move("markup::surface", "markup::htmlish::surface"),
    ],
    items: ["markup::parse", "markup::render"],
    parents: {
      "markup::htmlish":
        "//! htmlish: the HTML-tokenizer family, one grammar for two hosts.\n" +
        "//!\n" +
        "//! An `.html` document (also in-DOM templates and petite-vue) is read under\n" +
        "//! the Document profile, and an SFC `<template>` under the component profile\n" +
        "//! with Vue's attribute rule. The profiles in [`super::profile`] are the only\n" +
        "//! place the two differ, and `docs/davinci/plan/htmlish-divergences.json`\n" +
        "//! records every difference against each dialect's oracle. This module holds\n" +
        "//! the profile-generic lexer, the lossless surface tree and its builder, the\n" +
        "//! document and component parses, and rendering. Shared markup contracts\n" +
        "//! stay in [`super`].\n",
    },
  },
  // Step 2c: L1 records neutral tag facts (it does not know about lint), and
  // its types and functions are named by concept, not by migration status.
  "l1-tag-facts": {
    crate: "davinci/vize_l1",
    name: "vize_l1",
    moves: [
      move("markup::htmlish::view::lint", "markup::htmlish::view::tag_facts_component"),
      move("markup::htmlish::view::lint_tag", "markup::htmlish::view::tag_facts"),
      move("markup::htmlish::surface::lint_tag", "markup::htmlish::surface::tag_facts"),
    ],
    items: [],
    parents: {},
    identifiers: {
      LINT: "TAG_FACTS",
      LINT_TAGS: "RECORD_TAG_FACTS",
      LintHeaderFact: "HeaderFacts",
      LintTagFact: "TagKindFact",
      NATIVE_SYNTAX_UNIT_LIMIT: "EMBED_SYNTAX_UNIT_LIMIT",
      NativeAttribute: "AttributeView",
      NativeAttributeBindingExpression: "AttributeBindingExpression",
      NativeAttributeBindingExpressionView: "AttributeBindingExpressionView",
      NativeAttributeElse: "AttributeElse",
      NativeAttributeElseView: "AttributeElseView",
      NativeAttributeExpression: "AttributeExpression",
      NativeAttributeExpressionFailure: "AttributeExpressionFailure",
      NativeAttributeExpressionView: "AttributeExpressionView",
      NativeAttributeForHead: "AttributeForHead",
      NativeAttributeForHeadFailure: "AttributeForHeadFailure",
      NativeAttributeForHeadView: "AttributeForHeadView",
      NativeAttributeHandler: "AttributeHandler",
      NativeAttributeHandlerFailure: "AttributeHandlerFailure",
      NativeAttributeHandlerView: "AttributeHandlerView",
      NativeAttributeHead: "AttributeHead",
      NativeAttributeModelExpression: "AttributeModelExpression",
      NativeAttributeModelExpressionView: "AttributeModelExpressionView",
      NativeAttributeOperandError: "AttributeOperandError",
      NativeAttributeValue: "AttributeValueOperand",
      NativeAttributeValueFailure: "AttributeValueFailure",
      NativeAttributeValueView: "AttributeValueView",
      NativeAttributes: "AttributesView",
      NativeChild: "ChildView",
      NativeChildren: "ChildrenView",
      NativeComponent: "ParsedComponent",
      NativeConditionKind: "ConditionKind",
      NativeDocument: "ParsedDocument",
      NativeElement: "ElementView",
      NativeElementClosingName: "ElementClosingName",
      NativeElementNameRefusal: "ElementNameRefusal",
      NativeElementNames: "ElementNames",
      NativeExpressionView: "ExpressionView",
      NativeForHead: "ForHead",
      NativeForInput: "ForInput",
      NativeForInputError: "ForInputError",
      NativeForRefusal: "ForRefusal",
      NativeInterpolationError: "InterpolationError",
      NativeInterpolationFailure: "InterpolationFailure",
      NativeInterpolationOperand: "InterpolationOperand",
      NativeInterpolationView: "InterpolationView",
      NativeLintComponent: "ComponentTagFacts",
      NativeLintTag: "TagFacts",
      NativeLintTagKind: "TagFactsKind",
      NativeLintTagRefusal: "TagFactsRefusal",
      NativeModelHead: "ModelHead",
      NativeRootText: "RootText",
      NativeRootTextError: "RootTextError",
      NativeRootTextProfile: "RootTextProfile",
      NativeRootTextView: "RootTextView",
      NativeScriptSelection: "ScriptSelection",
      NativeStaticBindingHead: "StaticBindingHead",
      NativeSyntax: "EmbedSyntax",
      NativeTemplateComponent: "TemplateComponent",
      NativeTemplateFrameNameRefusal: "TemplateFrameNameRefusal",
      NativeTemplateFrameNames: "TemplateFrameNames",
      NativeTemplateGrammar: "TemplateGrammar",
      NativeTextValue: "TextValue",
      NativeTextValueError: "TextValueError",
      NativeTextValueFailure: "TextValueFailure",
      NativeTextValueView: "TextValueView",
      RejectedNativeForInput: "RejectedForInput",
      construct_with_lint: "construct_with_tag_facts",
      construct_with_lint_and_diagnostics: "construct_with_tag_facts_and_diagnostics",
      is_lint_component: "is_component_tag",
      lint_header: "header_facts",
      lint_header_is_literal: "header_is_literal",
      lint_header_literal: "header_facts_literal",
      lint_in_recovery_context: "in_recovery_context",
      lint_in_table_context: "in_table_context",
      lint_recovery_context: "header_recovery_context",
      lint_table_context: "header_table_context",
      lint_tag: "tag_facts",
      mark_lint_tag: "mark_tag_facts",
      native_refusal: "for_refusal",
      opening_end_with_lint: "opening_end_with_tag_facts",
      parse_component_with_lint: "parse_component_with_tag_facts",
      parse_lint_in: "parse_tag_facts_in",
      selected_lint_tag_aux_never_changes_mode_quotes_width_or_event_layout:
        "selected_tag_facts_aux_never_changes_mode_quotes_width_or_event_layout",
    },
  },
  // Step 2b: L1 modules named by concept, not by migration status (`native`).
  "l1-concepts": {
    crate: "davinci/vize_l1",
    name: "vize_l1",
    moves: [
      // The lexer is the module itself: lex/native.rs becomes lex.rs, beside
      // lex/compat (the byte-identical Armature tokenizer).
      move("markup::htmlish::lex::native", "markup::htmlish::lex"),
      move("markup::htmlish::event::native", "markup::htmlish::event::record"),
      move("markup::htmlish::native", "markup::htmlish::view"),
      move("markup::entity::native", "markup::entity::decoder"),
      move("embed::syntax::for_head::native", "embed::syntax::for_head::parse"),
      // The shared SFC block-boundary scanner; its lexical helpers by concept.
      move("container::vue::compat::block::compat", "container::vue::boundary::block::lexical"),
      move("container::vue::compat", "container::vue::boundary"),
    ],
    items: [],
    parents: {},
  },
};

const ident = "[A-Za-z_][A-Za-z0-9_]*";

const startsWith = (module: readonly string[], prefix: readonly string[]) =>
  prefix.length <= module.length && prefix.every((part, index) => module[index] === part);

/** Where an absolute module path lives after the moves. */
export function relocate(moves: readonly ModuleMove[], module: readonly string[]): string[] {
  for (const { from, to } of moves)
    if (startsWith(module, from)) return [...to, ...module.slice(from.length)];
  return [...module];
}

/** The file and directory a module occupies under `<crate>/src`. */
const moduleFiles = (crate: string, module: readonly string[]) => ({
  file: path.posix.join(crate, "src", ...module) + ".rs",
  dir: path.posix.join(crate, "src", ...module),
});

/** The module of a source file: `src/a/b.rs` and `src/a/b/mod.rs` are `a::b`. */
export function modulePathOf(crate: string, file: string): string[] {
  const inner = path.posix.relative(path.posix.join(crate, "src"), file).replace(/\.rs$/u, "");
  const parts = inner.split("/");
  if (parts.at(-1) === "mod") parts.pop();
  else if (parts.length === 1 && (parts[0] === "lib" || parts[0] === "main")) parts.pop();
  return parts;
}

export function matchingBrace(text: string, open: number): number {
  let depth = 0;
  for (let index = open; index < text.length; index++) {
    if (text[index] === "{") depth++;
    else if (text[index] === "}" && --depth === 0) return index;
  }
  return -1;
}

type Block = { name: string; start: number; end: number };

/** Byte ranges of inline `mod name { … }` blocks. */
function inlineModules(source: string): Block[] {
  const found: Block[] = [];
  const pattern = new RegExp(`\\bmod\\s+(${ident})\\s*\\{`, "gu");
  for (let match = pattern.exec(source); match; match = pattern.exec(source)) {
    const open = match.index + match[0].length - 1;
    const close = matchingBrace(source, open);
    if (close > open) found.push({ name: match[1], start: open, end: close });
  }
  return found;
}

/** The module that code at `offset` is in. */
const moduleAt = (file: readonly string[], inline: readonly Block[], offset: number) => [
  ...file,
  ...inline
    .filter((block) => block.start < offset && offset < block.end)
    .map((block) => block.name),
];

/** The child modules a file declares with `mod name;`. */
const declaredChildren = (source: string) =>
  new Set([...source.matchAll(new RegExp(`\\bmod\\s+(${ident})\\s*;`, "gu"))].map((m) => m[1]));

/**
 * Rewrite one file's Rust paths. A `self::`/`super::` chain, or a bare child
 * path at the start of a `use`, is resolved against its enclosing module; when
 * the moves change what that spelling reaches, it becomes an absolute `crate::`
 * path (or the shortest child path that still works). `crate::` paths, `use`
 * groups and other crates' `<name>::` paths are relocated.
 */
export function rewriteRust(
  source: string,
  step: Step,
  fileModule: readonly string[] | null,
): string {
  let out = source;
  if (fileModule) {
    const children = declaredChildren(out);
    // `self::{…}`/`super::{…}` groups: when the group's base or a member
    // moves, the group becomes an absolute `crate::` group.
    {
      const inline = inlineModules(out);
      const opener = /\b((?:self::|super::)+)\{/gu;
      let result = "";
      let last = 0;
      for (let match = opener.exec(out); match; match = opener.exec(out)) {
        const depth = (match[1].match(/super::/gu) ?? []).length;
        const here = moduleAt(fileModule, inline, match.index);
        if (depth > here.length) continue;
        const base = here.slice(0, here.length - depth);
        const open = match.index + match[0].length - 1;
        const close = matchingBrace(out, open);
        if (close < 0) continue;
        const body = out.slice(open + 1, close);
        const rewritten = rewriteGroupBody(body, base, step);
        const newHere = relocate(step.moves, here);
        const reached = newHere.slice(0, newHere.length - depth);
        const newBase = relocate(step.moves, base);
        if (rewritten === body && reached.join("::") === newBase.join("::")) continue;
        result +=
          out.slice(last, match.index) + ["crate", ...newBase].join("::") + "::{" + rewritten + "}";
        last = close + 1;
        opener.lastIndex = close + 1;
      }
      out = result + out.slice(last);
    }
    const inline = inlineModules(out);
    const chain = new RegExp(`\\b((?:self::|super::)+)((?:${ident}::)*${ident})`, "gu");
    out = out.replace(chain, (whole, relative: string, tail: string, offset: number) => {
      const depth = (relative.match(/super::/gu) ?? []).length;
      const here = moduleAt(fileModule, inline, offset);
      if (depth > here.length) return whole;
      const base = here.slice(0, here.length - depth);
      const terminal = !out.startsWith("::", offset + whole.length);
      const full = [...base, ...tail.split("::")];
      if (terminal && step.items.includes(full.join("::"))) return whole;
      const target = relocate(step.moves, full);
      const newHere = relocate(step.moves, here);
      const newBase = newHere.slice(0, newHere.length - depth);
      if (startsWith(target, newBase) && target.slice(newBase.length).join("::") === tail)
        return whole;
      if (depth === 0 && startsWith(target, newHere))
        return "self::" + target.slice(newHere.length).join("::");
      return ["crate", ...target].join("::");
    });
    // A bare child path, in a `use` or anywhere else (`native::ForOrigin<'a>`).
    const bare = new RegExp(`((?<![\\w:])|\\buse\\s+)((?:${ident}::)+)`, "gu");
    out = out.replace(bare, (whole, lead: string, path: string, offset: number) => {
      const segments = path.slice(0, -2).split("::");
      if (!children.has(segments[0])) return whole;
      const here = moduleAt(fileModule, inline, offset);
      const target = relocate(step.moves, [...here, ...segments]);
      const newHere = relocate(step.moves, here);
      if (startsWith(target, newHere)) return lead + target.slice(newHere.length).join("::") + "::";
      return lead + ["crate", ...target].join("::") + "::";
    });
    out = out.replace(
      new RegExp(`\\bcrate::((?:${ident}::)*${ident})`, "gu"),
      (whole, tail: string, offset: number, all: string) =>
        !all.startsWith("::", offset + whole.length) && step.items.includes(tail)
          ? whole
          : ["crate", ...relocate(step.moves, tail.split("::"))].join("::"),
    );
    out = rewriteGroups(out, "crate", step);
  }
  // Other crates, and the crate's own doc examples, spell the crate's name.
  out = out.replace(
    new RegExp(`\\b${step.name}::((?:${ident}::)+)`, "gu"),
    (_whole, tail: string) =>
      `${step.name}::${relocate(step.moves, tail.slice(0, -2).split("::")).join("::")}::`,
  );
  out = rewriteGroups(out, step.name, step);
  return out;
}

/** `use <root>::a::{b::c, d}`: relocate each member against its group prefix. */
function rewriteGroups(source: string, root: string, step: Step): string {
  const opener = new RegExp(`\\b${root}::((?:${ident}::)*)\\{`, "gu");
  let result = "";
  let last = 0;
  for (let match = opener.exec(source); match; match = opener.exec(source)) {
    const prefix = match[1] ? match[1].slice(0, -2).split("::") : [];
    const open = match.index + match[0].length - 1;
    const close = matchingBrace(source, open);
    if (close < 0) continue;
    result +=
      source.slice(last, open + 1) +
      rewriteGroupBody(source.slice(open + 1, close), prefix, step) +
      "}";
    last = close + 1;
    opener.lastIndex = close + 1;
  }
  return result + source.slice(last);
}

function rewriteGroupBody(inner: string, prefix: readonly string[], step: Step): string {
  const members: string[] = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < inner.length; index++) {
    const char = inner[index];
    if (char === "{") depth++;
    else if (char === "}") depth--;
    else if (char === "," && depth === 0) {
      members.push(inner.slice(start, index));
      start = index + 1;
    }
  }
  members.push(inner.slice(start));
  return members
    .map((member) => {
      const lead = /^\s*/u.exec(member)![0];
      const body = member.slice(lead.length);
      const head = new RegExp(`^((?:${ident}::)+)`, "u").exec(body);
      if (!head) return member;
      const parts = head[1].slice(0, -2).split("::");
      const original = [...prefix, ...parts];
      const relocated = relocate(step.moves, original);
      const rest = body.slice(head[1].length);
      if (relocated.join("::") === original.join("::")) {
        if (!rest.startsWith("{")) return member;
        const close = matchingBrace(rest, 0);
        return (
          lead +
          head[1] +
          "{" +
          rewriteGroupBody(rest.slice(1, close), original, step) +
          rest.slice(close)
        );
      }
      // A moved group prefix carries its members; only their own move shows.
      const newPrefix = relocate(step.moves, prefix);
      if (!startsWith(relocated, newPrefix))
        throw new Error(`a group member leaves its group: ${original.join("::")}`);
      return lead + relocated.slice(newPrefix.length).join("::") + "::" + rest;
    })
    .join(",");
}

/** Word-for-word identifier renames (`\bold\b` becomes `new`). */
export function renameIdentifiers(
  source: string,
  renames: Readonly<Record<string, string>>,
): string {
  const names = Object.keys(renames).sort((a, b) => b.length - a.length);
  if (names.length === 0) return source;
  const pattern = new RegExp(`\\b(?:${names.join("|")})\\b`, "gu");
  return source.replace(pattern, (name) => renames[name] ?? name);
}

/** Repository path strings: `<crate>/src/<old>` becomes `<crate>/src/<new>`. */
export function rewritePaths(source: string, step: Step): string {
  let out = source;
  const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\/]/gu, "\\$&");
  for (const { from, to } of step.moves) {
    const old = moduleFiles(step.crate, from).dir;
    const next = moduleFiles(step.crate, to).dir;
    out = out.replace(new RegExp(`${escape(old)}(?=\\.rs\\b|/)`, "gu"), next);
    // The same path inside a regular expression literal (`a\/b`).
    const slashed = (text: string) => text.replaceAll("/", "\\/");
    out = out.replace(
      new RegExp(`${escape(slashed(old))}(?=\\\\\\.rs|\\\\/|\\(\\?:\\\\\\.rs)`, "gu"),
      slashed(next),
    );
  }
  // Test-name filters (`cargo test -p <crate> --lib a::b::`) name module paths.
  const filter = new RegExp(`(-p ${escape(step.name)}\\b[^\\n]*?\\s)((?:${ident}::)+)`, "gu");
  out = out.replace(
    filter,
    (_whole, lead: string, tail: string) =>
      lead + relocate(step.moves, tail.slice(0, -2).split("::")).join("::") + "::",
  );
  return out;
}

/**
 * `include_str!`/`include_bytes!` and `#[path]` strings are relative to the
 * file. Re-express them from the moved file, following the target if it moved
 * too.
 */
export function rewriteRelativeFiles(
  source: string,
  from: string,
  to: string,
  renames: ReadonlyMap<string, string>,
): string {
  const pattern = /(include_(?:str|bytes)!\(\s*"|#\[path\s*=\s*")([^"]+)(")/gu;
  return source.replace(pattern, (whole, lead: string, relative: string, tail: string) => {
    if (relative.startsWith("/")) return whole;
    const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(from), relative));
    const moved = renames.get(resolved) ?? resolved;
    const next = path.posix.relative(path.posix.dirname(to), moved);
    return lead + next + tail;
  });
}

/** A move into its own parent (`a::b::c` to `a::b`): the child replaces the parent. */
const collapses = ({ from, to }: ModuleMove) =>
  to.length + 1 === from.length && startsWith(from, to);

/** Move `mod name;` declarations from each old parent to its new parent. */
function moveDeclarations(
  step: Step,
  read: (file: string) => string,
  write: (file: string, text: string) => void,
  renames: ReadonlyMap<string, string>,
) {
  // A parent that moves in the same step is edited where it is now.
  const current = (file: string) => [...renames].find(([, moved]) => moved === file)?.[0] ?? file;
  const parentFile = (module: readonly string[]) =>
    module.length === 1
      ? path.posix.join(step.crate, "src/lib.rs")
      : moduleFiles(step.crate, module.slice(0, -1)).file;
  const added = new Map<string, string[]>();
  for (const { from, to } of step.moves) {
    const oldParent = parentFile(from);
    const name = from.at(-1)!;
    const declaration = new RegExp(`^(\\s*)(pub(?:\\([^)]*\\))?\\s+)?mod\\s+${name}\\s*;\\n`, "mu");
    const text = read(oldParent);
    const found = declaration.exec(text);
    if (!found) throw new Error(`no \`mod ${name};\` in ${oldParent}`);
    write(oldParent, text.replace(declaration, ""));
    if (collapses({ from, to })) continue;
    const newParent = current(parentFile(to));
    // A rename in place keeps its visibility; a module that moves to another
    // parent stays reachable from where it was used.
    const kept = found[2]?.trim();
    const visibility =
      newParent === oldParent ? (kept ? kept + " " : "") : kept === "pub" ? "pub " : "pub(crate) ";
    const lines = added.get(newParent) ?? [];
    lines.push(`${visibility}mod ${to.at(-1)};`);
    added.set(newParent, lines);
  }
  for (const [module, doc] of Object.entries(step.parents)) {
    const parts = module.split("::");
    const file = moduleFiles(step.crate, parts).file;
    const grand = parentFile(parts);
    const text = read(grand);
    if (!new RegExp(`\\bmod\\s+${parts.at(-1)}\\s*;`, "u").test(text)) {
      added.set(grand, [...(added.get(grand) ?? []), `pub mod ${parts.at(-1)};`]);
    }
    if (!fs.existsSync(path.join(repoRoot, file))) write(file, doc + "\n");
  }
  for (const [file, lines] of added) {
    const text = read(file);
    const declarations = [...text.matchAll(/^(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;\n/gmu)];
    const at = declarations.length
      ? declarations.at(-1)!.index! + declarations.at(-1)![0].length
      : text.length;
    const insert = lines.sort().join("\n") + "\n";
    write(file, text.slice(0, at) + (declarations.length ? "" : "\n") + insert + text.slice(at));
  }
}

const git = (...args: string[]) =>
  execFileSync("git", args, { cwd: repoRoot, encoding: "utf8", maxBuffer: 1 << 28 }).trim();

const selfFiles = new Set([
  "tools/commands/davinci/level-layout-move.ts",
  "tests/tooling/davinci-level-layout-move.test.ts",
]);

export function run(stepName: string, dryRun: boolean): string[] {
  const step = steps[stepName];
  if (!step) throw new Error(`unknown step ${stepName}; known: ${Object.keys(steps).join(", ")}`);
  const files = git("ls-files", "-z").split("\0").filter(Boolean);
  const renames = new Map<string, string>();
  for (const { from, to } of step.moves) {
    const old = moduleFiles(step.crate, from);
    const next = moduleFiles(step.crate, to);
    for (const file of files) {
      // The first, most specific move of a file wins, as in `relocate`.
      if (renames.has(file)) continue;
      if (file === old.file) renames.set(file, next.file);
      else if (file.startsWith(old.dir + "/"))
        renames.set(file, next.dir + file.slice(old.dir.length));
    }
  }
  if (renames.size === 0) return [];
  const collapsed = step.moves.filter(collapses).map(({ from, to }) => ({
    child: moduleFiles(step.crate, from).file,
    parent: moduleFiles(step.crate, to).file,
    name: from.at(-1)!,
  }));
  const pending = new Map<string, string>();
  const read = (file: string) =>
    pending.get(file) ?? fs.readFileSync(path.join(repoRoot, file), "utf8");
  const write = (file: string, text: string) => void pending.set(file, text);
  for (const file of files) {
    // This script and its test spell every step's old and new paths on purpose.
    if (selfFiles.has(file)) continue;
    const absolute = path.join(repoRoot, file);
    if (!fs.existsSync(absolute)) continue;
    const stat = fs.statSync(absolute);
    if (!stat.isFile() || stat.size > 8_000_000) continue;
    const text = fs.readFileSync(absolute, "utf8");
    if (text.includes("\0")) continue;
    let next = text;
    if (file.endsWith(".rs")) {
      if (file.startsWith(step.crate + "/src/"))
        next = rewriteRust(next, step, modulePathOf(step.crate, file));
      else if (next.includes(step.name + "::")) next = rewriteRust(next, step, null);
    }
    next = rewritePaths(next, step);
    const target = renames.get(file);
    if (target && file.endsWith(".rs")) next = rewriteRelativeFiles(next, file, target, renames);
    if (next !== text) write(file, next);
  }
  moveDeclarations(step, read, write, renames);
  // Identifiers last, once module declarations have moved by their old names.
  if (step.identifiers)
    for (const file of new Set([
      ...files.filter((file) => file.endsWith(".rs")),
      ...pending.keys(),
    ])) {
      // Vendored upstream sources keep their own names.
      if (selfFiles.has(file) || !file.endsWith(".rs") || file.startsWith("upstream/")) continue;
      if (!pending.has(file) && !fs.existsSync(path.join(repoRoot, file))) continue;
      const text = read(file);
      const next = renameIdentifiers(text, step.identifiers);
      if (next !== text) write(file, next);
    }
  const touched = [...pending.keys()].sort();
  if (dryRun) return [...touched, ...[...renames].map(([from, to]) => `${from} -> ${to}`)];
  // A collapsed child replaces its parent file. The parent may hold only its
  // module docs, other module declarations (kept, with their docs) and
  // re-exports of the child (dropped: the items are now defined in place).
  for (const { child, parent, name } of collapsed) {
    const kept: string[] = [];
    let docs: string[] = [];
    const reexport = new RegExp(
      `^\\s*(?:pub(?:\\([^)]*\\))?\\s+)?use\\s+(?:self::)?(?:${name})?::`,
      "u",
    );
    const lines = read(parent).split("\n");
    for (let index = 0; index < lines.length; index++) {
      const line = lines[index];
      if (/^\s*\/\/!/u.test(line) || line.trim() === "") continue;
      if (/^\s*\/\/\//u.test(line)) {
        docs.push(line);
        continue;
      }
      if (/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;/u.test(line)) {
        kept.push(...docs, line);
        docs = [];
        continue;
      }
      if (reexport.test(line)) {
        while (!lines[index].includes(";")) index++;
        docs = [];
        continue;
      }
      throw new Error(`${parent} holds more than declarations and re-exports: ${line}`);
    }
    const body = read(child).replace(/\n*$/u, "\n");
    write(child, kept.length ? body + "\n" + kept.join("\n") + "\n" : body);
    pending.delete(parent);
    git("rm", "-q", parent);
  }
  for (const [file, text] of pending) fs.writeFileSync(path.join(repoRoot, file), text);
  for (const [from, to] of renames) {
    fs.mkdirSync(path.dirname(path.join(repoRoot, to)), { recursive: true });
    git("mv", from, to);
  }
  // `git mv` leaves the emptied directories behind.
  const prune = (directory: string): boolean => {
    if (!fs.existsSync(directory) || !fs.statSync(directory).isDirectory()) return false;
    const empty = fs
      .readdirSync(directory)
      .map((entry) => prune(path.join(directory, entry)))
      .every(Boolean);
    if (empty) fs.rmdirSync(directory);
    return empty;
  };
  for (const { from } of step.moves) prune(path.join(repoRoot, moduleFiles(step.crate, from).dir));
  for (const module of Object.keys(step.parents))
    git("add", moduleFiles(step.crate, module.split("::")).file);
  return [...touched, ...[...renames].map(([from, to]) => `${from} -> ${to}`)];
}

/** A module moved to another crate: `from` and `to` are `{ crate dir, crate name, module path }`. */
export interface CrateModule {
  readonly crate: string;
  readonly name: string;
  readonly module: readonly string[];
}

export interface CrossCrateStep {
  readonly from: CrateModule;
  readonly to: CrateModule;
}

export const crossCrateSteps: Readonly<Record<string, CrossCrateStep>> = {
  // Step 5: TSRX lowering is L1→L2's job, beside the JSX lowering.
  "l2-tsrx-lowering": {
    from: { crate: "davinci/vize_l2", name: "vize_l2", module: ["lang", "tsrx"] },
    to: {
      crate: "davinci/vize_l1_to_l2",
      name: "vize_l1_to_l2",
      module: ["framework", "vue", "tsrx"],
    },
  },
};

/**
 * Rewrite one file for a cross-crate move. Inside the moved module, `crate::`
 * paths into the module follow it and every other `crate::` path names the
 * old crate; `super::` chains that leave the module become absolute paths in
 * the old crate. Everywhere else, the old crate's path to the module becomes
 * the new crate's (`crate::` inside the new crate).
 */
export function rewriteCrossCrate(
  source: string,
  step: CrossCrateStep,
  file: string,
  moved: boolean,
): string {
  const { from, to } = step;
  const fromPath = from.module.join("::");
  const toPath = to.module.join("::");
  let out = source;
  if (moved) {
    const fileModule = modulePathOf(from.crate, file);
    const inline = inlineModules(out);
    out = out.replace(
      new RegExp(`\\b((?:super::)+)((?:${ident}::)*${ident})`, "gu"),
      (whole, supers: string, tail: string, offset: number) => {
        const depth = (supers.match(/super::/gu) ?? []).length;
        const here = moduleAt(fileModule, inline, offset);
        const base = here.slice(0, here.length - depth);
        if (startsWith(base, from.module)) return whole;
        return [from.name, ...base, ...tail.split("::")].join("::");
      },
    );
    out = out.replace(new RegExp(`\\bcrate::`, "gu"), (whole, offset: number) =>
      out.startsWith(fromPath + "::", offset + whole.length) ||
      out.startsWith(fromPath, offset + whole.length)
        ? whole
        : `${from.name}::`,
    );
    out = out.replaceAll(`crate::${fromPath}`, `crate::${toPath}`);
    return out;
  }
  const inTarget = file.startsWith(to.crate + "/src/") || file.startsWith(to.crate + "/tests/");
  const replacement = inTarget && file.startsWith(to.crate + "/src/") ? "crate" : to.name;
  out = out.replace(
    new RegExp(`\\b${from.name}::${fromPath.replaceAll("::", "::")}\\b`, "gu"),
    `${replacement}::${toPath}`,
  );
  return out;
}

export function runCrossCrate(stepName: string, dryRun: boolean): string[] {
  const step = crossCrateSteps[stepName];
  if (!step) throw new Error(`unknown step ${stepName}`);
  const files = git("ls-files", "-z").split("\0").filter(Boolean);
  const old = moduleFiles(step.from.crate, step.from.module);
  const next = moduleFiles(step.to.crate, step.to.module);
  const renames = new Map<string, string>();
  for (const file of files) {
    if (file === old.file) renames.set(file, next.file);
    else if (file.startsWith(old.dir + "/"))
      renames.set(file, next.dir + file.slice(old.dir.length));
  }
  if (renames.size === 0) return [];
  const pending = new Map<string, string>();
  const read = (file: string) =>
    pending.get(file) ?? fs.readFileSync(path.join(repoRoot, file), "utf8");
  for (const file of files) {
    if (selfFiles.has(file)) continue;
    const absolute = path.join(repoRoot, file);
    if (!fs.existsSync(absolute) || !fs.statSync(absolute).isFile()) continue;
    const text = fs.readFileSync(absolute, "utf8");
    if (text.includes("\0")) continue;
    let out = text;
    if (file.endsWith(".rs")) out = rewriteCrossCrate(out, step, file, renames.has(file));
    out = out.replaceAll(old.dir + "/", next.dir + "/").replaceAll(old.file, next.file);
    const target = renames.get(file);
    if (target && file.endsWith(".rs")) out = rewriteRelativeFiles(out, file, target, renames);
    if (out !== text) pending.set(file, out);
  }
  // Declarations: out of the old parent, into the new one.
  const parentOf = (crate: string, module: readonly string[]) =>
    module.length === 1
      ? path.posix.join(crate, "src/lib.rs")
      : (() => {
          const parent = moduleFiles(crate, module.slice(0, -1));
          return fs.existsSync(path.join(repoRoot, parent.file))
            ? parent.file
            : path.posix.join(parent.dir, "mod.rs");
        })();
  const name = step.from.module.at(-1)!;
  const oldParent = parentOf(step.from.crate, step.from.module);
  const declaration = new RegExp(`^(\\s*)(pub(?:\\([^)]*\\))?\\s+)?mod\\s+${name}\\s*;\\n`, "mu");
  const found = declaration.exec(read(oldParent));
  if (!found) throw new Error(`no \`mod ${name};\` in ${oldParent}`);
  pending.set(oldParent, read(oldParent).replace(declaration, ""));
  const newParent = parentOf(step.to.crate, step.to.module);
  const parentText = read(newParent);
  const declarations = [...parentText.matchAll(/^(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;\n/gmu)];
  const at = declarations.length
    ? declarations.at(-1)!.index! + declarations.at(-1)![0].length
    : parentText.length;
  pending.set(
    newParent,
    parentText.slice(0, at) + `pub mod ${step.to.module.at(-1)};\n` + parentText.slice(at),
  );
  const touched = [...pending.keys()].sort();
  const moves = [...renames].map(([from, to]) => `${from} -> ${to}`);
  if (dryRun) return [...touched, ...moves];
  for (const [file, text] of pending) fs.writeFileSync(path.join(repoRoot, file), text);
  for (const [from, to] of renames) {
    fs.mkdirSync(path.dirname(path.join(repoRoot, to)), { recursive: true });
    git("mv", from, to);
  }
  return [...touched, ...moves];
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const [stepName, ...flags] = process.argv.slice(2);
  if (!stepName) {
    console.error(
      `usage: level-layout-move.ts <${[...Object.keys(steps), ...Object.keys(crossCrateSteps)].join("|")}> [--dry-run]`,
    );
    process.exit(2);
  }
  const dryRun = flags.includes("--dry-run");
  const touched =
    stepName in crossCrateSteps ? runCrossCrate(stepName, dryRun) : run(stepName, dryRun);
  console.log(touched.join("\n"));
  console.log(`${touched.length} changes`);
}
