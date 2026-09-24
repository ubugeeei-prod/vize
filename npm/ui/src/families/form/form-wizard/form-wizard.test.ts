import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import FormWizard from "./form-wizard.vue";
import FormWizardBack from "./form-wizard-back.vue";
import FormWizardNext from "./form-wizard-next.vue";
import FormWizardProgress from "./form-wizard-progress.vue";
import FormWizardStep from "./form-wizard-step.vue";
import type {
  FormWizardDraftStore,
  FormWizardExpose,
  FormWizardSlotState,
  FormWizardSnapshot,
  FormWizardValidationContext,
} from "./form-wizard-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

type Step = "account" | "profile" | "review";
const steps: readonly Step[] = ["account", "profile", "review"];

function mountWizard(props: Record<string, unknown> = {}) {
  return mountInteraction(FormWizard, {
    props: { steps, id: "signup", ariaLabel: "Sign up", ...props },
    record: ["update:modelValue", "change", "blocked", "complete"],
    slots: {
      default: (state: FormWizardSlotState<Step>) => [
        ...steps.map((step) =>
          h(
            FormWizardStep,
            { key: step, step, label: step },
            { default: () => h("input", { "aria-label": step }) },
          ),
        ),
        h(FormWizardProgress),
        h(FormWizardBack, null, { default: () => "Back" }),
        h(FormWizardNext, null, {
          default: ({ isLast }: { isLast: boolean }) => (isLast ? "Finish" : "Next"),
        }),
        h("output", `${state.current}:${state.index}:${state.progress}:${state.visited.join(",")}`),
      ],
    },
  });
}

function parts(root: HTMLElement) {
  const panels = [...root.querySelectorAll<HTMLElement>('[data-vize-ui="form-wizard-step"]')];
  const next = root.querySelector('[data-vize-ui="form-wizard-next"]');
  const back = root.querySelector('[data-vize-ui="form-wizard-back"]');
  const progress = root.querySelector("progress");
  assert.ok(next instanceof HTMLButtonElement && back instanceof HTMLButtonElement);
  assert.ok(progress instanceof HTMLProgressElement);
  return { panels, next, back, progress, output: () => root.querySelector("output")?.textContent };
}

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index += 1) {
    await Promise.resolve();
    await nextTick();
  }
}

function memoryDraft(initial?: FormWizardSnapshot<string>) {
  const saved: FormWizardSnapshot<Step>[] = [];
  let cleared = 0;
  const store: FormWizardDraftStore<Step> = {
    load: () => initial,
    save: (snapshot) => saved.push(snapshot),
    clear: () => {
      cleared += 1;
    },
  };
  return { store, saved, cleared: () => cleared };
}

test("renders every step panel with only the current one visible", () => {
  const handle = mountWizard();
  const { panels, next, back, progress, output } = parts(handle.root());

  assert.equal(handle.root().getAttribute("role"), "group");
  assert.equal(handle.root().getAttribute("data-step"), "account");
  assert.deepEqual(
    panels.map((panel) => [panel.id, panel.hidden, panel.getAttribute("aria-current")]),
    [
      ["signup-account", false, "step"],
      ["signup-profile", true, null],
      ["signup-review", true, null],
    ],
  );
  assert.equal(panels[1]?.getAttribute("data-state"), "upcoming");
  assert.equal(back.disabled, true);
  assert.equal(next.textContent, "Next");
  assert.equal(next.getAttribute("aria-controls"), "signup-account");
  assert.equal(progress.max, 3);
  assert.equal(progress.value, 1);
  assert.equal(progress.getAttribute("aria-valuetext"), "Step 1 of 3");
  assert.equal(output(), "account:0:0:account");
  handle.unmount();
});

test("next and back move between steps, focus the panel, and complete on the last step", async () => {
  const handle = mountWizard();
  const { panels, next, back, output } = parts(handle.root());

  next.click();
  await settle();
  assert.equal(output(), "profile:1:0.5:account,profile");
  assert.ok(document.activeElement === panels[1], "focus moves to the new step panel");
  assert.equal(panels[0]?.getAttribute("data-state"), "visited");
  next.click();
  await settle();
  assert.equal(next.textContent, "Finish");
  back.click();
  await settle();
  assert.equal(output(), "profile:1:0.5:account,profile,review");
  next.click();
  await settle();
  next.click();
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "complete");
  assert.deepEqual(
    handle.recorded().filter((entry) => entry.event !== "update:modelValue"),
    [
      { event: "change", payload: ["profile", "account", "forward"] },
      { event: "change", payload: ["review", "profile", "forward"] },
      { event: "change", payload: ["profile", "review", "back"] },
      { event: "change", payload: ["review", "profile", "forward"] },
      { event: "complete", payload: [] },
    ],
  );
  handle.unmount();
});

