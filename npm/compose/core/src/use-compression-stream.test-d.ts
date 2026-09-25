/** Compile-only assertions for the `use-compression-stream` type contracts. */

import { compress, decompressText, useCompressionStream } from "./use-compression-stream.ts";
import type { CompressionStreamConstructor } from "./use-compression-stream.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _CompressResolvesBytes = Expect<
  Equal<Awaited<ReturnType<typeof compress>>, Uint8Array<ArrayBuffer>>
>;
type _TextResolvesString = Expect<Equal<Awaited<ReturnType<typeof decompressText>>, string>>;

CompressionStream satisfies CompressionStreamConstructor;
DecompressionStream satisfies CompressionStreamConstructor;

const controls = useCompressionStream({ CompressionStream, DecompressionStream });
void controls.compress(new Blob(["x"]), "deflate-raw");

// @ts-expect-error brotli is not a Compression Streams format.
void compress("x", "br");

// @ts-expect-error numbers are not compressible input.
void compress(1);

// @ts-expect-error pending is read-only.
controls.pending.value = true;
