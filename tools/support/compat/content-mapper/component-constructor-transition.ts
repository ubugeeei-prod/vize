/** Certify the finite JSX-witness constructor and generic context transitions. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { certifiedTransition } from "./canonical-helper-transition.ts";

const constructorSources = [
  [
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors.rs",
    11079,
    "c8a043b7eca7b7401ccd2f2bd3eb0b63149740f9982da9e68247fcb65ff3119e",
  ],
  [
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors/emission.rs",
    6636,
    "8891520e26d99b718b6d87546d2f8825cd8a01048196724b745f8465bb0c53ef",
  ],
  [
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors/generic_function.rs",
    2839,
    "7c7451a47f01d61f75bac6af903ca1236c7bdf41b59ab278361898b02351db9e",
  ],
] as const;
for (const [file, bytes, sha256] of constructorSources) {
  const source = readFileSync(new URL(file, import.meta.url));
  assert.equal(source.length, bytes, `certified component constructor producer size: ${file}`);
  assert.equal(
    createHash("sha256").update(source).digest("hex"),
    sha256,
    `certified component constructor producer source: ${file}`,
  );
}

export const previousGenericComponentContext = `type $GenericComponent = <T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'> | { attrs: any; emit: (...args: any[]) => any; slots: any }, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> };\n`;
const genericComponentIntersectionPrefix = "(";
const genericComponentPreciseOverload = `) & (<T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'>, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> })`;
export const currentGenericComponentContext = `type $GenericComponent = (<T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'> | { attrs: any; emit: (...args: any[]) => any; slots: any }, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> }) & (<T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'>, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> });\n`;

export function certifyGenericComponentContextTransition(before: Buffer, after: Buffer) {
  const transition = certifiedTransition(
    before,
    after,
    previousGenericComponentContext,
    currentGenericComponentContext,
    "generic component public context",
  );
  if (!transition) return;
  const prefixBoundary = previousGenericComponentContext.indexOf("= ") + 2;
  const suffixBoundary = previousGenericComponentContext.lastIndexOf(";\n");
  assert.ok(prefixBoundary > 1 && suffixBoundary > prefixBoundary);
  assert.equal(
    previousGenericComponentContext.slice(0, prefixBoundary) +
      genericComponentIntersectionPrefix +
      previousGenericComponentContext.slice(prefixBoundary, suffixBoundary) +
      genericComponentPreciseOverload +
      previousGenericComponentContext.slice(suffixBoundary),
    currentGenericComponentContext,
    "generic component overload must preserve the compatibility signature byte-for-byte",
  );
  const edits = [
    {
      start:
        transition.start +
        Buffer.byteLength(previousGenericComponentContext.slice(0, prefixBoundary)),
      end:
        transition.start +
        Buffer.byteLength(previousGenericComponentContext.slice(0, prefixBoundary)),
      replacement: Buffer.from(genericComponentIntersectionPrefix),
      label: `${transition.label} prefix`,
    },
    {
      start:
        transition.start +
        Buffer.byteLength(previousGenericComponentContext.slice(0, suffixBoundary)),
      end:
        transition.start +
        Buffer.byteLength(previousGenericComponentContext.slice(0, suffixBoundary)),
      replacement: Buffer.from(genericComponentPreciseOverload),
      label: `${transition.label} precise overload`,
    },
  ];
  return {
    edits,
    previousStart: transition.start,
    previousEnd: transition.end,
    preciseOverload: edits[1]!,
  };
}

export const componentConstructorTransitions = [
  {
    previous: `type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, __EmitProps<Emits>> & $ComponentInputGuard<Props, __EmitProps<Emits>, $AuthoredProps>> & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps> & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>;
`,
  },
  {
    previous: `type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<{
  $props: Props;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>> & $ComponentPublicBase;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, {}, $AuthoredProps, $JsxProps> & $ComponentPublicBase;
`,
  },
  {
    previous: `type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, __EmitProps<Emits>> & $ComponentInputGuard<Props, __EmitProps<Emits>, $AuthoredProps>> & $ComponentPublicBase;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps> & $ComponentPublicBase;
`,
  },
  {
    previous: `type $ComponentInstance = {
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
} & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>;
type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps>;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps> & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>;
`,
  },
  {
    previous: `type $ComponentInstance = {
  $props: Props;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
} & $ComponentPublicBase;
type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, {}, $AuthoredProps, $JsxProps> & $ComponentPublicBase;
`,
  },
  {
    previous: `type $ComponentInstance = {
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
} & $ComponentPublicBase;
type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps>;
`,
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{
  $props: Props & __EmitProps<Emits>;
  readonly __vizeRawProps?: Props;
  $emit: $PublicEmit<Emits>;
  $slots: $PublicSlots<$Slots>;
}, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps> & $ComponentPublicBase;
`,
  },
] as const;

export function certifyComponentConstructorTransition(before: Buffer, after: Buffer) {
  const old = before.toString();
  const next = after.toString();
  const occurrences = (text: string, needle: string) => text.split(needle).length - 1;
  const candidates = componentConstructorTransitions.flatMap(({ previous, current }, index) => {
    const counts = [
      occurrences(old, previous),
      occurrences(old, current),
      occurrences(next, previous),
      occurrences(next, current),
    ];
    return counts[0] === 1 && counts[1] === 0 && counts[2] === 0 && counts[3] === 1
      ? [{ previous, current, index }]
      : [];
  });
  assert.ok(candidates.length <= 1, "only one component constructor may transition per packet");
  if (candidates.length === 0) {
    for (const value of new Set(
      componentConstructorTransitions.flatMap((item) => Object.values(item)),
    )) {
      const counts = [occurrences(old, value), occurrences(next, value)];
      assert.deepEqual(counts, [counts[0], counts[0]], "component constructor stays exact");
      assert.ok(counts[0] <= 1, "component constructor is unique");
    }
    return;
  }
  const candidate = candidates[0]!;
  for (const previous of new Set(componentConstructorTransitions.map((item) => item.previous))) {
    if (previous === candidate.previous) continue;
    assert.equal(occurrences(old, previous), 0, "only one previous constructor variant is present");
    assert.equal(
      occurrences(next, previous),
      0,
      "only one previous constructor variant is present",
    );
  }
  return certifiedTransition(
    before,
    after,
    candidate.previous,
    candidate.current,
    `component constructor variant ${candidate.index + 1}`,
  );
}
