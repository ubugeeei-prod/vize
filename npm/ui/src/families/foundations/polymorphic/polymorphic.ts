import { h } from "vue";
import type {
  Component,
  ComponentPublicInstance,
  IntrinsicElementAttributes,
  VNode,
  VNodeArrayChildren,
  VNodeChild,
} from "vue";

const emptyTagDiagnostic = "VIZE_UI_POLYMORPHIC_AS";

/** Native tag names with typed attributes (HTML and SVG). */
export type PolymorphicTag = keyof IntrinsicElementAttributes;

/** Value accepted by a polymorphic `as` prop: a native tag or a component. */
export type PolymorphicAs = PolymorphicTag | Component;

/** Props of a component, inferred from its public instance or functional signature. */
export type ComponentPropsOf<Target> = Target extends abstract new (...args: never[]) => {
  readonly $props: infer Props;
}
  ? Props
  : Target extends (props: infer Props, ...rest: never[]) => unknown
    ? Props
    : Readonly<Record<string, unknown>>;

/** Attributes accepted by `As`: native attributes for tags, props for components. */
export type PolymorphicAttributes<As extends PolymorphicAs> = As extends PolymorphicTag
  ? IntrinsicElementAttributes[As]
  : ComponentPropsOf<As>;

/**
 * Props of a polymorphic component: own props, the `as` prop, and the
 * attributes of the rendered element (own props win on name clashes).
 */
export type PolymorphicProps<As extends PolymorphicAs, OwnProps extends object = {}> = OwnProps & {
  /** Element or component to render. */
  readonly as?: As;
} & Omit<PolymorphicAttributes<As>, keyof OwnProps | "as">;

/** DOM element (or component instance) rendered for `As`. */
export type PolymorphicElement<As extends PolymorphicAs> = As extends keyof HTMLElementTagNameMap
  ? HTMLElementTagNameMap[As]
  : As extends keyof SVGElementTagNameMap
    ? SVGElementTagNameMap[As]
    : As extends PolymorphicTag
      ? Element
      : ComponentPublicInstance;

/**
 * Whether `as` names a native element (a string) rather than a component.
 *
 * @param as Polymorphic target.
 * @returns Whether `as` is a tag name.
 */
export function isPolymorphicTag(as: PolymorphicAs): as is PolymorphicTag {
  return typeof as === "string";
}

/**
 * Add the implicit attributes a native element needs to keep its semantics
 * when rendered polymorphically.
 *
 * - `button` defaults to `type="button"` (never an accidental form submit).
 * - `a` with a truthy `disabled` drops `href`, gains `aria-disabled="true"`
 *   and `tabindex="-1"`, since anchors have no native disabled state.
 * - Any other non-button tag with a truthy `disabled` gains
 *   `aria-disabled="true"`.
 *
 * Explicit attributes always win. An empty tag name throws
 * `VIZE_UI_POLYMORPHIC_AS`. Pure, so SSR output is deterministic.
 *
 * @param as Polymorphic target.
 * @param attributes Attributes requested by the caller.
 * @returns Attributes to render.
 */
export function resolvePolymorphicAttributes<As extends PolymorphicAs>(
  as: As,
  attributes: PolymorphicAttributes<As>,
): PolymorphicAttributes<As>;
export function resolvePolymorphicAttributes(
  as: PolymorphicAs,
  attributes: Readonly<Record<string, unknown>>,
): Readonly<Record<string, unknown>> {
  if (!isPolymorphicTag(as)) return attributes;
  if (as.length === 0) throw new Error(`${emptyTagDiagnostic}: "as" must not be an empty tag name`);
  const disabled = attributes["disabled"] === true || attributes["disabled"] === "";
  if (as === "button") {
    return attributes["type"] === undefined ? { type: "button", ...attributes } : attributes;
  }
  if (!disabled) return attributes;
  if (as === "a") {
    const { href: _href, disabled: _disabled, ...rest } = attributes;
    return { "aria-disabled": "true", tabindex: -1, ...rest };
  }
  const { disabled: _disabled, ...rest } = attributes;
  return { "aria-disabled": "true", ...rest };
}

/**
 * Render `as` with attributes type-checked against the chosen element or
 * component (render-function counterpart of a polymorphic `as` prop).
 *
 * ```ts
 * renderPolymorphic("a", { href: "/docs" }, () => "Docs");
 * renderPolymorphic("a", { hreff: "/docs" }); // compile error
 * ```
 *
 * @param as Tag or component to render.
 * @param attributes Attributes inferred from `as`.
 * @param children Children or default slot.
 * @default children undefined
 * @returns The rendered VNode.
 */
export function renderPolymorphic<As extends PolymorphicAs>(
  as: As,
  attributes: PolymorphicAttributes<As>,
  children?: string | VNodeArrayChildren | (() => VNodeChild),
): VNode {
  const resolved = resolvePolymorphicAttributes(as, attributes);
  if (typeof children === "function") {
    return isPolymorphicTag(as)
      ? h(as, resolved, [children()])
      : h(as, resolved, { default: children });
  }
  return h(as, resolved, children);
}
