import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import SwitchControl from "./switch-control.vue";
import { mountInteraction } from "./testing/mount.ts";

function switchFormValue(form: HTMLFormElement, name = "notifications"): FormDataEntryValue | null {
  return new FormData(form).get(name);
}

test("renders a named native switch with ARIA and form attributes", () => {
  const handle = mountInteraction(SwitchControl, {
    props: {
      id: "marketing-switch",
      name: "marketing",
      value: "enabled",
      defaultChecked: true,
      required: true,
      ariaLabel: "Marketing emails",
      ariaDescribedby: "marketing-help",
      ariaErrormessage: "marketing-error",
      ariaInvalid: true,
    },
    slots: { default: "Marketing emails" },
  });
  const control = handle.getByRole("switch", { name: "Marketing emails" }) as HTMLButtonElement;
  const hidden = control.querySelector("input[type='hidden']") as HTMLInputElement | null;

  assert.equal(control.tagName, "BUTTON");
  assert.equal(control.type, "button");
  assert.equal(control.id, "marketing-switch");
  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.equal(control.getAttribute("aria-required"), "true");
  assert.equal(control.getAttribute("aria-describedby"), "marketing-help");
  assert.equal(control.getAttribute("aria-errormessage"), "marketing-error");
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("data-vize-ui"), "switch");
  assert.equal(control.getAttribute("data-state"), "checked");
  assert.equal(control.getAttribute("data-checked"), "true");
  assert.equal(hidden?.name, "marketing");
  assert.equal(hidden?.value, "enabled");

  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === control, "exposed focus() must focus the switch");
  handle.unmount();
});

test("hydrates a generated switch id without changing the server contract", async () => {
  const SsrProbe = defineComponent({
    name: "SwitchGeneratedIdSsrProbe",
    setup: () => () =>
      h(SwitchControl, {
        ariaDescribedby: "notifications-help",
        ariaLabel: "Notifications",
        defaultChecked: true,
        name: "notifications",
        required: true,
      }),
  });
  const [serverHtml, repeatedHtml] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(serverHtml, repeatedHtml);

  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverControl = host.querySelector<HTMLButtonElement>("[role='switch']");
  assert.ok(serverRoot);
  assert.ok(serverControl);
  const serverId = serverControl.id;
  assert.match(serverId, /^vize-v-\d+-switch$/);
  assert.equal(serverControl.getAttribute("aria-checked"), "true");
  assert.equal(serverControl.getAttribute("aria-describedby"), "notifications-help");

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    const hydratedControl = host.querySelector<HTMLButtonElement>("[role='switch']");
    assert.ok(hydratedControl);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydratedControl.id, serverId);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("uncontrolled switch toggles with pointer activation and form data", async () => {
  const recorded: [event: string, value: boolean, nativeEvent?: unknown][] = [];
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(SwitchControl, {
          ariaLabel: "Notifications",
          name: "notifications",
          onChange: (value: boolean, nativeEvent: MouseEvent) =>
            recorded.push(["change", value, nativeEvent]),
          "onUpdate:modelValue": (value: boolean) => recorded.push(["update:modelValue", value]),
          value: "email",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const control = handle.getByRole("switch", { name: "Notifications" });

  assert.equal(switchFormValue(form), null);
  await handle.click(control);
  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.equal(control.getAttribute("data-state"), "checked");
  assert.equal(control.getAttribute("data-checked"), "true");
  assert.equal(switchFormValue(form), "email");

  await handle.click(control);
  assert.equal(control.getAttribute("aria-checked"), "false");
  assert.equal(control.getAttribute("data-state"), "unchecked");
  assert.equal(switchFormValue(form), null);

  assert.deepEqual(
    recorded.map(([event, value]) => [event, value]),
    [
      ["update:modelValue", true],
      ["change", true],
      ["update:modelValue", false],
      ["change", false],
    ],
  );
  assert.ok(recorded[1]?.[2] instanceof MouseEvent);
  handle.unmount();
});

test("controlled checked state wins until the parent accepts the request", async () => {
  const handle = mountInteraction(SwitchControl, {
    props: { ariaLabel: "Notifications", modelValue: false, name: "notifications" },
    record: ["update:modelValue", "change"],
  });
  const control = handle.getByRole("switch") as HTMLButtonElement;

  await handle.click(control);
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", true],
      ["change", true],
    ],
  );
  assert.equal(control.getAttribute("aria-checked"), "false");
  assert.equal(control.querySelector("input[type='hidden']"), null);

  await handle.wrapper.setProps({ modelValue: true });
  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.ok(control.querySelector("input[type='hidden']") instanceof HTMLInputElement);
  handle.unmount();
});

test("defaultChecked seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(SwitchControl, {
          ariaLabel: "Notifications",
          defaultChecked: true,
          name: "notifications",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const control = handle.getByRole("switch", { name: "Notifications" });

  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.equal(switchFormValue(form), "on");
  await handle.click(control);
  assert.equal(control.getAttribute("aria-checked"), "false");
  assert.equal(switchFormValue(form), null);

  form.reset();
  await nextTick();
  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.equal(switchFormValue(form), "on");
  handle.unmount();
});

