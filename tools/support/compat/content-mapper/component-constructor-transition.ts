/** Certify the two finite JSX-witness constructor transitions. */
import assert from "node:assert/strict";
import { certifiedTransition } from "./canonical-helper-transition.ts";

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
] as const;

export function certifyComponentConstructorTransition(before: Buffer, after: Buffer) {
  const transitions = componentConstructorTransitions
    .map(({ previous, current }, index) =>
      certifiedTransition(
        before,
        after,
        previous,
        current,
        `component constructor variant ${index + 1}`,
      ),
    )
    .filter((transition) => transition !== undefined);
  assert.ok(transitions.length <= 1, "only one component constructor may transition per packet");
  return transitions[0];
}
