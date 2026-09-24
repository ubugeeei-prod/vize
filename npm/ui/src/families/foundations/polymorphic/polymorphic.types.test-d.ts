/** Compile-only assertions for typed polymorphic helpers. */

import { defineComponent } from "vue";
import type { AnchorHTMLAttributes, ButtonHTMLAttributes, VNode } from "vue";

import {
  isPolymorphicTag,
  renderPolymorphic,
  resolvePolymorphicAttributes,
} from "./polymorphic.ts";
import type {
  ComponentPropsOf,
  PolymorphicAs,
  PolymorphicAttributes,
  PolymorphicElement,
  PolymorphicProps,
} from "./polymorphic.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _AnchorAttributesAreNative = Expect<Equal<PolymorphicAttributes<"a">, AnchorHTMLAttributes>>;
type _ButtonAttributesAreNative = Expect<
  Equal<PolymorphicAttributes<"button">, ButtonHTMLAttributes>
>;
type _ElementIsInferred = Expect<Equal<PolymorphicElement<"a">, HTMLAnchorElement>>;
type _SvgElementIsInferred = Expect<Equal<PolymorphicElement<"circle">, SVGCircleElement>>;

type LinkProps = PolymorphicProps<"a", { readonly variant?: "plain" | "underline" }>;
type _OwnPropsSurvive = Expect<Equal<LinkProps["variant"], "plain" | "underline" | undefined>>;
type _NativeAttributesAreInferred = Expect<Equal<LinkProps["href"], string | undefined>>;
type _AsIsNarrowed = Expect<Equal<LinkProps["as"], "a" | undefined>>;

const Badge = defineComponent({
  props: { tone: { type: String as () => "info" | "warn", required: true } },
  setup: () => () => null,
});
type BadgeProps = ComponentPropsOf<typeof Badge>;
type _ComponentPropsAreInferred = Expect<Equal<BadgeProps["tone"], "info" | "warn">>;

renderPolymorphic("a", { href: "/docs", target: "_blank" }, () => "Docs") satisfies VNode;
renderPolymorphic("button", { type: "submit", disabled: true });
renderPolymorphic(Badge, { tone: "info" });
// @ts-expect-error attributes are checked against the chosen element.
renderPolymorphic("a", { hreff: "/docs" });
// @ts-expect-error component props are checked.
renderPolymorphic(Badge, { tone: "error" });
// @ts-expect-error unknown tags are rejected.
renderPolymorphic("notatag", {});

resolvePolymorphicAttributes("button", { disabled: true }) satisfies ButtonHTMLAttributes;

declare const target: PolymorphicAs;
if (isPolymorphicTag(target)) {
  target satisfies string;
}
