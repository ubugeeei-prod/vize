/** Compile-only assertions for the public native render-node kind contract. */

import type { RenderNodeNapi } from "@vizejs/fresco-native";

import type { FrescoRenderNodeKind } from "./index.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _FrescoKindIsTheClosedProtocol = Expect<
  Equal<FrescoRenderNodeKind, "root" | "box" | "text" | "input">
>;
type _NativeKindMatchesThePublicProtocol = Expect<
  Equal<RenderNodeNapi["nodeType"], FrescoRenderNodeKind>
>;

export const rootNode: RenderNodeNapi = { id: -1, nodeType: "root" };
export const boxNode: RenderNodeNapi = { id: 1, nodeType: "box" };
export const textNode: RenderNodeNapi = { id: 2, nodeType: "text", text: "hello" };
export const inputNode: RenderNodeNapi = { id: 3, nodeType: "input", value: "value" };

// @ts-expect-error - unknown node kinds must not cross the native render boundary.
export const unknownNode: RenderNodeNapi = { id: 4, nodeType: "grid" };

declare const arbitraryKind: string;

// @ts-expect-error - the native contract must not widen back to arbitrary strings.
export const arbitraryNode: RenderNodeNapi = { id: 5, nodeType: arbitraryKind };
