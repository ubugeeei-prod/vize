import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { createErrorSummary, useErrorSummary, type ErrorSummaryField } from "./error-summary.ts";
import ErrorSummary from "./error-summary.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const invalidFields: readonly ErrorSummaryField[] = [
  { id: "email", label: "Email", message: "Enter a valid address" },
  { id: "name", message: "Enter your name" },
];

function mountSummary(props: Record<string, unknown> = {}) {
  return mountInteraction(ErrorSummary, {
    props: { heading: "There is a problem", ...props },
    record: ["fieldFocus", "restore"],
  });
}

/** Attach a focusable control the summary links can target. */
function attachControl(tag: "button" | "input", id: string): HTMLElement {
  const control = document.createElement(tag);
  control.id = id;
  if (control instanceof HTMLInputElement) control.type = "text";
  document.body.append(control);
  return control;
}

test("stays out of the tree while every field is valid", () => {
  const handle = mountSummary();
  assert.equal(handle.root().getAttribute("data-vize-ui"), "error-summary-host");
  assert.equal(handle.root().querySelector('[data-vize-ui="error-summary"]'), null);
  handle.unmount();
});

test("lists invalid fields and takes focus", async () => {
  const submit = attachControl("button", "submit");
  submit.focus();
  const handle = mountSummary();
  await handle.wrapper.setProps({ fields: invalidFields });
  await nextTick();

  const summary = handle.root().querySelector('[data-vize-ui="error-summary"]');
  assert.ok(summary instanceof HTMLElement);
  assert.equal(summary.getAttribute("role"), "group");
  assert.equal(summary.getAttribute("tabindex"), "-1");
  const labelledBy = summary.getAttribute("aria-labelledby");
  assert.ok(labelledBy);
  const heading = handle.root().querySelector(`[id="${labelledBy}"]`);
  assert.equal(heading?.textContent?.trim(), "There is a problem");

  const links = [...handle.root().querySelectorAll('[data-vize-ui="error-summary-link"]')];
  assert.deepEqual(
    links.map((link) => link.getAttribute("href")),
    ["#email", "#name"],
  );
  assert.deepEqual(
    links.map((link) => link.textContent?.trim()),
    ["Email: Enter a valid address", "Enter your name"],
  );
  assert.equal(handle.activeElement(), summary);
  handle.unmount();
  submit.remove();
});

test("moves focus to an invalid control from its link", async () => {
  const email = attachControl("input", "email");
  const handle = mountSummary({ fields: invalidFields, autoFocus: false });
  const link = handle.root().querySelector('[data-vize-ui="error-summary-link"]');
  assert.ok(link instanceof HTMLElement);
  await handle.click(link);
  assert.equal(handle.activeElement(), email);
  assert.deepEqual(handle.recorded(), [{ event: "fieldFocus", payload: [invalidFields[0]] }]);

  const exposed = handle.exposes<{ focusField: (id: string) => HTMLElement | null }>();
  assert.equal(exposed.focusField("unknown"), null);
  assert.equal(handle.activeElement(), email);
  handle.unmount();
  email.remove();
});

test("restores focus when every field is repaired", async () => {
  const submit = attachControl("button", "submit");
  submit.focus();
  const handle = mountSummary();
  await handle.wrapper.setProps({ fields: invalidFields });
  await nextTick();
  const summary = handle.root().querySelector('[data-vize-ui="error-summary"]');
  assert.equal(handle.activeElement(), summary);

  await handle.wrapper.setProps({ fields: [] });
  await nextTick();
  assert.equal(handle.root().querySelector('[data-vize-ui="error-summary"]'), null);
  assert.equal(handle.activeElement(), submit);
  assert.deepEqual(handle.recorded(), [{ event: "restore", payload: [submit] }]);
  handle.unmount();
  submit.remove();
});

test("does not steal focus after a repair", async () => {
  const submit = attachControl("button", "submit");
  const other = attachControl("input", "other");
  submit.focus();
  const handle = mountSummary();
  await handle.wrapper.setProps({ fields: invalidFields });
  await nextTick();
  other.focus();

  await handle.wrapper.setProps({ fields: [] });
  await nextTick();
  assert.equal(handle.activeElement(), other);
  assert.deepEqual(handle.recorded(), []);

  const exposed = handle.exposes<{ restoreFocus: () => boolean }>();
  assert.equal(exposed.restoreFocus(), false);
  handle.unmount();
  submit.remove();
  other.remove();
});

test("respects autoFocus false", async () => {
  const submit = attachControl("button", "submit");
  submit.focus();
  const handle = mountSummary({ autoFocus: false });
  await handle.wrapper.setProps({ fields: invalidFields });
  await nextTick();
  assert.ok(handle.root().querySelector('[data-vize-ui="error-summary"]'));
  assert.equal(handle.activeElement(), submit);
  handle.unmount();
  submit.remove();
});

test("rejects invalid fields options", () => {
  assert.throws(
    () =>
      createErrorSummary({
        fields: [
          { id: "email", message: "first" },
          { id: "email", message: "second" },
        ],
      }),
    /VIZE_UI_ERROR_SUMMARY_OPTION/,
  );
  assert.throws(
    () => createErrorSummary({ fields: [{ id: "", message: "empty" }] }),
    /VIZE_UI_ERROR_SUMMARY_OPTION/,
  );
  assert.throws(
    () => createErrorSummary({ autoFocus: "yes" as never }),
    /VIZE_UI_ERROR_SUMMARY_OPTION/,
  );
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => useErrorSummary(), /VIZE_UI_ERROR_SUMMARY_SETUP/);
});

test("rejects use after dispose", () => {
  const controller = createErrorSummary();
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.focusSummary(), /VIZE_UI_ERROR_SUMMARY_DISPOSED/);
  assert.throws(() => controller.focusField("email"), /VIZE_UI_ERROR_SUMMARY_DISPOSED/);
  assert.throws(() => controller.restoreFocus(), /VIZE_UI_ERROR_SUMMARY_DISPOSED/);
});
