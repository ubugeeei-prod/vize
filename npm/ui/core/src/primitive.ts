import type { Component, ComponentPublicInstance } from "vue";

/** Native element, custom element, or component accepted by {@link Primitive}. */
export type PrimitiveAs = string | Component;

/** Rendered value exposed by {@link Primitive}. */
export type PrimitiveElement = Element | ComponentPublicInstance;

/** Unstyled polymorphic foundation with attribute and slot forwarding. */
export { default as Primitive } from "./primitive-element.vue";