test("keyboard activation toggles with Enter and Space", async () => {
  const handle = mountInteraction(SwitchControl, { props: { ariaLabel: "Notifications" } });
  const control = handle.getByRole("switch");
  control.focus();

  const enter = await handle.press(control, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(control.getAttribute("aria-checked"), "true");

  const space = await handle.press(control, " ");
  assert.equal(space.activated, true);
  assert.equal(control.getAttribute("aria-checked"), "false");
  assert.equal(handle.wrapper.emitted("change")?.length, 2);
  handle.unmount();
});

test("disabled and read-only switches keep availability semantics", async () => {
  const disabled = mountInteraction(SwitchControl, {
    props: {
      ariaLabel: "Notifications",
      defaultChecked: true,
      disabled: true,
      name: "notifications",
    },
  });
  const disabledControl = disabled.getByRole("switch") as HTMLButtonElement;
  assert.equal(disabledControl.disabled, true);
  assert.equal(disabledControl.getAttribute("aria-disabled"), "true");
  assert.equal(disabledControl.getAttribute("data-state"), "disabled");
  assert.equal(disabledControl.querySelector("input[type='hidden']"), null);

  await disabled.click(disabledControl);
  assert.equal(disabledControl.getAttribute("aria-checked"), "true");
  assert.equal(disabled.wrapper.emitted("change"), undefined);
  assert.ok((await disabled.tab()) === null);
  disabled.unmount();

  const readOnly = mountInteraction(SwitchControl, {
    props: {
      ariaLabel: "Notifications",
      defaultChecked: true,
      name: "notifications",
      readOnly: true,
    },
  });
  const readOnlyControl = readOnly.getByRole("switch") as HTMLButtonElement;
  assert.equal(readOnlyControl.disabled, false);
  assert.equal(readOnlyControl.getAttribute("aria-readonly"), "true");
  assert.equal(readOnlyControl.getAttribute("data-state"), "readonly");
  assert.ok(readOnlyControl.querySelector("input[type='hidden']") instanceof HTMLInputElement);
  assert.ok((await readOnly.tab()) === readOnlyControl);

  await readOnly.click(readOnlyControl);
  await readOnly.press(readOnlyControl, " ");
  assert.equal(readOnlyControl.getAttribute("aria-checked"), "true");
  assert.equal(readOnly.wrapper.emitted("change"), undefined);
  readOnly.unmount();
});

test("exposes focus, toggle, setChecked, reset, and slot state", async () => {
  const handle = mountInteraction(SwitchControl, {
    props: { defaultChecked: false },
    slots: {
      default: (state: { checked: boolean; disabled: boolean; readOnly: boolean }) =>
        `checked:${state.checked} disabled:${state.disabled} readonly:${state.readOnly}`,
    },
  });
  const control = handle.getByRole("switch");
  const exposed = handle.exposes<{
    focus: (options?: FocusOptions) => void;
    reset: () => boolean;
    setChecked: (value: boolean) => boolean;
    toggle: () => boolean;
  }>();

  assert.equal(handle.root().textContent, "checked:false disabled:false readonly:false");
  assert.equal(exposed.toggle(), true);
  await nextTick();
  assert.equal(control.getAttribute("aria-checked"), "true");
  assert.equal(handle.root().textContent, "checked:true disabled:false readonly:false");

  assert.equal(exposed.setChecked(true), false);
  assert.equal(exposed.setChecked(false), true);
  await nextTick();
  assert.equal(control.getAttribute("aria-checked"), "false");

  control.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === control);

  exposed.setChecked(true);
  await nextTick();
  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(control.getAttribute("aria-checked"), "false");
  handle.unmount();
});
