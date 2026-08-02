import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  onigurumaModulePath,
  onigurumaWasmPath,
  shikiLanguageModulePath,
  textmateDependencyVersions,
  textmateModulePath,
} from "./textmate-deps.ts";
import { byteOrder, canonicalJson, sha256 } from "./syntax-evidence.ts";
import type { TextMateGrammar } from "./vue-textmate.ts";

const packageName = "@shikijs/langs";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const packageVersion = "4.0.2";
const vueModuleSha256 = "610450422f0a3b39468c42c078ff9e2dfd2d55045d31bbbb95173eba5986962d";
const licenseSha256 = "7a9d8d01038aeacf9e5bcdabbddf2a7815200dce9fc1118468cc553e00ae3eee";
const grammarClosureSha256 = "9d6e6d7c4109574dec2aedcd2adb501b327c7e69943ead3adfee0ecccd379960";
export const shikiVueOracleProvenance = {
  grammarClosureSha256,
  licenseSha256,
  module: `${packageName}/vue`,
  moduleSha256: vueModuleSha256,
  package: packageName,
  version: packageVersion,
} as const;
type BundledGrammar = {
  embeddedLangsLazy?: string[];
  injectTo?: string[];
  scopeName?: string;
};

export type OracleEvidence = {
  configuredGrammarSha256: string;
  dependencyVersions: typeof textmateDependencyVersions;
  grammarClosureSha256: string;
  licenseSha256: string;
  module: string;
  moduleSha256: string;
  package: string;
  requestedScopes: string[];
  rootScope: string;
  unresolvedScopeSentinels: string[];
  version: string;
};

export async function loadPinnedShikiVueOracle(
  services: {
    modulePath?: string;
    readFile?: typeof fs.readFileSync;
  } = {},
) {
  const modulePath = services.modulePath ?? shikiLanguageModulePath("vue");
  const readFile = services.readFile ?? fs.readFileSync;
  const manifestPath = findPackageManifest(modulePath, packageName, readFile);
  const manifest = JSON.parse(readFile(manifestPath, "utf8") as string) as {
    license?: string;
    name?: string;
    version?: string;
  };
  const licensePath = path.join(path.dirname(manifestPath), "LICENSE");
  const moduleSha = sha256(readFile(modulePath));
  const licenseSha = sha256(readFile(licensePath));
  validateOraclePin({
    license: manifest.license,
    licenseSha256: licenseSha,
    moduleSha256: moduleSha,
    name: manifest.name,
    version: manifest.version,
  });

  const [{ Registry }, { createOnigurumaEngine }, vueModule] = await Promise.all([
    import(pathToFileURL(textmateModulePath).href),
    import(pathToFileURL(onigurumaModulePath).href),
    import(pathToFileURL(modulePath).href),
  ]);
  const engine = await createOnigurumaEngine(readFile(onigurumaWasmPath));
  // The Vue grammar contains a historical PostCSS include that is not listed
  // in its generated embeddedLangsLazy metadata.
  const grammarList = await loadLanguageClosure(vueModule.default, ["postcss", "twig"]);
  validateOracleGrammarClosure(grammarList);
  const grammars = new Map<string, BundledGrammar>();
  for (const grammar of grammarList) {
    if (typeof grammar.scopeName === "string") grammars.set(grammar.scopeName, grammar);
  }
  const unresolvedScopeSentinels = new Set<string>();
  grammars.set(
    "source.js.regexp",
    readJsonGrammar("tests/tooling/fixtures/javascript-regexp.tmLanguage.json"),
  );
  grammars.set("source.sassdoc", readJsonGrammar("tests/tooling/fixtures/sassdoc.tmLanguage.json"));
  addScopeAlias(grammars, "source.postcss", "source.css.postcss");
  addScopeAlias(grammars, "source.less", "source.css.less");
  addScopeAlias(grammars, "source.twig", "text.html.twig");
  const rootScope = "text.html.vue";
  const rootGrammar = grammarList.find(
    (grammar) => grammar.scopeName === rootScope && (grammar as { name?: string }).name === "vue",
  );
  assert.ok(rootGrammar, `pinned ${packageName}/vue has no ${rootScope} grammar`);
  grammars.set(rootScope, rootGrammar);
  const requestedScopes = new Set<string>();
  const injections = new Map<string, string[]>();
  for (const grammar of grammarList) {
    if (typeof grammar.scopeName !== "string") continue;
    for (const target of grammar.injectTo ?? []) {
      const values = injections.get(target) ?? [];
      if (!values.includes(grammar.scopeName)) values.push(grammar.scopeName);
      injections.set(target, values);
    }
  }
  const registry = new Registry({
    getInjections(scopeName: string) {
      return [...(injections.get(scopeName) ?? [])].sort(byteOrder);
    },
    loadGrammar(scopeName: string) {
      requestedScopes.add(scopeName);
      let grammar = grammars.get(scopeName);
      if (grammar == null) {
        unresolvedScopeSentinels.add(scopeName);
        grammar = unresolvedSentinelGrammar(scopeName);
        grammars.set(scopeName, grammar);
      }
      return grammar;
    },
    onigLib: {
      createOnigScanner(patterns: Array<string | RegExp>) {
        return engine.createScanner(patterns);
      },
      createOnigString(value: string) {
        return engine.createString(value);
      },
    },
  });
  let grammar: TextMateGrammar | null;
  try {
    grammar = registry.loadGrammar(rootScope) as TextMateGrammar | null;
    assert.ok(grammar, `failed to load pinned oracle grammar ${rootScope}`);
  } catch (error) {
    registry.dispose();
    throw error;
  }
  return {
    getEvidence: (): OracleEvidence => ({
      configuredGrammarSha256: sha256(
        JSON.stringify([...grammars.entries()].sort(([left], [right]) => byteOrder(left, right))),
      ),
      dependencyVersions: textmateDependencyVersions,
      grammarClosureSha256,
      licenseSha256: licenseSha,
      module: `${packageName}/vue`,
      moduleSha256: moduleSha,
      package: packageName,
      requestedScopes: [...requestedScopes].sort(byteOrder),
      rootScope,
      unresolvedScopeSentinels: [...unresolvedScopeSentinels].sort(byteOrder),
      version: manifest.version as string,
    }),
    grammar,
    registry,
    rootScope,
  };
}

