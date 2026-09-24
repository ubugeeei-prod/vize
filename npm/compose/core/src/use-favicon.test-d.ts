/** Compile-only assertions for the `use-favicon` type contracts. */

import type { Ref } from "vue";

import { useFavicon } from "./use-favicon.ts";
import type { FaviconLink } from "./use-favicon.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const { icon } = useFavicon("/icon.svg");
type _IconIsNullableString = Expect<Equal<typeof icon, Ref<string | null | undefined>>>;

declare const link: HTMLLinkElement;
link satisfies FaviconLink;

// @ts-expect-error icons are URLs.
useFavicon(1);

// @ts-expect-error hosts must find and create icons.
useFavicon("/x", { host: { findIcons: () => [] } });
