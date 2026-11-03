// A private ScriptRule callback has its registered parent's identity, not a
// second META/registration. Prove the complete direct-module/field association.
import path from "node:path";

import type { ParsedRuleFile } from "./rule-parity-rules.ts";
import { matchBraceBlock } from "./rule-parity-rust-text.ts";
import { expandUseTree, findUseDecls } from "./rust-source.ts";

export interface PrivateScriptAssociation {
  callback: ParsedRuleFile;
  parent: ParsedRuleFile;
  type: string;
  field: string;
}

export interface PrivateScripts {
  byCallback: Map<string, PrivateScriptAssociation>;
  byParent: Map<string, PrivateScriptAssociation>;
}

const ID = "[A-Za-z_][A-Za-z0-9_]*";
const escape = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const depthAt = (source: string, end: number | undefined): number => {
  let depth = 0;
  for (const ch of source.slice(0, end)) {
    if (ch === "{") depth++;
    else if (ch === "}") depth--;
  }
  return depth;
};
const topMatches = (source: string, regex: RegExp): RegExpExecArray[] =>
  [...source.matchAll(regex)].filter((match) => depthAt(source, match.index) === 0);
const hasConditionalAttributes = (source: string, index: number | undefined): boolean => {
  let prefix = source.slice(0, index).trimEnd();
  while (prefix.endsWith("]")) {
    let depth = 1;
    let start = prefix.length - 2;
    for (; start >= 0 && depth; start--) {
      if (prefix[start] === "]") depth++;
      else if (prefix[start] === "[") depth--;
    }
    if (depth || prefix[start] !== "#") break;
    if (/^#\[\s*(?:cfg|cfg_attr|path)\b/.test(prefix.slice(start))) return true;
    prefix = prefix.slice(0, start).trimEnd();
  }
  return false;
};

export function collectPrivateScriptCallbacks(
  files: readonly ParsedRuleFile[],
  registeredTypes: ReadonlySet<string>,
): PrivateScripts {
  const byCallback = new Map<string, PrivateScriptAssociation>();
  const byParent = new Map<string, PrivateScriptAssociation>();
  for (const callback of files.filter(
    (file) => !file.meta && file.impls.some((block) => block.trait === "ScriptRule"),
  )) {
    const fail: (reason: string) => never = (reason) => {
      throw new Error(`rule trait impl without META in ${callback.rel}: ${reason}`);
    };
    const one = <T>(rows: readonly T[], reason: string): T => {
      if (rows.length !== 1) fail(reason);
      return rows[0];
    };
    const impl = one(callback.impls, "ambiguous callback rule traits");
    if (impl.trait !== "ScriptRule" || registeredTypes.has(impl.type)) {
      fail("callback is independently registered or is not private ScriptRule");
    }
    const ownerStem = path.posix.dirname(callback.rel);
    const parent = one(
      files.filter((file) => [ownerStem + ".rs", ownerStem + "/mod.rs"].includes(file.rel)),
      "missing or ambiguous direct module owner",
    );
    if (parent.meta?.kind !== "RuleMeta") fail("owner has no canonical RuleMeta");
    const parentMeta = one(
      topMatches(
        parent.source,
        /^[ \t]*(?:pub(?:\s*\([^)]*\))?\s+)?static\s+META\s*:\s*RuleMeta\s*=/gm,
      ),
      "ambiguous owner META",
    );
    if (hasConditionalAttributes(parent.source, parentMeta.index)) fail("conditional owner META");
    const parentImpl = one(
      parent.impls.filter((block) => block.trait === "Rule"),
      "ambiguous parent Rule implementation",
    );
    if (!registeredTypes.has(parentImpl.type) || parentImpl.fns.get("meta")?.trim() !== "&META") {
      fail("owner is not registered with its canonical META");
    }
    const module = path.posix.basename(callback.rel, ".rs");
    const declaration = one(
      topMatches(
        parent.source,
        new RegExp(`^[ \\t]*(pub(?:\\s*\\([^)]*\\))?\\s+)?mod\\s+${escape(module)}\\s*;`, "gm"),
      ),
      "missing or ambiguous direct callback module",
    );
    if (declaration[1] || hasConditionalAttributes(parent.source, declaration.index)) {
      fail("callback module is public, conditional or has a foreign path");
    }
    const owner = one(
      topMatches(
        parent.source,
        new RegExp(
          `^[ \\t]*(?:pub(?:\\s*\\([^)]*\\))?\\s+)?struct\\s+${escape(parentImpl.type)}\\s*\\{`,
          "gm",
        ),
      ),
      "missing or ambiguous owner fields",
    );
    const open = parent.source.indexOf("{", owner.index);
    const close = matchBraceBlock(parent.source, open);
    if (close < 0) fail("unterminated owner fields");
    const fields = parent.source.slice(open + 1, close);
    const field = one(
      [
        ...fields.matchAll(
          new RegExp(
            `(?:^|,)\\s*(${ID})\\s*:\\s*${escape(module)}\\s*::\\s*${escape(impl.type)}\\s*(?=,|$)`,
            "g",
          ),
        ),
      ],
      "missing or ambiguous typed callback field",
    )[1];
    if ([...fields.matchAll(new RegExp(`(?:^|,)\\s*${escape(field)}\\s*:`, "g"))].length !== 1) {
      fail("ambiguous owner field binding");
    }
    const hook = parentImpl.fns.get("as_sfc_script_rule") ?? "";
    if (
      !new RegExp(`^\\s*Some\\s*\\(\\s*&\\s*self\\s*\\.\\s*${escape(field)}\\s*\\)\\s*$`).test(hook)
    ) {
      fail("owner does not return exactly its typed callback field");
    }
    const privateType = one(
      topMatches(
        callback.source,
        new RegExp(`^[ \\t]*(pub(?:\\s*\\([^)]*\\))?\\s+)?struct\\s+${escape(impl.type)}\\b`, "gm"),
      ),
      "missing or ambiguous callback type",
    );
    if (privateType[1] && privateType[1].replace(/\s/g, "") !== "pub(super)") {
      fail("callback type is publicly exposed");
    }
    const declarations = findUseDecls(callback.source).filter(
      (decl) => depthAt(callback.source, decl.start) === 0,
    );
    const imports = declarations.flatMap((decl) => expandUseTree(decl.body));
    const metaImport = one(
      imports.filter((entry) => entry.segments.join("::") === "super::META" && !entry.glob),
      "missing or ambiguous imported parent META",
    );
    const metaName = metaImport.alias ?? "META";
    if (
      imports.filter((entry) => (entry.alias ?? entry.segments.at(-1)) === metaName).length !== 1 ||
      new RegExp(`\\b(?:static|const|let)\\s+(?:mut\\s+)?${escape(metaName)}\\b`).test(
        callback.source,
      )
    ) {
      fail("parent META import is shadowed");
    }
    const metadata = one(
      topMatches(
        callback.source,
        new RegExp(
          `^[ \\t]*(?:pub(?:\\s*\\([^)]*\\))?\\s+)?static\\s+(${ID})\\s*:\\s*ScriptRuleMeta\\s*=\\s*ScriptRuleMeta\\s*\\{`,
          "gm",
        ),
      ),
      "missing or ambiguous callback metadata projection",
    );
    const parentTrait = one(
      topMatches(
        parent.source,
        new RegExp(`^[ \\t]*impl\\s+Rule\\s+for\\s+${escape(parentImpl.type)}\\b`, "gm"),
      ),
      "ambiguous owner trait source",
    );
    const callbackTrait = one(
      topMatches(
        callback.source,
        new RegExp(`^[ \\t]*impl\\s+ScriptRule\\s+for\\s+${escape(impl.type)}\\b`, "gm"),
      ),
      "ambiguous callback trait source",
    );
    if (
      hasConditionalAttributes(parent.source, owner.index) ||
      hasConditionalAttributes(parent.source, parentTrait.index) ||
      [privateType.index, metadata.index, callbackTrait.index].some((index) =>
        hasConditionalAttributes(callback.source, index),
      ) ||
      declarations.some((decl) => hasConditionalAttributes(callback.source, decl.start))
    ) {
      fail("conditional callback ownership or metadata source");
    }
    const metaOpen = callback.source.indexOf("{", metadata.index);
    const metaClose = matchBraceBlock(callback.source, metaOpen);
    if (metaClose < 0) fail("unterminated callback metadata");
    const projections = callback.source
      .slice(metaOpen + 1, metaClose)
      .split(",")
      .map((part) => part.trim())
      .filter(Boolean);
    const expected = ["name", "description", "default_severity"].sort();
    const keys = projections.map((part) => {
      const projection = new RegExp(`^(${ID})\\s*:\\s*(${ID})\\s*\\.\\s*(${ID})$`).exec(part);
      if (!projection || projection[2] !== metaName || projection[1] !== projection[3]) {
        fail("callback metadata does not exactly project the owner's META");
      }
      return projection[1];
    });
    if (
      keys.sort().join(",") !== expected.join(",") ||
      impl.fns.get("meta")?.trim() !== `&${metadata[1]}`
    ) {
      fail("callback metadata does not exactly project the owner's META");
    }
    if (byParent.has(parent.rel)) fail("multiple callbacks claim one parent capability");
    const association = { callback, parent, type: impl.type, field };
    byCallback.set(callback.rel, association);
    byParent.set(parent.rel, association);
  }
  for (const parent of files.filter((file) => file.meta?.kind === "RuleMeta")) {
    for (const impl of parent.impls.filter((block) => block.trait === "Rule")) {
      const hook = impl.fns.get("as_sfc_script_rule");
      if (hook !== undefined && hook.trim() !== "None" && !byParent.has(parent.rel)) {
        throw new Error(`unproven SFC script callback in ${parent.rel}`);
      }
    }
  }
  return { byCallback, byParent };
}
