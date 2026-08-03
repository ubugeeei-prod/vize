/** Compile-only assertions for public composable type contracts. */

import type { ShallowRef } from "vue";

import { type AsyncResourceExecution, useAsyncResource } from "./async-resource.js";
import { type TextDirection, useLocale } from "./locale.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const resource = useAsyncResource(async (_context, id: 1 | 2) => ({ id }) as const);

interface LoadFailure {
  readonly code: "load-failed";
}

const failureTypedResource = useAsyncResource<{ readonly id: 1 | 2 }, [id: 1 | 2], LoadFailure>(
  async (_context, id) => ({ id }),
);

type _ExecuteArgumentsStayExact = Expect<Equal<Parameters<typeof resource.execute>, [id: 1 | 2]>>;
type _ExecutionPreservesDataAndFailure = Expect<
  Equal<
    Awaited<ReturnType<typeof resource.execute>>,
    AsyncResourceExecution<{ readonly id: 1 | 2 }, unknown>
  >
>;
type _DataRefPreservesTheLoaderResult = Expect<
  Equal<typeof resource.data, Readonly<ShallowRef<{ readonly id: 1 | 2 } | undefined>>>
>;
type _ExecutionPreservesAnExplicitFailure = Expect<
  Equal<
    Awaited<ReturnType<typeof failureTypedResource.execute>>,
    AsyncResourceExecution<{ readonly id: 1 | 2 }, LoadFailure>
  >
>;

// @ts-expect-error execute keeps the loader's argument tuple.
void resource.execute("1");

const locale = useLocale("en");

type _TextDirectionStaysClosed = Expect<Equal<TextDirection, "ltr" | "rtl">>;
type _LocaleDirectionNeverWidensToUndefined = Expect<
  Equal<typeof locale.direction.value, TextDirection>
>;

// @ts-expect-error the public direction ref never exposes capability absence.
const _undefinedDirection: undefined = locale.direction.value;
