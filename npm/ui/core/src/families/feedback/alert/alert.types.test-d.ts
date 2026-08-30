/** Compile-only assertions for the public alert contract. */

import type { ShallowRef } from "vue";

import type { AlertExpose, AlertRole, AlertSlotState, AlertState, AlertVariant } from "./alert.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const roles: readonly AlertRole[] = ["alert", "status"];
export const variants: readonly AlertVariant[] = ["danger", "info", "success", "warning"];

type _StateIsLiteral = Expect<Equal<AlertState, "closed" | "open">>;
type _ElementIsTemplateRef = Expect<
  Equal<AlertExpose["element"], Readonly<ShallowRef<HTMLDivElement | null>>>
>;
type _SlotStateIsLiteral = Expect<
  Equal<
    AlertSlotState,
    {
      readonly open: boolean;
      readonly role: AlertRole;
      readonly state: AlertState;
      readonly variant: AlertVariant;
    }
  >
>;

// @ts-expect-error log is not an Alert live-region role.
export const invalidRole: AlertRole = "log";

// @ts-expect-error neutral is not a shipped Alert variant.
export const invalidVariant: AlertVariant = "neutral";