test("validation gates block forward moves and report the blocked step", async () => {
  const calls: FormWizardValidationContext<Step>[] = [];
  let allowProfile = false;
  const handle = mountWizard({
    validate: async (context: FormWizardValidationContext<Step>) => {
      calls.push(context);
      await Promise.resolve();
      return context.step !== "profile" || allowProfile;
    },
  });
  const { next, back, output } = parts(handle.root());
  const api = handle.exposes<FormWizardExpose<Step>>();

  const pending = api.next();
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "validating");
  assert.equal(handle.root().getAttribute("aria-busy"), "true");
  assert.equal(next.disabled, true);
  assert.equal(await pending, true);
  await settle();
  next.click();
  await settle();
  assert.equal(output()?.split(":")[0], "profile", "invalid step stays current");
  assert.deepEqual(handle.recorded().at(-1), { event: "blocked", payload: ["profile", "review"] });
  assert.deepEqual(
    calls.map((call) => [call.step, call.target, call.signal.aborted]),
    [
      ["account", "profile", false],
      ["profile", "review", false],
    ],
  );
  allowProfile = true;
  back.click();
  await settle();
  assert.equal(calls.length, 2, "going back never validates");
  handle.unmount();
});

test("goTo jumps back freely, forward only through visited steps, validating in between", async () => {
  const validated: Step[] = [];
  const handle = mountWizard({
    validate: ({ step }: FormWizardValidationContext<Step>) => {
      validated.push(step);
      return true;
    },
  });
  const api = handle.exposes<FormWizardExpose<Step>>();

  assert.equal(await api.goTo("review"), false, "linear wizards cannot skip unvisited steps");
  assert.equal(await api.goTo("profile"), true);
  assert.equal(await api.goTo("review"), true);
  assert.equal(await api.goTo("account"), true);
  assert.deepEqual(validated, ["account", "profile"]);
  assert.equal(await api.goTo("review"), true, "visited steps can be revisited");
  assert.deepEqual(validated, ["account", "profile", "account", "profile"]);
  handle.unmount();

  const free = mountWizard({ linear: false });
  assert.equal(await free.exposes<FormWizardExpose<Step>>().goTo("review"), true);
  free.unmount();
});

test("drafts restore after mount, save every change, and clear on completion or reset", async () => {
  const draft = memoryDraft({ step: "profile", visited: ["account", "profile", "bogus"] });
  const handle = mountWizard({ draft: draft.store });
  await settle();
  const { next, output } = parts(handle.root());

  assert.equal(output(), "profile:1:0.5:account,profile");
  next.click();
  await settle();
  assert.deepEqual(draft.saved.at(-1), {
    step: "review",
    visited: ["account", "profile", "review"],
  });
  next.click();
  await settle();
  assert.equal(draft.cleared(), 1);
  handle.exposes<FormWizardExpose<Step>>().reset();
  await settle();
  assert.equal(output(), "account:0:0:account");
  assert.equal(draft.cleared(), 2);
  handle.unmount();

  const ignored = memoryDraft({ step: "unknown", visited: [] });
  const fresh = mountWizard({ draft: ignored.store });
  await settle();
  assert.equal(
    parts(fresh.root()).output(),
    "account:0:0:account",
    "unknown saved steps are ignored",
  );
  fresh.unmount();
});

test("controlled steps win until the parent accepts them", async () => {
  const handle = mountWizard({ modelValue: "account" });
  const { next, output } = parts(handle.root());

  next.click();
  await settle();
  assert.deepEqual(handle.recorded()[0], { event: "update:modelValue", payload: ["profile"] });
  assert.equal(output()?.split(":")[0], "account");
  await handle.wrapper.setProps({ modelValue: "review" });
  assert.equal(output()?.split(":")[0], "review");
  handle.unmount();
});

test("rejects empty step lists and parts outside a FormWizard", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(FormWizard, { props: { steps: [] } }),
      /VIZE_UI_FORM_WIZARD_STEPS/,
    );
    assert.throws(() => mountInteraction(FormWizardNext), /VIZE_UI_CONTEXT_MISSING: FormWizard/);
    assert.throws(() => mountInteraction(FormWizardBack), /VIZE_UI_CONTEXT_MISSING: FormWizard/);
    assert.throws(
      () => mountInteraction(FormWizardProgress),
      /VIZE_UI_CONTEXT_MISSING: FormWizard/,
    );
    assert.throws(
      () => mountInteraction(FormWizardStep, { props: { step: "a" } }),
      /VIZE_UI_CONTEXT_MISSING: FormWizard/,
    );
  } finally {
    console.warn = warn;
  }
});
