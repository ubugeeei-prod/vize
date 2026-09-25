/** Compile-only assertions for the `use-service-worker` type contracts. */

import { useServiceWorker } from "./use-service-worker.ts";
import type {
  ServiceWorkerContainerLike,
  ServiceWorkerRegistrationLike,
  ServiceWorkerStateName,
} from "./use-service-worker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const sw = useServiceWorker("/sw.js");
type _WaitingState = Expect<Equal<typeof sw.state.value.waiting, ServiceWorkerStateName | null>>;
type _Registration = Expect<
  Equal<Awaited<ReturnType<typeof sw.register>>, ServiceWorkerRegistrationLike | null>
>;

declare const container: ServiceWorkerContainer;
container satisfies ServiceWorkerContainerLike;

// @ts-expect-error only classic and module scripts exist.
useServiceWorker("/sw.js", { type: "worker" });

// @ts-expect-error the registration is read-only.
sw.registration.value = null;
