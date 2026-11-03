import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const originals = [
  [
    "integration__build_declaration_emit_cli__build_dts_emits_declarations_alongside_compiled_js",
    "b3700c0bf6219fdfa83fd4c84daed401d150f0116975cd77b41b8d0e0ca55290",
    "284aee47116f72565448c0b2dd06153349e439e476ba8917626b302c1c21ed05",
  ],
  [
    "integration__check_cli__check_can_emit_declarations",
    "5705ceab9598e11053117bbd6c06b2f7d332ba8e1e5b3617ff03983fb0ff0d20",
    "f1c1d227a6106d762a91c62c28c8aabdef4db265b7841d752f22a6c96765de07",
  ],
] as const;

const helper = fs.readFileSync(
  path.join(
    repoRoot,
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
  ),
  "utf8",
);
assert.equal(
  createHash("sha256").update(helper).digest("hex"),
  "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5",
);
const recursive = helper.split("\n")[1];
assert.ok(recursive);
const currentRecursive: string = recursive;
const historicalRecursive =
  "type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = []> = TDepth['length'] extends 32 ? never : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? never : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown]> | ((...args: TArgs) => TReturn) : never;";

// #11445 later key-remapped the public instance base; invert it first.
const currentPublicBase =
  "type $ComponentPublicBase<I = import('vue').ComponentPublicInstance> = Omit<I, '$props' | '$emit' | '$slots'> & Pick<I, Exclude<$LiteralKeys<I>, '$props' | '$emit' | '$slots'>>;";
const historicalPublicBase =
  "type $ComponentPublicBase = Omit<import('vue').ComponentPublicInstance, '$props' | '$emit' | '$slots'>;";
assert.equal(
  fs
    .readFileSync(
      path.join(repoRoot, "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt"),
      "utf8",
    )
    .split(currentPublicBase).length - 1,
  1,
);
// #11453 then guarded the camelized emit handler key against a declared twin.
const currentEmitProps =
  "type __EmitProps<T> = { [K in keyof __EmitOptions<T> & string as ($Camelize<K> extends keyof __EmitOptions<T> ? string extends keyof __EmitOptions<T> ? $HandlerKey<K> : never : $HandlerKey<K>) | `on${Capitalize<K>}`]?: __EmitOptions<T>[K] };";
const historicalEmitProps =
  "type __EmitProps<T> = { [K in keyof __EmitOptions<T> & string as $HandlerKey<K> | `on${Capitalize<K>}`]?: __EmitOptions<T>[K] };";
// #11454/#11591: `$ExtraInputKeys` admits vue-tsc's props fallback unless vize checks the program.
const currentExtraKeys =
  "K extends keyof Authored = Exclude<Exclude<keyof Authored, keyof Allowed>, typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never } ? keyof P : never>>";
