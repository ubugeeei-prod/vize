/** Certify the finite JSX-witness constructor transitions. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { certifiedTransition } from "./canonical-helper-transition.ts";

const constructorSource = readFileSync(
  new URL(
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_constructors.rs",
    import.meta.url,
  ),
);
assert.equal(constructorSource.length, 10645, "certified component constructor producer size");
assert.equal(
  createHash("sha256").update(constructorSource).digest("hex"),
  "94e915b4968a2d7d4b0d1b4292b74aa24cbc7434e4b7d8f191213b59cfdbb853",
  "certified component constructor producer source",
);

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
