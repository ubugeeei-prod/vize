import { relative } from "node:path";

/**
 * Detect a `vue-tsc` baseline whose *type environment* was not the fixture's own
 * (run 31979524200).
 *
 * `typecheck-baseline-configuration.mjs` catches a baseline that could not load
 * the project, and `typecheck-baseline-coverage.mjs` catches one that checked a
 * different set of `.vue` files. Run 31979524200 breached elk's budget with both
 * of those clean — 259 shared Vue files, zero configuration diagnostics — and
 * reported `711 false positives / 894 false negatives`, ratios 0.996, under the
 * verdict "Vize divergence, the vue-tsc baseline loaded cleanly over the same
 * 259 Vue files". 893 of the 894 were `TS2339` on names elk gets from Nuxt:
 * `$t`, `$d`, `useNuxtApp`, and 50 more of its own auto-imported composables.
 *
 * The fixture's `.nuxt` was fully present and every one of its 26 generated
 * `.d.ts` files was in the program. What was wrong is which `vue` they augmented.
 * `.nuxt/nuxt.d.ts` carries `/// <reference types="vue-router" />`, and a *type
 * reference directive* is resolved by walking `node_modules` upward from the
 * containing file — `compilerOptions.paths` is not consulted, so the mapping
 * Nuxt writes for exactly this package is ignored. `vue-router` is a transitive
 * dependency, so pnpm never links it at `<fixture>/node_modules/vue-router`, the
 * walk left the fixture, and it was satisfied by *Vize's own* `node_modules`:
 *
 *   .../node_modules/.pnpm/vue-router@4.5.1_vue@3.6.0-beta.10_typescript@6.0.3_/...
 *     Type library referenced via 'vue-router' from file '.nuxt/nuxt.d.ts'
 *
 * That dragged Vize's `vue@3.6.0-beta.10` in beside elk's own `vue@3.5.30` — two
 * `vue` module identities, two `@vue/runtime-dom`, 229 foreign files. A
 * `declare module 'vue'` augmentation only takes effect against the identity it
 * resolves to, and with two of them in one program the fixture's own
 * augmentations stopped reaching its components: the instance type collapsed
 * from 680+ members to 21, the Nuxt auto-imports stopped being globals, and the
 * ledger scored the wreckage against Vize. Removing the foreign copies — with
 * the byte-identical generated config and the same `.nuxt` — takes the baseline
 * from 902 diagnostics to 9.
 *
 * So this asserts the one invariant that failure violates and a corpus check
 * cannot see: the packages that own `ComponentCustomProperties` must resolve to
 * exactly one copy, and that copy must be the fixture's. Everything else the
 * program reached outside the fixture is recorded but does not gate — the
 * compiler's own `lib.*.d.ts` and Volar's injected shims are legitimately
 * foreign, and a gate that fires on them is a gate nobody can keep green.
 */

/** The packages whose `ComponentCustomProperties`/`GlobalComponents` a fixture augments. */
const vueRuntimePackages = ["@vue/runtime-core", "@vue/runtime-dom", "vue"];
const nodeModulesMarker = "/node_modules/";

export function evaluateBaselineAmbientEnvironment(vueTscOutput, fixtureRoot) {
  if (typeof vueTscOutput !== "string") throw new Error("vue-tsc output must be a string");
  const prefix = `${normalize(fixtureRoot)}/`;
  const roots = new Map();
  let externalFileCount = 0;
  for (const rawLine of vueTscOutput.replaceAll("\r\n", "\n").split("\n")) {
    const file = normalize(rawLine.trimEnd());
    // `--listFilesOnly` prints absolute program paths. Requiring that shape keeps
    // diagnostic text that happens to mention a path out of the evidence.
    if (!isAbsoluteProgramPath(file)) continue;
    const inside = file.startsWith(prefix);
    if (!inside) externalFileCount += 1;
    const parsed = parsePackageRoot(file);
    if (parsed == null) continue;
    if (!roots.has(parsed.name)) roots.set(parsed.name, new Map());
    roots.get(parsed.name).set(parsed.root, parsed.root.startsWith(prefix));
  }
  const vueRuntime = vueRuntimePackages
    .filter((name) => roots.has(name))
    .map((name) => describePackage(name, roots.get(name), fixtureRoot));
  const externalPackages = [...roots]
    .filter(([, copies]) => [...copies.values()].includes(false))
    .map(([name]) => name)
    .sort(codeUnitOrder);
  const unusableReason = firstAmbientFailure(vueRuntime);
  return {
    externalFileCount,
    externalPackages,
    unusableReason,
    verdict: unusableReason == null ? "isolated" : "contaminated",
    vueRuntime,
  };
}

function describePackage(name, copies, fixtureRoot) {
  const paths = [...copies.keys()].sort(codeUnitOrder);
  return {
    name,
    copies: paths.map((root) => ({
      path: relative(fixtureRoot, root).replaceAll("\\", "/"),
      insideFixture: copies.get(root),
    })),
    insideFixtureCount: paths.filter((root) => copies.get(root)).length,
    outsideFixtureCount: paths.filter((root) => !copies.get(root)).length,
  };
}

/**
 * Duplication is reported before absence: two copies is the failure that reads
 * as a working baseline, while zero fixture-owned copies is already visible as a
 * wall of unresolved-name diagnostics.
 */
function firstAmbientFailure(vueRuntime) {
  for (const entry of vueRuntime) {
    if (entry.copies.length > 1) {
      return (
        `vue-tsc resolved ${entry.copies.length} copies of '${entry.name}' into the baseline ` +
        `program (${entry.copies.map((copy) => copy.path).join(", ")}), so the fixture's own ` +
        "module augmentations merged into a different module identity than its components"
      );
    }
  }
  for (const entry of vueRuntime) {
    if (entry.insideFixtureCount === 0) {
      return (
        `vue-tsc resolved '${entry.name}' outside the fixture (${entry.copies[0].path}), so the ` +
        "baseline measured a type environment the fixture does not own"
      );
    }
  }
  return null;
}

/**
 * pnpm stores a package at `<store>/.pnpm/<id>/node_modules/<name>`, so the last
 * `node_modules` segment is the one that names the package. Anything still under
 * `.pnpm` after that is a store path rather than a resolved package and is
 * skipped instead of being reported as a package called `.pnpm`.
 */
function parsePackageRoot(file) {
  const index = file.lastIndexOf(nodeModulesMarker);
  if (index < 0) return null;
  const base = index + nodeModulesMarker.length;
  const segments = file.slice(base).split("/");
  if (segments[0] === ".pnpm" || segments[0] === "") return null;
  const name = segments[0].startsWith("@")
    ? segments.length > 1 && segments[1] !== ""
      ? `${segments[0]}/${segments[1]}`
      : null
    : segments[0];
  if (name == null) return null;
  return { name, root: `${file.slice(0, base)}${name}` };
}

/**
 * The comparison `Array.prototype.sort` already performs on strings, written
 * out. Spelled explicitly because these orderings are artifact content — a
 * locale-sensitive comparator would make the same program hash differently on
 * two runners — and because the workspace lint budget is zero warnings.
 */
function codeUnitOrder(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function isAbsoluteProgramPath(file) {
  return file.startsWith("/") || /^[A-Za-z]:\//u.test(file);
}

function normalize(value) {
  return value.replaceAll("\\", "/").replace(/\/+$/u, "");
}
