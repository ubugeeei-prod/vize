import { computed, readonly, ref, shallowRef, unref } from "vue";
import type { ComputedRef, MaybeRef, Ref, ShallowRef } from "vue";

/** Formats supported by `CompressionStream` and `DecompressionStream`. */
export type CompressionFormatName = "gzip" | "deflate" | "deflate-raw";

/** Data accepted by {@link compress} and {@link decompress}. */
export type CompressionInput =
  | string
  | ArrayBuffer
  | ArrayBufferView
  | Blob
  | ReadableStream<BufferSource>;

/** Transform stream created by a (de)compression constructor. */
export interface CompressionTransformLike {
  /** Transformed output. */
  readonly readable: ReadableStream<Uint8Array>;

  /** Raw input. */
  readonly writable: WritableStream<BufferSource>;
}

/** Minimal `CompressionStream` / `DecompressionStream` constructor. */
export type CompressionStreamConstructor = new (
  format: CompressionFormatName,
) => CompressionTransformLike;

/** Options for {@link compress}. */
export interface CompressOptions {
  /**
   * Compression stream constructor.
   *
   * @default window.CompressionStream when it exists
   */
  readonly CompressionStream?: CompressionStreamConstructor | null | undefined;
}

/** Options for {@link decompress} and {@link decompressText}. */
export interface DecompressOptions {
  /**
   * Decompression stream constructor.
   *
   * @default window.DecompressionStream when it exists
   */
  readonly DecompressionStream?: CompressionStreamConstructor | null | undefined;
}

/** Options for {@link useCompressionStream}. */
export interface UseCompressionStreamOptions {
  /**
   * Compression stream constructor. A ref (not a getter) because the host is
   * a constructor function.
   *
   * @default window.CompressionStream when it exists
   */
  readonly CompressionStream?: MaybeRef<CompressionStreamConstructor | null | undefined>;

  /**
   * Decompression stream constructor.
   *
   * @default window.DecompressionStream when it exists
   */
  readonly DecompressionStream?: MaybeRef<CompressionStreamConstructor | null | undefined>;

  /**
   * Format used when an action is called without one.
   *
   * @default "gzip"
   */
  readonly format?: CompressionFormatName;
}

/** Reactive state and actions returned by {@link useCompressionStream}. */
export interface CompressionStreamControls {
  /** Whether both `CompressionStream` and `DecompressionStream` are available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether an action is in flight. */
  readonly pending: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared by the next successful action. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Compress `data`; rejects like {@link compress}.
   *
   * @param data Input data.
   * @param format Compression format.
   * @returns Compressed bytes.
   */
  readonly compress: (
    data: CompressionInput,
    format?: CompressionFormatName,
  ) => Promise<Uint8Array<ArrayBuffer>>;

  /**
   * Decompress `data`; rejects like {@link decompress}.
   *
   * @param data Compressed data.
   * @param format Compression format.
   * @returns Decompressed bytes.
   */
  readonly decompress: (
    data: CompressionInput,
    format?: CompressionFormatName,
  ) => Promise<Uint8Array<ArrayBuffer>>;

  /**
   * Decompress `data` and decode it as UTF-8.
   *
   * @param data Compressed data.
   * @param format Compression format.
   * @returns Decompressed text.
   */
  readonly decompressText: (
    data: CompressionInput,
    format?: CompressionFormatName,
  ) => Promise<string>;
}

const formats: readonly string[] = ["gzip", "deflate", "deflate-raw"];

function isConstructor(candidate: unknown): candidate is CompressionStreamConstructor {
  return typeof candidate === "function";
}

function browserConstructor(
  name: "CompressionStream" | "DecompressionStream",
): CompressionStreamConstructor | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, name);
  return isConstructor(candidate) ? candidate : undefined;
}

function toReadable(data: CompressionInput): ReadableStream<BufferSource> {
  if (data instanceof ReadableStream) return data;
  if (data instanceof Blob) return data.stream();
  const bytes =
    typeof data === "string"
      ? new TextEncoder().encode(data)
      : ArrayBuffer.isView(data)
        ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength).slice()
        : new Uint8Array(data);
  return new ReadableStream<BufferSource>({
    start(controller) {
      controller.enqueue(bytes);
      controller.close();
    },
  });
}

async function transform(
  data: CompressionInput,
  format: CompressionFormatName,
  Stream: CompressionStreamConstructor | null | undefined,
  name: "CompressionStream" | "DecompressionStream",
): Promise<Uint8Array<ArrayBuffer>> {
  if (!formats.includes(format)) {
    throw new TypeError(
      `[VIZE_COMPOSE_COMPRESSION_INVALID_FORMAT] Unsupported format "${String(format)}".`,
    );
  }
  if (!Stream) {
    throw new TypeError(`[VIZE_COMPOSE_COMPRESSION_UNSUPPORTED] ${name} is not available.`);
  }
  const reader = toReadable(data).pipeThrough(new Stream(format)).getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    length += value.byteLength;
  }
  const output = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    output.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return output;
}

