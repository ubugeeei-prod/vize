/** Compile-only assertions for the `use-permission` type contracts. */

import { usePermission } from "./use-permission.ts";
import type { PermissionQueryState, PermissionsHost } from "./use-permission.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const camera = usePermission("camera");
type _StateIsClosed = Expect<Equal<typeof camera.state.value, PermissionQueryState>>;

usePermission("future-permission-name");
usePermission({ name: "midi", sysex: true });

declare const permissions: Permissions;
permissions satisfies PermissionsHost;

// @ts-expect-error the state is read-only.
camera.state.value = "granted";

// @ts-expect-error descriptors need a name.
usePermission({ sysex: true });

// @ts-expect-error the initial state is a closed union.
usePermission("camera", { initialState: "allowed" });
