/** Compile-only assertions for the `use-fetch` type contracts. */

import { useFetch } from "./use-fetch.ts";
import type { FetchFailure, FetchResult, FetchStatus } from "./use-fetch.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const untyped = useFetch("/api");
type _JsonDefaultsToUnknown = Expect<Equal<typeof untyped.data.value, unknown>>;

interface User {
  readonly name: string;
}
const isUser = (value: unknown): value is User =>
  typeof value === "object" && value !== null && "name" in value;
const user = useFetch("/user", { validate: isUser });
type _ValidateInfersData = Expect<Equal<typeof user.data.value, User | undefined>>;

const text = useFetch("/t", { responseType: "text" });
type _TextInfersString = Expect<Equal<typeof text.data.value, string | undefined>>;
const blob = useFetch("/b", { responseType: "blob" });
type _BlobInfersBlob = Expect<Equal<typeof blob.data.value, Blob | undefined>>;

const parsed = useFetch("/p", { parse: async (response) => response.status });
type _ParseInfersData = Expect<Equal<typeof parsed.data.value, number | undefined>>;

type _ExecuteIsDiscriminated = Expect<
  Equal<Awaited<ReturnType<typeof user.execute>>, FetchResult<User>>
>;
type _StatusIsClosed = Expect<Equal<typeof user.status.value, FetchStatus>>;
type _ErrorIsTyped = Expect<Equal<typeof user.error.value, FetchFailure | undefined>>;

// @ts-expect-error response types are a closed union.
useFetch("/x", { responseType: "stream" });

// @ts-expect-error afterResponse must return the data type.
useFetch("/t", { responseType: "text", afterResponse: () => 1 });

// @ts-expect-error data is read-only.
user.data.value = { name: "x" };
