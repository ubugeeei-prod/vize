/** Compile-only assertions for the `use-object-url` type contracts. */

import type { Ref } from "vue";

import { useObjectUrl } from "./use-object-url.ts";
import type { ObjectUrlHost } from "./use-object-url.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const url = useObjectUrl(new Blob());
type _UrlIsOptionalString = Expect<Equal<typeof url, Readonly<Ref<string | undefined>>>>;

URL satisfies ObjectUrlHost;

// @ts-expect-error only blobs and media sources can be referenced.
useObjectUrl("text");

// @ts-expect-error the URL is read-only.
url.value = "blob:x";
