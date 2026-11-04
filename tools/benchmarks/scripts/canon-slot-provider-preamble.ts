/** Forward the frozen preamble edits that postdate the overload-alias certificate. */
import assert from "node:assert/strict";

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
