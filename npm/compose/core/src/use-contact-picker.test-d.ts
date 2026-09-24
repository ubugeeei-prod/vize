/** Compile-only assertions for the `use-contact-picker` type contracts. */

import { useContactPicker } from "./use-contact-picker.ts";
import type { ContactProperty } from "./use-contact-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const picker = useContactPicker();

async function narrow(): Promise<void> {
  const result = await picker.select(["name", "email"]);
  if (result.status === "selected") {
    const [first] = result.contacts;
    type _Keys = Expect<Equal<keyof NonNullable<typeof first>, "name" | "email">>;
    type _Email = Expect<Equal<NonNullable<typeof first>["email"], readonly string[]>>;
    // @ts-expect-error tel was not requested.
    void first?.tel;
  }
}
void narrow;

type _Properties = Expect<
  Equal<Awaited<ReturnType<typeof picker.getProperties>>, ContactProperty[]>
>;

// @ts-expect-error unknown contact property.
void picker.select(["birthday"]);

// @ts-expect-error pending is read-only.
picker.pending.value = true;
