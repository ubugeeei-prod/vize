import type {
  __DefineProps,
  __RuntimePropCtor,
  $ComponentInput,
  $ComponentOwnKeys,
  $IsAny,
  $PublicComponentInputProps,
  $UnownedComponentAttrs,
} from "vize/content-mapper-helpers";
import type { ComponentCustomProps } from "vue";
import type { IsAny } from "vize/virtual-ts";

declare module "vue" {
  interface ComponentCustomProps {
    sharedProjectFlag?: boolean;
  }
}

type Equal<L, R> =
  (<T>() => T extends L ? 1 : 2) extends <T>() => T extends R ? 1 : 2 ? true : false;
type Expect<T extends true> = T;
type Props = { id: number; label?: string };
type Accepts<A> = A extends $ComponentInput<Props, {}, A> ? true : false;

export type PreservesAuthoredProps = Expect<Equal<Accepts<{ id: number }>, true>>;
export type RejectsWrongProp = Expect<Equal<Accepts<{ id: string }>, false>>;
export type RejectsMissingProp = Expect<Equal<Accepts<{}>, false>>;
export type RejectsUnknownProp = Expect<Equal<Accepts<{ id: number; incorrect: true }>, false>>;
export type PreservesNativeFallthrough = Expect<
  Equal<Accepts<{ id: number; onClick: (event: MouseEvent) => void }>, true>
>;
export type PreservesConsumerAugmentation = Expect<
  Equal<$PublicComponentInputProps["sharedProjectFlag"], ComponentCustomProps["sharedProjectFlag"]>
>;
export type AugmentationRemainsOwned = Expect<
  Equal<"sharedProjectFlag" extends $ComponentOwnKeys ? true : false, true>
>;
export type KeepsOnlyUnownedAttributes = Expect<
  Equal<
    $UnownedComponentAttrs<
      { id: number; title: string; onSave: () => void },
      Props,
      { onSave?: () => void }
    >,
    { title: string }
  >
>;
export type RuntimeBooleanStaysBoolean = Expect<
  Equal<__RuntimePropCtor<BooleanConstructor>, boolean>
>;
export type GenericMacroStaysExact = Expect<
  Equal<__DefineProps<{ value: string }>["value"], string>
>;
declare const declaredProps: __DefineProps<{ value: string }>;
// @ts-expect-error compiler macro props remain readonly across the shared alias.
declaredProps.value = "changed";
export type DoesNotWidenToAny = Expect<
  Equal<$IsAny<$ComponentInput<Props, {}, { id: number }>>, false>
>;
export type PublicAliasSeesSharedAmbient = Expect<Equal<IsAny<number>, false>>;
export type SharedAmbientRemainsExact = Expect<Equal<__RuntimePropCtor<StringConstructor>, string>>;
export type PatternAmbientRemainsAvailable = Expect<Equal<$Patterns.IsAny<any>, true>>;
declare const ambientString: __RuntimePropCtor<StringConstructor>;
// @ts-expect-error moving ambient declarations must preserve the string constraint.
export const rejectsNumber: number = ambientString;
