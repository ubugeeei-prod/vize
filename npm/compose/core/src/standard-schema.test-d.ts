/** Compile-only assertions for the `standard-schema` type contracts. */

import { isStandardSchema, validateStandardSchema } from "./standard-schema.ts";
import type {
  InferStandardSchemaInput,
  InferStandardSchemaOutput,
  NormalizedSchemaResult,
  StandardSchemaV1,
} from "./standard-schema.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const schema: StandardSchemaV1<string, number>;
type _Input = Expect<Equal<InferStandardSchemaInput<typeof schema>, string>>;
type _Output = Expect<Equal<InferStandardSchemaOutput<typeof schema>, number>>;

const result = validateStandardSchema(schema, "1");
type _NormalizedOutput = Expect<Equal<Awaited<typeof result>, NormalizedSchemaResult<number>>>;

declare const candidate: unknown;
if (isStandardSchema(candidate)) candidate["~standard"].vendor satisfies string;

// A third-party schema only needs the protocol shape.
({
  "~standard": { version: 1, vendor: "x", validate: () => ({ value: 1 }) },
}) satisfies StandardSchemaV1<unknown, number>;

({
  "~standard": {
    // @ts-expect-error only protocol version 1 is supported.
    version: 2,
    vendor: "x",
    validate: () => ({ value: 1 }),
  },
}) satisfies StandardSchemaV1;
