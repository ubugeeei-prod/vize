/** Forward the frozen preamble edits that postdate the overload-alias certificate. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

// Chronological source-owned pairs. Each historical snippet occurs once.
const TRANSITIONS: ReadonlyArray<readonly [string, string]> = [
  [
    "type $ComponentPublicBase = Omit<import('vue').ComponentPublicInstance, '$props' | '$emit' | '$slots'>;\n",
    "type $ComponentPublicBase<I = import('vue').ComponentPublicInstance> = Omit<I, '$props' | '$emit' | '$slots'> & Pick<I, Exclude<$LiteralKeys<I>, '$props' | '$emit' | '$slots'>>;\n",
  ],
  [
    "as $HandlerKey<K> | `on${Capitalize<K>}`]",
    "as ($Camelize<K> extends keyof __EmitOptions<T> ? string extends keyof __EmitOptions<T> ? $HandlerKey<K> : never : $HandlerKey<K>) | `on${Capitalize<K>}`]",
  ],
  [
    "type $EmitRequiredFields<T> = T extends unknown ? Pick<T, $EmitRequiredKeys<T>> : never;",
    "type $EmitRequiredFields<T> = T extends unknown ? {} extends T ? Pick<T, never> : Pick<T, $EmitRequiredKeys<T>> : never;",
  ],
  [
    "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n\n",
    "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n// vize checks these templates itself, so component input guards serve only\n// TS and TSX callers and keep extra keys exact.\ndeclare var __vizeTemplateChecker: true;\n\n",
  ],
  [
    "K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>>",
    "K extends keyof Authored = Exclude<Exclude<keyof Authored, keyof Allowed>, typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never } ? keyof P : never>>",
  ],
  [
    "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;",
    "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;",
  ],
];

const INTEGRATED_TRANSITIONS: ReadonlyArray<readonly [string, string]> = [
  [
    "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n// vize checks these templates itself, so component input guards serve only\n// TS and TSX callers and keep extra keys exact.\ndeclare var __vizeTemplateChecker: true;\n\n",
    "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n\n",
  ],
  [
    "K extends keyof Authored = Exclude<Exclude<keyof Authored, keyof Allowed>, typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never } ? keyof P : never>>",
    "K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>>",
  ],
  [
    "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n",
    "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n// Constructor arguments (templates, `new`) bind `Authored`; TSX binds `Jsx` through `$props` (#11629).\ntype $ComponentInstanceFor<I, T, Declared, Authored, Jsx> = $IsAny<Authored> extends true ? I : $IsUnknown<Authored> extends true ? $UsePublicInstance<Jsx> extends true ? I : $ReplaceComponentProps<I, Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> & $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Jsx>> : $ReplaceComponentProps<I, $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Authored>>;\n",
  ],
];

const previousOverloadPath = "tests/tooling/fixtures/emit-overload-transition/before.d.ts.txt";
const currentOverloadPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const previousOverloadSha256 = "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5";
const currentOverloadSha256 = "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74";
const overloadPrefix = "type $OverloadUnionRecursive<";

export const AUTHENTICATED_PREAMBLE_BYTES = 44_748;
export const AUTHENTICATED_PREAMBLE_SHA256 =
  "378516f3c59cecb166ab9197761b77679824ea7145efd84d752bf5afd3060b07";

export function applySourceOwnedPreambleTransitions(historical: string) {
  let current = historical;
  for (const [before, after] of TRANSITIONS) {
    assert.equal(
      current.split(before).length,
      2,
      `missing frozen preamble edit ${before.slice(0, 48)}`,
    );
    assert.equal(current.includes(after), false, `preamble already contains ${after.slice(0, 48)}`);
    current = current.replace(before, after);
  }
  return current;
}

/** Apply source-owned changes that follow the immutable emit and slot certificates. */
export function applyIntegratedSourcePreambleTransitions(certified: string) {
  let current = certified;
  for (const [before, after] of INTEGRATED_TRANSITIONS) {
    assert.equal(
      current.split(before).length,
      2,
      `missing integrated preamble edit ${before.slice(0, 48)}`,
    );
    assert.equal(current.includes(after), false, `preamble already contains ${after.slice(0, 48)}`);
    current = current.replace(before, after);
  }
  return current;
}

type Identity = { bytes: number; sha256: string };
type EmitPreambleCertificate = {
  files: { path: string; after: Identity }[];
  preamble: {
    before: Identity;
    after: Identity;
    edits: { before: string; after: string }[];
  };
};

const emitCertificate =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/canonical-loose-emit-sdk-update-11815.json";
const emitCertificateSha256 = "b39a4a47a11d2e343c2cec25a899830c0ac13403b9445d2bc73715f533e22247";
const emitAssets = [
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
];
const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

function authenticatedOverloadAliases(read: (file: string) => string) {
  const selected = (file: string, expected: string) => {
    const text = read(file);
    assert.equal(identity(text).sha256, expected, `authenticated overload asset: ${file}`);
    const lines = text.split("\n").filter((line) => line.startsWith(overloadPrefix));
    assert.equal(lines.length, 1, `one recursive overload alias: ${file}`);
    return lines[0]!;
  };
  return {
    previous: selected(previousOverloadPath, previousOverloadSha256),
    current: selected(currentOverloadPath, currentOverloadSha256),
  };
}

/** Advance the immutable slot evidence to the authenticated PR-base overload helper. */
export function applyPreviousOverloadPreamble(
  historical: string,
  read = (file: string) => fs.readFileSync(file, "utf8"),
) {
  const { previous, current } = authenticatedOverloadAliases(read);
  const historicalAliases = historical
    .split("\n")
    .filter((line) => line.startsWith(overloadPrefix));
  assert.equal(historicalAliases.length, 1, "one historical recursive overload alias");
  const original = historicalAliases[0]!;
  assert.notEqual(original, previous);
  assert.notEqual(original, current);
  assert.equal(historical.split(original).length, 2);
  assert.equal(historical.includes(previous), false);
  assert.equal(historical.includes(current), false);
  return historical.replace(original, previous);
}

/** Apply the separately certified overload-helper successor after other preamble certificates. */
export function applyCurrentOverloadPreamble(
  previousPreamble: string,
  read = (file: string) => fs.readFileSync(file, "utf8"),
) {
  const { previous, current } = authenticatedOverloadAliases(read);
  assert.equal(previousPreamble.split(previous).length, 2);
  assert.equal(previousPreamble.includes(current), false);
  return previousPreamble.replace(previous, current);
}

/** Advance only the already certified emit-helper layout for the current source CLI. */
export function applyCurrentEmitPreamble(
  previous: string,
  read = (file: string) => fs.readFileSync(file, "utf8"),
) {
  const raw = read(emitCertificate);
  assert.equal(identity(raw).sha256, emitCertificateSha256, "immutable emit preamble certificate");
  const certificate = JSON.parse(raw) as EmitPreambleCertificate;
  assert.deepEqual(identity(previous), certificate.preamble.before, "certified previous preamble");
  for (const file of emitAssets) {
    const entries = certificate.files.filter((entry) => entry.path === file);
    assert.equal(entries.length, 1, "one selected canonical emit asset");
    assert.deepEqual(
      identity(read(file)),
      entries[0]!.after,
      `selected canonical emit source: ${file}`,
    );
  }
  let current = previous;
  for (const edit of certificate.preamble.edits) {
    assert.ok(edit.before.length > 0);
    assert.equal(current.split(edit.before).length, 2, "one certified emit preamble replacement");
    current = current.replace(edit.before, edit.after);
  }
  assert.deepEqual(identity(current), certificate.preamble.after, "certified current preamble");
  return current;
}