const historicalExtraKeys = "K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>>";
// #11629: constructor arguments and TSX bind separate input channels.
const inputChannels = [
  [
    '\\nexport type { __EmitShape, __EmitArgs, __EmitFn, __RuntimePropValue, __RuntimePropCtorInner, __RuntimePropCtor, __RuntimePropHasBoolean, __RuntimePropResolved, __RuntimePropShape, $IsAny, __LooseRequired, $BooleanKey, __DefineProps, $IfAny, $NotUndefined, $MappedOmit, $DefaultNativeType, $InferDefault, __WithDefaultsArgs, __WithDefaultsResult, __Ref, __ShallowRef, $KebabCase, $KebabProps, $RequiredKeys, $DomListenerProps, $FallthroughAttrs, $PublicComponentInputProps, $ComponentMetadataKeys, $ComponentOwnKeys, $UnownedComponentAttrs, $Prettify, $ComponentFallthroughSurface, $ComponentFallthroughProps, $ComponentInputProps, $MissingRequiredInputKeys, $IsPropKeyUnion, $IsSinglePropKey, $RelaxedDynamicUnionExtra, $ExtraInputKeys, $ComponentMissingInputGuard, $ComponentInputGuardFor, $ComponentInputGuard, $ComponentInputFor, $ComponentInput, $AuthoredKeyWitness, $ComponentInstanceFor, $IsUnknown, $UsePublicInstance, $LiteralKeys, $ReplaceComponentProps, $PublicEmit, $PublicSlots, $ComponentPublicBase, $OverloadProps, $OverloadUnionRecursive, $OverloadUnion, $OverloadParameters, $IsStringLiteral, $ParametersToFns, __EmitOptions, $Camelize, $HandlerKey, __EmitProps, $ComponentGlobalHtmlAttrs };\\n"\n',
    '\\nexport type { __EmitShape, __EmitArgs, __EmitFn, __RuntimePropValue, __RuntimePropCtorInner, __RuntimePropCtor, __RuntimePropHasBoolean, __RuntimePropResolved, __RuntimePropShape, $IsAny, __LooseRequired, $BooleanKey, __DefineProps, $IfAny, $NotUndefined, $MappedOmit, $DefaultNativeType, $InferDefault, __WithDefaultsArgs, __WithDefaultsResult, __Ref, __ShallowRef, $KebabCase, $KebabProps, $RequiredKeys, $DomListenerProps, $FallthroughAttrs, $PublicComponentInputProps, $ComponentMetadataKeys, $ComponentOwnKeys, $UnownedComponentAttrs, $Prettify, $ComponentFallthroughSurface, $ComponentFallthroughProps, $ComponentInputProps, $MissingRequiredInputKeys, $IsPropKeyUnion, $IsSinglePropKey, $RelaxedDynamicUnionExtra, $ExtraInputKeys, $ComponentMissingInputGuard, $ComponentInputGuardFor, $ComponentInputGuard, $ComponentInputFor, $ComponentInput, $AuthoredKeyWitness, $IsUnknown, $UsePublicInstance, $LiteralKeys, $ReplaceComponentProps, $PublicEmit, $PublicSlots, $ComponentPublicBase, $OverloadProps, $OverloadUnionRecursive, $OverloadUnion, $OverloadParameters, $IsStringLiteral, $ParametersToFns, __EmitOptions, $Camelize, $HandlerKey, __EmitProps, $ComponentGlobalHtmlAttrs };\\n"\n',
  ],
  [
    "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\\n// Constructor arguments (templates, `new`) bind `Authored`; TSX binds `Jsx` through `$props` (#11629).\\ntype $ComponentInstanceFor<I, T, Declared, Authored, Jsx> = $IsAny<Authored> extends true ? I : $IsUnknown<Authored> extends true ? $UsePublicInstance<Jsx> extends true ? I : $ReplaceComponentProps<I, $AuthoredKeyWitness<Jsx> & $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx>> : $ReplaceComponentProps<I, $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Authored>>;\\ntype $IsUnknown<T> = unknown extends T ? [keyof T] extends [never] ? true : false : false;\\n",
    "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\\ntype $IsUnknown<T> = unknown extends T ? [keyof T] extends [never] ? true : false : false;\\n",
  ],
  [
    "type $RelaxedDynamicUnionExtra<T, Declared, Authored, K extends keyof Authored> = [keyof Declared] extends [never] ? $IsSinglePropKey<$RequiredKeys<T>> extends true ? K extends string ? K extends `on${string}` ? false : Authored[K] extends true ? false : true : false : false : false;\\ntype $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;\\ntype $ComponentMissingInputGuard<T, Authored> = T extends unknown ? [$MissingRequiredInputKeys<T, Authored>] extends [never] ? {} : { readonly __vizeInvalidComponentProps: { readonly missing: $MissingRequiredInputKeys<T, Authored>; readonly extra: never; } } : never;\\n",
    "type $RelaxedDynamicUnionExtra<T, Declared, Authored, K extends keyof Authored> = [keyof Declared] extends [never] ? $IsSinglePropKey<$RequiredKeys<T>> extends true ? K extends string ? K extends `on${string}` ? false : Authored[K] extends true ? false : true : false : false : false;\\ntype $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<Exclude<keyof Authored, keyof Allowed>, typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never } ? keyof P : never>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;\\ntype $ComponentMissingInputGuard<T, Authored> = T extends unknown ? [$MissingRequiredInputKeys<T, Authored>] extends [never] ? {} : { readonly __vizeInvalidComponentProps: { readonly missing: $MissingRequiredInputKeys<T, Authored>; readonly extra: never; } } : never;\\n",
  ],
  [
    "import type { $ComponentInput, $ComponentInstanceFor, $PublicEmit, $PublicSlots, $ComponentPublicBase } from",
    "import type { $ComponentInputProps, $ComponentInputGuard, $ComponentInput, $AuthoredKeyWitness, $UsePublicInstance, $ReplaceComponentProps, $PublicEmit, $PublicSlots, $ComponentPublicBase } from",
  ],
  [
    "new <$AuthoredProps = unknown, $JsxProps = unknown>(props?:",
    "new <$AuthoredProps = unknown>(props?:",
  ],
] as const;
const constructorTails: Record<string, readonly [string, string]> = {
  integration__check_cli__check_can_emit_declarations: [
    "=> $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;",
    "=> $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>>;",
  ],
  integration__build_declaration_emit_cli__build_dts_emits_declarations_alongside_compiled_js: [
    '=> $ComponentInstanceFor<$ComponentInstance, Props, import(\\"./__vize_helpers.js\\").__EmitProps<Emits>, $AuthoredProps, $JsxProps>;',
    '=> $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, import(\\"./__vize_helpers.js\\").__EmitProps<Emits>> & $ComponentInputGuard<Props, import(\\"./__vize_helpers.js\\").__EmitProps<Emits>, $AuthoredProps>>;',
  ],
};
const successors = [
  [currentExtraKeys, historicalExtraKeys],
  [currentEmitProps, historicalEmitProps],
  [currentPublicBase, historicalPublicBase],
  [currentRecursive, historicalRecursive],
] as const;

