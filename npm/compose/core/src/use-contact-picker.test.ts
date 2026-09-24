import assert from "node:assert/strict";
import { test } from "node:test";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useContactPicker } from "./use-contact-picker.ts";
import type { ContactInfo, ContactProperty, ContactsManagerLike } from "./use-contact-picker.ts";

class FakeContacts implements ContactsManagerLike {
  calls: { properties: ContactProperty[]; multiple: boolean | undefined }[] = [];
  next: readonly Partial<ContactInfo>[] | Error = [];
  properties: readonly string[] = ["name", "email", "tel", "future"];

  getProperties(): Promise<readonly string[]> {
    return Promise.resolve(this.properties);
  }

  select(
    properties: ContactProperty[],
    options?: { multiple?: boolean },
  ): Promise<readonly Partial<ContactInfo>[]> {
    this.calls.push({ properties, multiple: options?.multiple });
    return this.next instanceof Error ? Promise.reject(this.next) : Promise.resolve(this.next);
  }
}

void test("select returns contacts with exactly the requested keys", async () => {
  const contacts = new FakeContacts();
  contacts.next = [
    { name: ["Ada"], email: ["ada@example.com"], tel: ["+81"] },
    { name: ["Grace"] },
  ];
  const picker = useContactPicker({ contacts });
  assert.equal(picker.supported.value, true);

  const pending = picker.select(["name", "email"], { multiple: true });
  assert.equal(picker.pending.value, true);
  const result = await pending;
  assert.deepEqual(result, {
    status: "selected",
    contacts: [
      { name: ["Ada"], email: ["ada@example.com"] },
      { name: ["Grace"], email: [] },
    ],
  });
  assert.equal(picker.pending.value, false);
  assert.deepEqual(contacts.calls, [{ properties: ["name", "email"], multiple: true }]);
});

void test("dismissal resolves an empty selection and multiple defaults to false", async () => {
  const contacts = new FakeContacts();
  const picker = useContactPicker({ contacts });
  assert.deepEqual(await picker.select(["tel"]), { status: "selected", contacts: [] });
  assert.equal(contacts.calls[0]?.multiple, false);
});

void test("failures land in error and clear on success", async () => {
  const contacts = new FakeContacts();
  const denial = new DOMException("no activation", "SecurityError");
  contacts.next = denial;
  const picker = useContactPicker({ contacts });
  assert.deepEqual(await picker.select(["name"]), { status: "failed", error: denial });
  assert.equal(picker.error.value, denial);
  contacts.next = [];
  await picker.select(["name"]);
  assert.equal(picker.error.value, undefined);
});

void test("rejects an empty property list", () => {
  const picker = useContactPicker({ contacts: new FakeContacts() });
  assert.throws(() => picker.select([]), /VIZE_COMPOSE_CONTACT_PICKER_NO_PROPERTIES/u);
});

void test("getProperties filters unknown properties and swallows failures", async () => {
  const contacts = new FakeContacts();
  const picker = useContactPicker({ contacts });
  assert.deepEqual(await picker.getProperties(), ["name", "email", "tel"]);
  contacts.getProperties = () => Promise.reject(new Error("boom"));
  assert.deepEqual(await picker.getProperties(), []);
});

void test("reports unsupported without a host", async () => {
  const picker = useContactPicker({ contacts: null });
  assert.equal(picker.supported.value, false);
  assert.deepEqual(await picker.select(["name"]), { status: "unsupported", error: undefined });
  assert.deepEqual(await picker.getProperties(), []);
});

void test("server rendering requests nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const picker = useContactPicker();
    return { supported: picker.supported, pending: picker.pending };
  });
  assert.equal(state, '{"supported":false,"pending":false}');
});
