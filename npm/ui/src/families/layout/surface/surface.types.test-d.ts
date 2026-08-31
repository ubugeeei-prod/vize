/** Compile-only assertions for the public Surface contract. */

import { Surface } from "./surface.ts";
import type { ComponentPublicInstance } from "vue";
import { defineComponent } from "vue";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import type {
  SurfaceAriaState,
  SurfaceAs,
  SurfaceElement,
  SurfaceElevation,
  SurfaceExpose,
  SurfaceProps,
  SurfaceSemanticHost,
  SurfaceSlotState,
  SurfaceTone,
} from "./surface.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: SurfaceExpose;

type _SemanticHostIsDocumented = Expect<
  Equal<SurfaceSemanticHost, "article" | "aside" | "div" | "section">
>;
type _AsIsPolymorphic = Expect<Equal<SurfaceAs, PrimitiveAs>>;
type _ToneIsClosed = Expect<
  Equal<SurfaceTone, "accent" | "danger" | "info" | "muted" | "neutral" | "success" | "warning">
>;
type _ElevationIsThemeRole = Expect<Equal<SurfaceElevation, "floating" | "overlay" | "raised">>;
type _ElementIsPrimitive = Expect<Equal<SurfaceElement, PrimitiveElement>>;
type _ExposeAsIsLiteral = Expect<Equal<typeof exposed.as, SurfaceAs>>;
type _ExposeToneIsOptional = Expect<Equal<typeof exposed.tone, SurfaceTone | undefined>>;
type _ExposeElevationIsOptional = Expect<
  Equal<typeof exposed.elevation, SurfaceElevation | undefined>
>;
type _ExposeLabelledbyIsOptional = Expect<Equal<typeof exposed.ariaLabelledby, string | undefined>>;
type _AriaStateIsStrict = Expect<
  Equal<
    SurfaceAriaState,
    {
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
    }
  >
>;
type _PropsKeysAreClosed = Expect<
  Equal<keyof SurfaceProps, "ariaDescribedby" | "ariaLabelledby" | "as" | "elevation" | "tone">
>;
type _SlotStateIsStrict = Expect<
  Equal<
    SurfaceSlotState,
    {
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
      readonly as: SurfaceAs;
      readonly tone: SurfaceTone | undefined;
      readonly elevation: SurfaceElevation | undefined;
      readonly labelled: boolean;
      readonly described: boolean;
    }
  >
>;

const exposedElement: SurfaceElement | null = exposed.element;
const publicProps = {
  ariaDescribedby: "surface-help",
  ariaLabelledby: "surface-title",
  as: "article",
  elevation: "floating",
  tone: "muted",
} satisfies SurfaceProps;
const CustomSurfaceHost = defineComponent({
  name: "CustomSurfaceHost",
  setup() {
    return () => null;
  },
});
const customHostProps = {
  as: CustomSurfaceHost,
  elevation: "overlay",
  tone: "info",
} satisfies SurfaceProps;
const componentProps: InstanceType<typeof Surface>["$props"] = publicProps;
const slotState: SurfaceSlotState = {
  ariaDescribedby: undefined,
  ariaLabelledby: "surface-title",
  as: "section",
  described: false,
  elevation: undefined,
  labelled: true,
  tone: "neutral",
};
const primitiveElement: SurfaceElement = {} as ComponentPublicInstance;

const semanticHost: SurfaceSemanticHost = "article";

// @ts-expect-error Surface tones are strict consumer styling tokens.
const invalidTone: SurfaceTone = "brand";

// @ts-expect-error Surface elevation hooks must match theme elevation roles.
const invalidElevation: SurfaceElevation = "flat";

// @ts-expect-error optional hooks must stay strings, not booleans.
const badAriaProp = { ariaLabelledby: true } satisfies SurfaceProps;

void Surface;
void badAriaProp;
void componentProps;
void customHostProps;
void exposedElement;
void invalidElevation;
void invalidTone;
void primitiveElement;
void publicProps;
void semanticHost;
void slotState;
