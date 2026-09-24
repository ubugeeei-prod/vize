/** Compile-only assertions for the `use-image` type contracts. */

import { useImage } from "./use-image.ts";
import type { ImageHost, ImageLike, ImageLoadResult, ImageStatus } from "./use-image.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const image = useImage({ src: "/a.png", loading: "lazy" });
type _StatusIsClosed = Expect<Equal<typeof image.status.value, ImageStatus>>;
type _LoadResolvesAResult = Expect<Equal<Awaited<ReturnType<typeof image.load>>, ImageLoadResult>>;

Image satisfies ImageHost;
declare const element: HTMLImageElement;
element satisfies ImageLike;

// @ts-expect-error a source URL is required.
useImage({ alt: "missing src" });

// @ts-expect-error loading is a closed union.
useImage({ src: "/a.png", loading: "soon" });
