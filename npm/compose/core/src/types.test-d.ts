/** Compile-only assertions for public composable type contracts. */

import type { ShallowRef } from "vue";

import { anyAbortSignal } from "./abort-signal.js";
import { type AsyncResourceExecution, useAsyncResource } from "./async-resource.js";
import {
  availableCapability,
  type AvailableCapability,
  type CapabilityResult,
  isCapabilityAvailable,
  isCapabilityUnavailable,
  unavailableCapability,
  type UnavailableCapability,
} from "./capability.js";
import { type TextDirection, useLocale } from "./locale.js";
import { createDisposalScope, type DisposalError } from "./disposal-scope.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const combinedAbortSignal = anyAbortSignal(new Set<AbortSignal>());
type _AbortCompositionReturnsThePlatformSignal = Expect<
  Equal<typeof combinedAbortSignal, AbortSignal>
>;

// @ts-expect-error cancellation composition accepts AbortSignal inputs only.
anyAbortSignal([new AbortController()]);

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

const runtimeCapability = availableCapability({ kind: "camera" } as const);
const adapterCapability = availableCapability(1 as const, "native-camera");
const unsupportedCapability = unavailableCapability("unsupported");
const permissionCapability = unavailableCapability("permission-denied", {
  permission: "camera",
  canRequest: true,
} as const);

type _RuntimeCapabilityPreservesValueAndDefaultSource = Expect<
  Equal<typeof runtimeCapability, AvailableCapability<{ readonly kind: "camera" }, "runtime">>
>;
type _AdapterCapabilityPreservesLiteralSource = Expect<
  Equal<typeof adapterCapability, AvailableCapability<1, "native-camera">>
>;
type _UnavailableCapabilityPreservesReason = Expect<
  Equal<typeof unsupportedCapability, UnavailableCapability<"unsupported", undefined>>
>;
type _UnavailableCapabilityPreservesDetails = Expect<
  Equal<
    typeof permissionCapability,
    UnavailableCapability<
      "permission-denied",
      { readonly permission: "camera"; readonly canRequest: true }
    >
  >
>;

declare const capabilityResult: CapabilityResult<
  { readonly request: () => void },
  "unsupported" | "permission-denied",
  { readonly canRequest: boolean },
  "runtime" | "native-host"
>;

if (isCapabilityAvailable(capabilityResult)) {
  capabilityResult.value.request();
  type _AvailableGuardPreservesSource = Expect<
    Equal<typeof capabilityResult.source, "runtime" | "native-host">
  >;
  // @ts-expect-error the available branch has no unavailability reason.
  void capabilityResult.reason;
}

if (isCapabilityUnavailable(capabilityResult)) {
  type _UnavailableGuardPreservesReason = Expect<
    Equal<typeof capabilityResult.reason, "unsupported" | "permission-denied">
  >;
  capabilityResult.details.canRequest satisfies boolean;
  // @ts-expect-error the unavailable branch has no capability value.
  void capabilityResult.value;
}

const disposalOwner = createDisposalScope({ scope: false });
const cleanupRegistration = disposalOwner.add(() => undefined);
const disposalChild = disposalOwner.child();

disposalOwner.disposed satisfies boolean;
disposalOwner.size satisfies number;
cleanupRegistration.active satisfies boolean;
cleanupRegistration.unregister() satisfies boolean;
disposalChild.dispose();

// @ts-expect-error cleanup registration accepts synchronous cleanup only.
disposalOwner.add(async () => undefined);

declare const disposalFailure: DisposalError;
disposalFailure.code satisfies "VIZE_COMPOSE_DISPOSAL_FAILED";