function historicalSnapshot(
  current: Buffer,
  digest: string,
  tail: readonly [string, string],
): Buffer {
  let text = current.toString("utf8");
  for (const [now, then] of [...inputChannels, tail, ...successors]) {
    assert.equal(text.split(now).length - 1, 1);
    assert.equal(text.split(then).length - 1, 0);
    text = text.replace(now, then);
  }
  const historical = Buffer.from(text);
  assert.equal(createHash("sha256").update(historical).digest("hex"), digest);
  let restored = historical.toString("utf8");
  for (const [now, then] of [...inputChannels, tail, ...successors].reverse())
    restored = restored.replace(then, now);
  assert.deepEqual(Buffer.from(restored), current);
  return historical;
}

for (const [name, digest, successorDigest] of originals) {
  test(`${name} preserves its historical packet through exact overload and slot export successors`, () => {
    const snapshotRoot = path.join(repoRoot, "crates/vize/tests/snapshots");
    const current = fs.readFileSync(path.join(snapshotRoot, `${name}.snap`));
    const tail = constructorTails[name]!;
    const original = historicalSnapshot(current, digest, tail);
    const successor = historicalSnapshot(
      fs.readFileSync(path.join(snapshotRoot, `${name}_slot_reexport.snap`)),
      successorDigest,
      tail,
    );
    const before = "export type Slots = $Slots;";
    const after = "export type { $Slots as Slots };";
    const text = original.toString("utf8");
    assert.equal(text.split(before).length - 1, 1);
    assert.equal(text.split(after).length - 1, 0);
    assert.deepEqual(successor, Buffer.from(text.replace(before, after)));
    const actual = current.toString("utf8");
    for (const mutated of [
      `${actual}\n`,
      actual.replace(currentRecursive, ""),
      `${actual}${currentRecursive}`,
      actual.replace(currentRecursive, historicalRecursive),
      actual.replace("Generated by vize", "Generated by foreign"),
      actual.replace(inputChannels[3][0], inputChannels[3][1]),
    ]) {
      assert.notEqual(mutated, actual);
      assert.throws(() => historicalSnapshot(Buffer.from(mutated), digest, tail));
    }
  });
}
