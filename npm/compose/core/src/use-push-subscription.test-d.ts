/** Compile-only assertions for the `use-push-subscription` type contracts. */

import { usePushSubscription } from "./use-push-subscription.ts";
import type {
  PushPermissionState,
  PushRegistrationLike,
  PushSubscriptionLike,
  PushSubscriptionSnapshot,
} from "./use-push-subscription.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const push = usePushSubscription();
type _Json = Expect<Equal<typeof push.json.value, PushSubscriptionSnapshot | null>>;
type _Permission = Expect<
  Equal<Awaited<ReturnType<typeof push.permissionState>>, PushPermissionState | "unsupported">
>;

declare const registration: ServiceWorkerRegistration;
registration satisfies PushRegistrationLike;

declare const subscription: PushSubscription;
subscription satisfies PushSubscriptionLike;

void push.subscribe({ applicationServerKey: new Uint8Array(65) });

// @ts-expect-error the key is required.
void push.subscribe({ userVisibleOnly: true });

// @ts-expect-error the subscription is read-only.
push.subscription.value = null;
