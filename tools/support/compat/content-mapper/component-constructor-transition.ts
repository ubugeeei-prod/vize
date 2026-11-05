/** Certify the finite JSX-witness constructor and generic context transitions. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { certifiedTransition } from "./canonical-helper-transition.ts";

const constructorSources = [
  [
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors.rs",
    10645,
    "94e915b4968a2d7d4b0d1b4292b74aa24cbc7434e4b7d8f191213b59cfdbb853",
  ],
  [
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors/emission.rs",
    6111,
    "03495714bf27a42979433d45320839e155869f48ca240755df157ea4d1f1094c",
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

export const previousGenericComponentContext = `type $GenericComponent = <T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'>, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> };\n`;
export const currentGenericComponentContext = `type $GenericComponent = <T extends { id: number; label: string } = { id: number; label: string }>(props: $GenericContext<T>['props'], ctx?: Pick<$GenericContext<T>, 'attrs' | 'emit' | 'slots'> | { attrs: any; emit: (...args: any[]) => any; slots: any }, expose?: $GenericContext<T>['expose']) => import('vue').VNode & { __ctx?: $GenericContext<T> };\n`;
const genericComponentContextInsertion =
  " | { attrs: any; emit: (...args: any[]) => any; slots: any }";

export function certifyGenericComponentContextTransition(before: Buffer, after: Buffer) {
  const transition = certifiedTransition(
    before,
    after,
    previousGenericComponentContext,
    currentGenericComponentContext,
    "generic component public context",
  );
  if (!transition) return;
  const insertion = previousGenericComponentContext.indexOf(", expose?:");
  assert.ok(insertion > 0, "generic component context insertion boundary");
  assert.equal(
    previousGenericComponentContext.slice(0, insertion) +
      genericComponentContextInsertion +
      previousGenericComponentContext.slice(insertion),
    currentGenericComponentContext,
    "generic component context must be one exact insertion",
  );
  const start =
    transition.start + Buffer.byteLength(previousGenericComponentContext.slice(0, insertion));
  return {
    start,
    end: start,
    replacement: Buffer.from(genericComponentContextInsertion),
    label: transition.label,
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
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps>;
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
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;
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
    current: `type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, __EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, __EmitProps<Emits>, $AuthoredProps, $JsxProps>;
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
