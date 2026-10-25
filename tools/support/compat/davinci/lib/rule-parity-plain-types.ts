// Derive the conditional plain-module type lane from its real guards and work.
// Admission is not evidence that a checker query executed or succeeded.
import { readFileSync } from "node:fs";
import path from "node:path";

import { PATINA_SRC } from "./rule-parity-paths.ts";
import {
  collectStringLiterals,
  matchBraceBlock,
  stripRustComments,
} from "./rule-parity-rust-text.ts";
import { stripRust } from "./rust-source.ts";

export interface PlainTypeDispatch {
  rules: string[];
  extensions: string[];
  declarationSuffixes: string[];
}

export const PLAIN_TYPE_SOURCES = {
  module: "linter/native_type_aware/plain_module.rs",
  prepared: "linter/native_type_aware/plain_module/prepared.rs",
  reactivity: "linter/native_type_aware/reactivity_loss.rs",
  native: "linter/native_type_aware.rs",
  linter: "linter.rs",
  engine: "linter/engine.rs",
  batch: "linter/engine/batch.rs",
  config: "linter/config.rs",
  selection: "linter/rule_selection.rs",
  constructors: "linter/config/constructors.rs",
} as const;

export type PlainTypeSources = Record<keyof typeof PLAIN_TYPE_SOURCES, string>;
const compact = (source: string): string => stripRust(source).replace(/\s+/gu, "");
const fail = (reason: string): never => {
  throw new Error(`plain type dispatch drift: ${reason}`);
};
const requireCode = (source: string, token: string, reason: string): void => {
  if (!compact(source).includes(token)) fail(reason);
};

function body(source: string, name: string): string {
  const code = stripRust(source);
  const matches = [...code.matchAll(new RegExp(`\\bfn\\s+${name}\\b`, "gu"))];
  if (matches.length !== 1) fail(`expected one ${name} function`);
  const open = code.indexOf("{", matches[0].index);
  return stripRustComments(source.slice(open + 1, matchBraceBlock(code, open)));
}

function admission(source: string): Pick<PlainTypeDispatch, "extensions" | "declarationSuffixes"> {
  const code = body(source, "is_plain_type_script");
  const declarationSuffixes = [...code.matchAll(/\bname\.ends_with\(\s*"([^"\n]+)"\s*\)/gu)].map(
    (match) => match[1],
  );
  const pattern = /Some\s*\(\s*\(\s*_\s*,([\s\S]*?)\)\s*\)/u.exec(code);
  if (!pattern || !declarationSuffixes.length) fail("filename admission guards are missing");
  const extensions = collectStringLiterals(pattern[1]);
  if (
    !extensions.length ||
    [...extensions, ...declarationSuffixes].some((s) => !/^[.a-z]+$/u.test(s))
  ) {
    fail("filename admission uses unsupported literals");
  }
  const expected =
    "letname=filename.rsplit([,]).next().unwrap_or(filename);if" +
    declarationSuffixes.map(() => "name.ends_with()").join("||") +
    "{returnfalse;}matches!(name.rsplit_once(),Some((_," +
    "|".repeat(extensions.length - 1) +
    ")))";
  if (compact(code) !== expected) fail("filename admission shape changed");
  const literalCode = code.replace(/\s+/gu, "");
  if (
    !literalCode.startsWith(
      String.raw`letname=filename.rsplit(['/','\\']).next().unwrap_or(filename);`,
    ) ||
    !literalCode.includes("name.rsplit_once('.')")
  ) {
    fail("filename separator boundaries changed");
  }
  if (
    new Set(extensions).size !== extensions.length ||
    new Set(declarationSuffixes).size !== declarationSuffixes.length
  ) {
    fail("filename admission contains ambiguous duplicates");
  }
  return { extensions, declarationSuffixes };
}

