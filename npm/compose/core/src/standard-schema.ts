/**
 * Minimal, dependency-free Standard Schema v1 interop.
 *
 * Any validation library implementing the `~standard` protocol (Zod,
 * Valibot, ArkType, Effect Schema, …) satisfies {@link StandardSchemaV1}
 * structurally; nothing from those libraries is imported.
 */

/** One segment of a Standard Schema issue path. */
export type StandardSchemaPathSegment = PropertyKey | { readonly key: PropertyKey };

/** One validation issue reported by a Standard Schema. */
export interface StandardSchemaIssue {
  /** Human-readable message. */
  readonly message: string;

  /** Location of the issue inside the validated value; absent for root issues. */
  readonly path?: readonly StandardSchemaPathSegment[] | undefined;
}

/** Successful Standard Schema result. */
export interface StandardSchemaSuccess<Output> {
  /** Parsed (possibly transformed) output. */
  readonly value: Output;

  /** Always absent on success. */
  readonly issues?: undefined;
}

/** Failed Standard Schema result. */
export interface StandardSchemaFailure {
  /** Non-empty list of issues. */
  readonly issues: readonly StandardSchemaIssue[];
}

/** Result of `~standard.validate`. */
export type StandardSchemaResult<Output> = StandardSchemaSuccess<Output> | StandardSchemaFailure;

/** The `~standard` property of a Standard Schema v1 validator. */
export interface StandardSchemaProps<Input, Output> {
  /** Protocol version. */
  readonly version: 1;

  /** Library that produced the schema. */
  readonly vendor: string;

  /** Validate an unknown value, synchronously or asynchronously. */
  readonly validate: (
    value: unknown,
  ) => StandardSchemaResult<Output> | Promise<StandardSchemaResult<Output>>;

  /** Type-only carrier of the input and output types. */
  readonly types?: { readonly input: Input; readonly output: Output } | undefined;
}

/** Any Standard Schema v1 validator. */
export interface StandardSchemaV1<Input = unknown, Output = Input> {
  /** Standard Schema protocol entry point. */
  readonly "~standard": StandardSchemaProps<Input, Output>;
}

/** Input type accepted by a Standard Schema. */
export type InferStandardSchemaInput<Schema extends StandardSchemaV1> = NonNullable<
  Schema["~standard"]["types"]
>["input"];

/** Output type produced by a Standard Schema. */
export type InferStandardSchemaOutput<Schema extends StandardSchemaV1> = NonNullable<
  Schema["~standard"]["types"]
>["output"];

/** Issue normalized to a dotted path (`""` for the root). */
export interface NormalizedSchemaIssue {
  /** Dotted path such as `address.city` or `items.0.name`; `""` for the root. */
  readonly path: string;

  /** Human-readable message. */
  readonly message: string;
}

/** Normalized outcome of {@link validateStandardSchema}. */
export type NormalizedSchemaResult<Output> =
  | {
      /** Validation passed. */
      readonly status: "valid";
      /** Parsed output. */
      readonly value: Output;
    }
  | {
      /** Validation failed. */
      readonly status: "invalid";
      /** Issues with dotted paths, in schema order. */
      readonly issues: readonly NormalizedSchemaIssue[];
    };

/**
 * Whether a value implements the Standard Schema v1 protocol.
 *
 * @param candidate Value to test.
 * @returns Whether `candidate["~standard"].validate` is callable with version 1.
 */
export function isStandardSchema(candidate: unknown): candidate is StandardSchemaV1 {
  if ((typeof candidate !== "object" && typeof candidate !== "function") || candidate === null) {
    return false;
  }
  const props: unknown = Reflect.get(candidate, "~standard");
  return (
    typeof props === "object" &&
    props !== null &&
    Reflect.get(props, "version") === 1 &&
    typeof Reflect.get(props, "validate") === "function"
  );
}

/**
 * Convert a Standard Schema issue path to a dotted path.
 *
 * `{ key }` segments are unwrapped; symbols use their description.
 *
 * @param path Issue path.
 * @returns Dotted path, `""` for the root.
 */
export function formatSchemaPath(path: readonly StandardSchemaPathSegment[] | undefined): string {
  if (!path) return "";
  return path
    .map((segment) => {
      const key = typeof segment === "object" ? segment.key : segment;
      return typeof key === "symbol" ? (key.description ?? "") : String(key);
    })
    .join(".");
}

/**
 * Validate a value with a Standard Schema and normalize the result.
 *
 * Synchronous and asynchronous schemas are handled identically. Throwing
 * schemas propagate their error. Pure: safe on the server.
 *
 * @example
 * ```ts
 * const result = await validateStandardSchema(zodSchema, formValues);
 * if (result.status === "invalid") console.log(result.issues);
 * ```
 *
 * @typeParam Output Schema output type.
 * @param schema Standard Schema v1 validator.
 * @param value Value to validate.
 * @returns The normalized result.
 */
export async function validateStandardSchema<Output>(
  schema: StandardSchemaV1<unknown, Output>,
  value: unknown,
): Promise<NormalizedSchemaResult<Output>> {
  const result = await schema["~standard"].validate(value);
  if (result.issues === undefined) return { status: "valid", value: result.value };
  return {
    status: "invalid",
    issues: result.issues.map((issue) => ({
      path: formatSchemaPath(issue.path),
      message: issue.message,
    })),
  };
}
