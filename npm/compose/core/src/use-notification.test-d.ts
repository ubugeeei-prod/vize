/** Compile-only assertions for the `use-notification` type contracts. */

import { useWebNotification } from "./use-notification.ts";
import type { NotificationHost, NotificationPermissionState } from "./use-notification.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const notify = useWebNotification<{ readonly id: string }>();
type _DataIsTyped = Expect<Equal<typeof notify.data.value, { readonly id: string } | undefined>>;
type _PermissionIsClosed = Expect<
  Equal<typeof notify.permission.value, NotificationPermissionState>
>;

void notify.show("Done", { data: { id: "1" } });

Notification satisfies NotificationHost;

// @ts-expect-error data must match the declared type.
void notify.show("Done", { data: { id: 1 } });

// @ts-expect-error only known lifecycle events can be observed.
notify.on("hover", () => undefined);