export function validateOracleGrammarClosure(value: unknown): void {
  assert.ok(Array.isArray(value) && value.length > 0, "oracle grammar closure must be non-empty");
  assert.equal(
    sha256(canonicalJson(value)),
    grammarClosureSha256,
    "oracle grammar closure drifted",
  );
}

export function validateOraclePin(evidence: {
  license?: string;
  licenseSha256: string;
  moduleSha256: string;
  name?: string;
  version?: string;
}): void {
  assert.deepEqual(
    evidence,
    {
      license: "MIT",
      licenseSha256,
      moduleSha256: vueModuleSha256,
      name: packageName,
      version: packageVersion,
    },
    "pinned @shikijs/langs/vue oracle provenance drifted",
  );
}

function findPackageManifest(
  modulePath: string,
  expectedName: string,
  readFile: typeof fs.readFileSync = fs.readFileSync,
): string {
  let directory = path.dirname(modulePath);
  while (directory !== path.dirname(directory)) {
    const candidate = path.join(directory, "package.json");
    if (fs.existsSync(candidate)) {
      const manifest = JSON.parse(readFile(candidate, "utf8") as string) as { name?: string };
      if (manifest.name === expectedName) return candidate;
    }
    directory = path.dirname(directory);
  }
  throw new Error(`could not resolve ${expectedName} provenance from ${modulePath}`);
}

function addScopeAlias(
  grammars: Map<string, BundledGrammar>,
  alias: string,
  canonical: string,
): void {
  if (!grammars.has(alias) && grammars.has(canonical))
    grammars.set(alias, grammars.get(canonical)!);
}

async function loadLanguageClosure(
  initial: unknown,
  requiredNames: string[],
): Promise<BundledGrammar[]> {
  const grammars = (Array.isArray(initial) ? initial : [initial]).filter(
    (grammar): grammar is BundledGrammar => grammar != null && typeof grammar === "object",
  );
  const loadedNames = new Set(["vue"]);
  let pending = [
    ...requiredNames.filter((name) => {
      if (loadedNames.has(name)) return false;
      loadedNames.add(name);
      return true;
    }),
    ...collectLazyNames(grammars, loadedNames),
  ].sort(byteOrder);
  while (pending.length > 0) {
    const names = pending;
    pending = [];
    const modules = await Promise.all(
      names.map((name) => import(pathToFileURL(shikiLanguageModulePath(name)).href)),
    );
    const added = modules
      .flatMap((module) => (Array.isArray(module.default) ? module.default : [module.default]))
      .filter(
        (grammar): grammar is BundledGrammar => grammar != null && typeof grammar === "object",
      );
    grammars.push(...added);
    pending.push(...collectLazyNames(added, loadedNames));
  }
  return grammars;
}

function collectLazyNames(grammars: BundledGrammar[], loadedNames: Set<string>): string[] {
  const names = new Set<string>();
  for (const grammar of grammars) {
    for (const name of grammar.embeddedLangsLazy ?? []) {
      if (loadedNames.has(name)) continue;
      loadedNames.add(name);
      names.add(name);
    }
  }
  return [...names].sort(byteOrder);
}

function unresolvedSentinelGrammar(scopeName: string): BundledGrammar {
  return {
    scopeName,
    patterns: [
      {
        match: ".+",
        name: `invalid.unresolved-oracle-grammar.${scopeName}`,
      },
    ],
  } as BundledGrammar;
}

function readJsonGrammar(relativePath: string): BundledGrammar {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8")) as BundledGrammar;
}