/** Validate the admitted lane against actual source, including skipped work. */
export function derivePlainTypeDispatch(
  sources: PlainTypeSources,
  constants: ReadonlyMap<string, string>,
): PlainTypeDispatch {
  const files = admission(sources.module);
  const prepare = body(sources.module, "prepare_plain_module");
  requireCode(
    prepare,
    "if!linter.type_aware_enabled||!is_plain_type_script(filename){returnPreparedPlainModule::new(result,None,false);}",
    "preparation lost the type-aware or filename guard",
  );
  const branches = [
    ...stripRust(prepare).matchAll(
      /let\s+(\w+)\s*=\s*rule_active\(\s*linter\s*,\s*(RULE_[A-Z_]+)\s*\)\s*;/gu,
    ),
  ];
  if (branches.length !== 2 || branches[0][1] !== "promises" || branches[1][1] !== "reactivity") {
    fail("expected the two guarded plain-module query branches");
  }
  const rules = branches.map(
    (match) => constants.get(match[2]) ?? fail(`unknown rule ${match[2]}`),
  );
  if (new Set(rules).size !== 2) fail("plain-module branches must retain distinct rule identities");
  const prefix =
    "if!linter.type_aware_enabled||!is_plain_type_script(filename){returnPreparedPlainModule::new(result,None,false);}" +
    `letpromises=rule_active(linter,${branches[0][2]});letreactivity=rule_active(linter,${branches[1][2]});` +
    "if!promises&&!reactivity{returnPreparedPlainModule::new(result,None,false);}" +
    "letmutslots=Vec::new();" +
    "ifpromises{push_promise_slots(source,filename,&mutslots);}" +
    "ifreactivity{push_reactivity_slots(linter,source,filename,&mutresult,&mutslots);}" +
    "place_slots_after_statements(source,filename,&mutslots);" +
    "ifslots.is_empty(){returnPreparedPlainModule::new(result,None,true);}";
  if (!compact(prepare).startsWith(prefix))
    fail("preparation guards and query branches changed order or reachability");
  requireCode(
    prepare,
    "if!promises&&!reactivity{returnPreparedPlainModule::new(result,None,false);}",
    "inactive rules must skip preparation",
  );
  requireCode(
    prepare,
    "ifpromises{push_promise_slots(source,filename,&mutslots);}",
    "promise branch is no longer reachable",
  );
  requireCode(
    prepare,
    "ifreactivity{push_reactivity_slots(linter,source,filename,&mutresult,&mutslots);}",
    "reactivity branch is no longer reachable",
  );
  requireCode(
    prepare,
    "ifslots.is_empty(){returnPreparedPlainModule::new(result,None,true);}",
    "query-free input must not create a checker plan",
  );
  if (
    compact(body(sources.module, "rule_active")) !==
    "linter.registry.has_rule(rule_name)&&linter.is_rule_enabled(rule_name)"
  ) {
    fail("registration and configured rule selection changed");
  }
  const append = body(sources.module, "append_plain_module_type_diagnostics");
  requireCode(
    append,
    `if!linter.type_aware_enabled||!is_plain_type_script(filename)||!(rule_active(linter,${branches[0][2]})||rule_active(linter,${branches[1][2]})){return;}`,
    "single-file completion lost its admission guards",
  );
  requireCode(
    append,
    "letmutprepared=prepare_plain_module(linter,source,filename,initial);",
    "single-file completion bypasses preparation",
  );
  requireCode(
    append,
    "ifprepared.virtual_content().is_some()",
    "single-file completion starts a session without queries",
  );
  requireCode(
    append,
    "prepared.evaluate(linter,session)",
    "single-file completion lost checker evaluation",
  );
  requireCode(
    body(sources.prepared, "evaluate"),
    "letSome(plan)=&self.planelse{returnOk(());};",
    "query-free evaluation must return without a checker query",
  );
  requireCode(
    body(sources.prepared, "evaluate"),
    "session.probe_types_at_offsets(&plan.content,&requests)?",
    "prepared work no longer queries the checker",
  );
  requireCode(
    body(sources.prepared, "evaluate"),
    "report_probe(&mutself.result,kind,probe.as_ref());",
    "checker results no longer reach the rule branches",
  );
  const report = body(sources.module, "report_probe");
  requireCode(
    report,
    `LintDiagnostic::warn(${branches[0][2]},`,
    "promise branch no longer reports its admitted rule",
  );
  requireCode(
    report,
    "Probe::Reactivity(query)=>{ifshould_warn_for_reactivity_loss(probe){push_warning(result,query.diagnostic(0));}}",
    "reactivity branch no longer reports its checker result",
  );
  requireCode(
    body(sources.reactivity, "diagnostic"),
    `LintDiagnostic::warn(${branches[1][2]},`,
    "reactivity branch no longer reports its admitted rule",
  );

  for (const [key, module] of [
    ["linter", "native_type_aware"],
    ["engine", "batch"],
  ] as const) {
    const declaration = new RegExp(
      `#\\[cfg\\(not\\(target_arch\\s*=\\s*"wasm32"\\)\\)\\]\\s*mod\\s+${module}\\s*;`,
      "gu",
    );
    const source = stripRustComments(sources[key]);
    const matches = [...source.matchAll(declaration)];
    if (
      matches.length !== 1 ||
      !compact(sources[key].slice(matches[0].index)).startsWith(
        `#[cfg(not(target_arch=))]mod${module};`,
      )
    ) {
      fail(`${module} lost its native-only module boundary`);
    }
  }
  const jsx = body(sources.engine, "lint_jsx");
  if (
    compact(jsx) !==
    "letmutresult=self.lint_jsx_without_types(source,filename,lang);" +
      "#[cfg(not(target_arch=))]super::native_type_aware::plain_module::append_plain_module_type_diagnostics(self,source,filename,&mutresult,);" +
      "self.apply_diagnostic_policy(result)"
  ) {
    fail("public JSX completion changed order or reachability");
  }
  requireCode(
    jsx,
    "#[cfg(not(target_arch=))]super::native_type_aware::plain_module::append_plain_module_type_diagnostics(self,source,filename,&mutresult,);",
    "public JSX completion lost its native guarded delegate",
  );
  for (const method of ["prepare_script", "prepare_jsx"]) {
    const entry = body(sources.batch, method);
    requireCode(
      entry,
      "if!native_type_aware::has_active_type_aware_rules(self){returnPreparedLint::complete(self.",
      `${method} lost its inactive-rule fallback`,
    );
    requireCode(
      entry,
      "plan:native_type_aware::plain_module::prepare_plain_module(self,source,filename,result,)",
      `${method} bypasses plain-module preparation`,
    );
  }
  const constructors = [
    ...stripRust(sources.constructors).matchAll(/\btype_aware_enabled\s*:\s*([^,]+),/gu),
  ];
  if (!constructors.length || constructors.some((match) => match[1].trim() !== "false")) {
    fail("type-aware defaults changed");
  }
  requireCode(
    body(sources.config, "with_type_aware_lint"),
    "self.type_aware_enabled=enabled;",
    "host opt-in changed",
  );
  requireCode(
    body(sources.config, "with_rule"),
    "ifis_type_rule(rule_name){self.type_aware_enabled=true;}",
    "explicit type-rule opt-in changed",
  );
  requireCode(
    body(sources.selection, "with_enabled_rules"),
    "ifrules.as_ref().is_some_and(|rules|has_type_rule(rules)){self.type_aware_enabled=true;}",
    "explicit enabled type-rule selection no longer opts in",
  );
  requireCode(
    body(sources.selection, "with_additional_rules"),
    "ifhas_type_rule(&rules){self.type_aware_enabled=true;}",
    "additional type-rule selection no longer opts in",
  );
  if (
    compact(body(sources.config, "is_rule_enabled")) !==
    "ifself.disabled_rules.contains(rule_name){returnfalse;}match&self.enabled_rules{Some(set)=>set.contains(rule_name),None=>true,}"
  ) {
    fail("disabled rules and enabled-rule filters changed");
  }
  const session = body(sources.native, "with_corsa_session");
  for (const token of [
    "effective_tsconfig_for_source(",
    "validate_tsconfig(",
    "returnErr(TypeAwareFailure::ProjectConfiguration{message,report});",
    "CorsaTypeAwareSession::new_with_corsa_path(",
    ".map_err(TypeAwareFailure::Runtime)",
  ]) {
    requireCode(session, token, "project validation or runtime failure handling changed");
  }
  return { ...files, rules: rules.sort() };
}

export function loadPlainTypeDispatch(constants: ReadonlyMap<string, string>): PlainTypeDispatch {
  const sources = Object.fromEntries(
    Object.entries(PLAIN_TYPE_SOURCES).map(([key, relative]) => [
      key,
      readFileSync(path.join(PATINA_SRC, relative), "utf8"),
    ]),
  ) as PlainTypeSources;
  return derivePlainTypeDispatch(sources, constants);
}