/**
 * Compress data with the Compression Streams API.
 *
 * Strings are UTF-8 encoded first. Rejects with a tagged `TypeError` for an
 * unknown format or when no `CompressionStream` is available (always on the
 * server unless one is injected).
 *
 * @example
 * ```ts
 * const gzipped = await compress(JSON.stringify(state), "gzip");
 * ```
 *
 * @param data Input data.
 * @param format Compression format.
 * @param options Constructor override.
 * @default format "gzip"
 * @default options {}
 * @returns Compressed bytes.
 */
export function compress(
  data: CompressionInput,
  format: CompressionFormatName = "gzip",
  options: CompressOptions = {},
): Promise<Uint8Array<ArrayBuffer>> {
  const Stream =
    options.CompressionStream === undefined
      ? browserConstructor("CompressionStream")
      : options.CompressionStream;
  return transform(data, format, Stream, "CompressionStream");
}

/**
 * Decompress data with the Compression Streams API.
 *
 * Rejects with a tagged `TypeError` for an unknown format or when no
 * `DecompressionStream` is available, and with the stream's error for
 * corrupt input.
 *
 * @example
 * ```ts
 * const bytes = await decompress(await response.blob(), "gzip");
 * ```
 *
 * @param data Compressed data.
 * @param format Compression format.
 * @param options Constructor override.
 * @default format "gzip"
 * @default options {}
 * @returns Decompressed bytes.
 */
export function decompress(
  data: CompressionInput,
  format: CompressionFormatName = "gzip",
  options: DecompressOptions = {},
): Promise<Uint8Array<ArrayBuffer>> {
  const Stream =
    options.DecompressionStream === undefined
      ? browserConstructor("DecompressionStream")
      : options.DecompressionStream;
  return transform(data, format, Stream, "DecompressionStream");
}

/**
 * Decompress data and decode the result as UTF-8 text.
 *
 * Rejects exactly like {@link decompress}.
 *
 * @example
 * ```ts
 * const json = JSON.parse(await decompressText(bytes, "deflate"));
 * ```
 *
 * @param data Compressed data.
 * @param format Compression format.
 * @param options Constructor override.
 * @default format "gzip"
 * @default options {}
 * @returns Decompressed text.
 */
export async function decompressText(
  data: CompressionInput,
  format: CompressionFormatName = "gzip",
  options: DecompressOptions = {},
): Promise<string> {
  return new TextDecoder().decode(await decompress(data, format, options));
}

/**
 * Reactive wrapper around {@link compress}, {@link decompress} and
 * {@link decompressText}.
 *
 * Tracks whether an action is in flight and records the last failure in
 * `error`; actions still reject so callers can handle failures inline.
 * Nothing needs cleanup.
 *
 * Server rendering: `supported` is false and actions reject with the tagged
 * unsupported error unless constructors are injected.
 *
 * @example
 * ```ts
 * const { compress, decompressText, pending } = useCompressionStream({ format: "deflate" });
 * const packed = await compress(draft.value);
 * ```
 *
 * @param options Constructor overrides and default format.
 * @default options {}
 * @returns Compression state and bound actions.
 */
export function useCompressionStream(
  options: UseCompressionStreamOptions = {},
): CompressionStreamControls {
  const defaultFormat = options.format ?? "gzip";
  const inFlight = ref(0);
  const error = shallowRef<unknown>(undefined);

  const resolve = (
    source: MaybeRef<CompressionStreamConstructor | null | undefined> | undefined,
    name: "CompressionStream" | "DecompressionStream",
  ): CompressionStreamConstructor | undefined =>
    source === undefined ? browserConstructor(name) : (unref(source) ?? undefined);
  const resolveCompression = (): CompressionStreamConstructor | undefined =>
    resolve(options.CompressionStream, "CompressionStream");
  const resolveDecompression = (): CompressionStreamConstructor | undefined =>
    resolve(options.DecompressionStream, "DecompressionStream");

  const track = async <Value>(action: () => Promise<Value>): Promise<Value> => {
    inFlight.value += 1;
    try {
      const value = await action();
      error.value = undefined;
      return value;
    } catch (cause) {
      error.value = cause;
      throw cause;
    } finally {
      inFlight.value -= 1;
    }
  };

  return {
    supported: computed(
      () => resolveCompression() !== undefined && resolveDecompression() !== undefined,
    ),
    pending: computed(() => inFlight.value > 0),
    error: readonly(error),
    compress: (data, format = defaultFormat) =>
      track(() => compress(data, format, { CompressionStream: resolveCompression() ?? null })),
    decompress: (data, format = defaultFormat) =>
      track(() =>
        decompress(data, format, { DecompressionStream: resolveDecompression() ?? null }),
      ),
    decompressText: (data, format = defaultFormat) =>
      track(() =>
        decompressText(data, format, { DecompressionStream: resolveDecompression() ?? null }),
      ),
  };
}
