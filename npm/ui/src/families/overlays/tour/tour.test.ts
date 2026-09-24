import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type {
  TourAfterLeaveContext,
  TourBeforeEnterContext,
  TourRootExpose,
  TourSlotState,
  TourStepDefinition,
} from "./tour.ts";
import TourArrow from "./tour-arrow.vue";
import TourClose from "./tour-close.vue";
import TourContent from "./tour-content.vue";
import TourDescription from "./tour-description.vue";
import TourNext from "./tour-next.vue";
import TourPrev from "./tour-prev.vue";
import TourProgress from "./tour-progress.vue";
import TourRoot from "./tour-root.vue";
import TourSpotlight from "./tour-spotlight.vue";
import TourStep from "./tour-step.vue";
import TourTitle from "./tour-title.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface DemoStep extends TourStepDefinition {
  readonly title: string;
}

const defaultSteps: readonly DemoStep[] = [
  { value: "welcome", title: "Welcome" },
  { value: "search", title: "Search", target: "#tour-search", placement: "bottom-start" },
  { value: "profile", title: "Profile", target: "#tour-profile" },
];

async function settle(): Promise<void> {
  for (let round = 0; round < 4; round++) await nextTick();
}

function content(): HTMLElement | null {
  const element = document.querySelector('[data-vize-ui="tour-content"]');
  assert.ok(element === null || element instanceof HTMLElement);
  return element;
}

function requireContent(): HTMLElement {
  const element = content();
  assert.ok(element, "tour content must be rendered");
  return element;
}

function part(name: string): HTMLElement {
  const element = document.querySelector(`[data-vize-ui="tour-${name}"]`);
  assert.ok(element instanceof HTMLElement, `tour-${name} must be rendered`);
  return element;
}

interface Fixture {
  readonly launcher: HTMLButtonElement;
  readonly search: HTMLElement;
  readonly profile: HTMLElement;
  readonly scrolls: ScrollIntoViewOptions[];
  readonly cleanup: () => void;
}

function createTargets(): Fixture {
  const launcher = document.createElement("button");
  launcher.type = "button";
  launcher.textContent = "Start tour";
  const search = document.createElement("input");
  search.id = "tour-search";
  const profile = document.createElement("button");
  profile.id = "tour-profile";
  profile.textContent = "Profile";
  const scrolls: ScrollIntoViewOptions[] = [];
  for (const element of [search, profile]) {
    element.scrollIntoView = (options?: boolean | ScrollIntoViewOptions) => {
      if (typeof options === "object") scrolls.push(options);
    };
  }
  document.body.append(launcher, search, profile);
  return {
    launcher,
    search,
    profile,
    scrolls,
    cleanup: () => {
      launcher.remove();
      search.remove();
      profile.remove();
    },
  };
}

function mountTour(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(TourRoot, {
    props: { id: "onboarding", steps: defaultSteps, ...rootProps },
    record: [
      "update:open",
      "update:step",
      "step-change",
      "complete",
      "dismiss",
      "navigation-cancel",
      "navigation-error",
    ],
    slots: {
      default: (state: TourSlotState<DemoStep>) => [
        h(TourSpotlight, { padding: 6, radius: 8 }),
        h(TourContent, { portalDisabled: true, ...contentProps }, () => [
          h(TourTitle, null, () => state.step?.title ?? ""),
          h(TourDescription, null, () => `Step ${state.value ?? "none"}`),
          h(TourStep, { value: "search" }, ({ index }) =>
            h("p", { "data-search-copy": String(index) }, "Type to search"),
          ),
          h(TourProgress),
          h(TourPrev, null, () => "Back"),
          h(TourNext, null, ({ last }) => (last ? "Finish" : "Next")),
          h(TourClose, { reason: "skip" }, () => "Skip tour"),
          h(TourArrow),
        ]),
      ],
    },
  });
}

test("stays closed until started, then wires an accessible non-modal dialog", async () => {
  const targets = createTargets();
  const handle = mountTour();
  await settle();

  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(handle.root().getAttribute("data-step"), "welcome");
  assert.equal(content(), null);
  assert.equal(part("spotlight").hidden, true);

  targets.launcher.focus();
  assert.equal(handle.exposes<TourRootExpose<DemoStep>>().start(), true);
  await settle();
  const dialog = requireContent();

  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.equal(dialog.id, "onboarding-content");
  assert.equal(dialog.getAttribute("role"), "dialog");
  assert.equal(dialog.getAttribute("aria-modal"), null);
  assert.equal(dialog.getAttribute("aria-labelledby"), "onboarding-title");
  assert.equal(dialog.getAttribute("aria-describedby"), "onboarding-description");
  assert.equal(part("title").id, "onboarding-title");
  assert.equal(part("title").tagName, "H2");
  assert.equal(part("title").textContent, "Welcome");
  assert.equal(part("description").id, "onboarding-description");
  assert.equal(dialog.getAttribute("data-target"), "none");
  assert.equal(dialog.getAttribute("data-placement"), "center");
  assert.equal(dialog.getAttribute("data-side"), "center");
  assert.equal(dialog.getAttribute("data-first"), "true");
  assert.equal(part("positioner").style.cssText, "");
  assert.equal(part("arrow").hidden, true);
  assert.equal(part("progress").textContent, "1 / 3");
  assert.equal((part("prev") as HTMLButtonElement).disabled, true);
  assert.equal(part("next").textContent, "Next");
  assert.equal(document.activeElement, dialog);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  handle.unmount();
  targets.cleanup();
});

test("next and previous walk steps, anchor resolved targets, and complete on the last step", async () => {
  const targets = createTargets();
  const handle = mountTour({ defaultOpen: true });
  await settle();
  targets.launcher.focus();

  await handle.click(part("next"));
  await settle();
  const dialog = requireContent();
  assert.equal(dialog.getAttribute("data-step"), "search");
  assert.equal(dialog.getAttribute("data-target"), "resolved");
  assert.equal(dialog.getAttribute("data-side"), "bottom");
  assert.equal(dialog.getAttribute("data-align"), "start");
  assert.equal(targets.search.getAttribute("data-vize-tour-target"), "active");
  assert.deepEqual(targets.scrolls.at(-1), {
    behavior: "smooth",
    block: "center",
    inline: "nearest",
  });
  assert.match(part("positioner").style.cssText, /position: fixed/);
  assert.equal(part("arrow").hidden, false);
  assert.equal(part("progress").textContent, "2 / 3");
  assert.equal(document.querySelector("[data-search-copy]")?.getAttribute("data-search-copy"), "1");
  assert.equal(document.activeElement, dialog);

  await handle.click(part("prev"));
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "welcome");
  assert.equal(targets.search.hasAttribute("data-vize-tour-target"), false);
  assert.equal(document.querySelector("[data-search-copy]"), null);

  await handle.click(part("next"));
  await handle.click(part("next"));
  await settle();
  assert.equal(requireContent().getAttribute("data-last"), "true");
  assert.equal(part("next").textContent, "Finish");
  assert.equal(part("next").getAttribute("data-last"), "true");
  assert.equal(targets.profile.getAttribute("data-vize-tour-target"), "active");

  await handle.click(part("next"));
  await settle();
  assert.equal(content(), null);
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(handle.wrapper.emitted("complete")?.length, 1);
  assert.ok(handle.wrapper.emitted("complete")?.[0]?.[0] instanceof MouseEvent);
  assert.equal(targets.profile.hasAttribute("data-vize-tour-target"), false);
  assert.deepEqual(handle.wrapper.emitted("update:step"), [
    ["search"],
    ["welcome"],
    ["search"],
    ["profile"],
  ]);
  assert.deepEqual(handle.wrapper.emitted("step-change")?.[0]?.slice(0, 2), ["search", "welcome"]);
  handle.unmount();
  targets.cleanup();
});

test("restores focus to the launcher when the tour closes", async () => {
  const targets = createTargets();
  const handle = mountTour();
  await settle();
  targets.launcher.focus();
  handle.exposes<TourRootExpose<DemoStep>>().start();
  await settle();
  assert.equal(document.activeElement, requireContent());

  await handle.click(part("close"));
  await settle();
  assert.equal(content(), null);
  assert.equal(document.activeElement, targets.launcher);
  handle.unmount();
  targets.cleanup();
});

test("controlled open and step emit requests without changing until the parent accepts", async () => {
  const targets = createTargets();
  const handle = mountTour({ open: true, step: "search" });
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "search");

  await handle.click(part("next"));
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:step"), [["profile"]]);
  assert.equal(requireContent().getAttribute("data-step"), "search");

  await handle.wrapper.setProps({ step: "profile" });
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "profile");
  assert.equal(targets.profile.getAttribute("data-vize-tour-target"), "active");

  await handle.click(part("close"));
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);
  assert.ok(content());

  await handle.wrapper.setProps({ open: false });
  await settle();
  assert.equal(content(), null);
  handle.unmount();
  targets.cleanup();
});

test("missing targets center by default and skip in the navigation direction when configured", async () => {
  const targets = createTargets();
  const steps: readonly DemoStep[] = [
    { value: "welcome", title: "Welcome" },
    { value: "ghost", title: "Ghost", target: "#tour-missing" },
    { value: "broken", title: "Broken", target: "::not a selector" },
    { value: "profile", title: "Profile", target: "#tour-profile" },
  ];
  const centered = mountTour({ defaultOpen: true, steps });
  await settle();
  await centered.click(part("next"));
  await settle();
  const dialog = requireContent();
  assert.equal(dialog.getAttribute("data-step"), "ghost");
  assert.equal(dialog.getAttribute("data-target"), "missing");
  assert.equal(dialog.getAttribute("data-placement"), "center");
  assert.equal(centered.root().getAttribute("data-target"), "missing");
  centered.unmount();

  const skipping = mountTour({ defaultOpen: true, steps, missingTarget: "skip" });
  await settle();
  await skipping.click(part("next"));
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "profile");
  await skipping.click(part("prev"));
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "welcome");
  skipping.unmount();

  const override = mountTour({
    defaultOpen: true,
    defaultStep: "ghost",
    steps: steps.map((step) =>
      step.value === "ghost" ? { ...step, missingTarget: "skip" as const } : step,
    ),
  });
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "broken");
  assert.equal(requireContent().getAttribute("data-target"), "missing");
  override.unmount();
  targets.cleanup();
});

test("disabled steps leave the sequence and progress counts", async () => {
  const targets = createTargets();
  const steps: readonly DemoStep[] = [
    { value: "welcome", title: "Welcome" },
    { value: "search", title: "Search", target: "#tour-search", disabled: true },
    { value: "profile", title: "Profile", target: "#tour-profile" },
  ];
  const handle = mountTour({ defaultOpen: true, steps });
  await settle();
  assert.equal(part("progress").textContent, "1 / 2");
  await handle.click(part("next"));
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "profile");
  assert.equal(handle.exposes<TourRootExpose<DemoStep>>().goTo("search"), false);
  handle.unmount();
  targets.cleanup();
});

test("Escape dismisses with a reason and arrow keys navigate outside editable controls", async () => {
  const targets = createTargets();
  const handle = mountTour({ defaultOpen: true });
  await settle();
  const dialog = requireContent();

  const right = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowRight",
  });
  dialog.dispatchEvent(right);
  await settle();
  assert.equal(right.defaultPrevented, true);
  assert.equal(requireContent().getAttribute("data-step"), "search");

  const field = document.createElement("input");
  requireContent().append(field);
  const ignored = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowRight",
  });
  field.dispatchEvent(ignored);
  await settle();
  assert.equal(ignored.defaultPrevented, false);
  assert.equal(requireContent().getAttribute("data-step"), "search");
  field.remove();

  const left = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "ArrowLeft" });
  requireContent().dispatchEvent(left);
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "welcome");

  requireContent().dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
  await settle();
  assert.equal(content(), null);
  assert.equal(handle.wrapper.emitted("dismiss")?.[0]?.[0], "escape-key");
  handle.unmount();

  const rtl = mountTour({ defaultOpen: true, dir: "rtl" }, { closeOnEscape: false });
  await settle();
  requireContent().dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "ArrowLeft" }),
  );
  await settle();
  assert.equal(requireContent().getAttribute("data-step"), "search");
  requireContent().dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
  await settle();
  assert.ok(content());
  rtl.unmount();
  targets.cleanup();
});

test("close controls report their reason and preventDefault keeps the tour open", async () => {
  const targets = createTargets();
  const Probe = defineComponent({
    setup: () => () =>
      h(TourRoot, { steps: defaultSteps, defaultOpen: true, onDismiss: recordDismiss }, () =>
        h(TourContent, { portalDisabled: true }, () => [
          h(TourClose, { onClick: (event: MouseEvent) => event.preventDefault() }, () => "Stay"),
          h(TourClose, null, () => "Close"),
        ]),
      ),
  });
  const reasons: string[] = [];
  function recordDismiss(reason: string): void {
    reasons.push(reason);
  }
  const handle = mountInteraction(Probe);
  await settle();
  const [stay, close] = [
    ...document.querySelectorAll<HTMLButtonElement>("[data-vize-ui='tour-close']"),
  ];
  assert.ok(stay && close);
  assert.equal(close.getAttribute("data-reason"), "close");
  await handle.click(stay);
  await settle();
  assert.ok(content());
  assert.deepEqual(reasons, []);
  await handle.click(close);
  await settle();
  assert.equal(content(), null);
  assert.deepEqual(reasons, ["close"]);
  handle.unmount();
  targets.cleanup();
});

test("skip controls dismiss with the skip reason", async () => {
  const targets = createTargets();
  const handle = mountTour({ defaultOpen: true });
  await settle();
  await handle.click(part("close"));
  await settle();
  assert.equal(content(), null);
  assert.deepEqual(handle.wrapper.emitted("dismiss")?.[0]?.[0], "skip");
  assert.equal(handle.wrapper.emitted("complete"), undefined);
  handle.unmount();
  targets.cleanup();
});

test("spotlight publishes padded target geometry as custom properties", async () => {
  const targets = createTargets();
  targets.search.getBoundingClientRect = () => new DOMRect(40, 60, 200, 32);
  const handle = mountTour({ defaultOpen: true, defaultStep: "search" });
  await settle();
  const spotlight = part("spotlight");

  assert.equal(spotlight.hidden, false);
  assert.equal(spotlight.getAttribute("aria-hidden"), "true");
  assert.equal(spotlight.getAttribute("data-target"), "resolved");
  assert.equal(spotlight.getAttribute("data-interactive"), "false");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-x"), "34px");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-y"), "54px");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-width"), "212px");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-height"), "44px");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-spotlight-radius"), "8px");

  targets.search.getBoundingClientRect = () => new DOMRect(10, 20, 200, 32);
  globalThis.dispatchEvent(new Event("resize"));
  await settle();
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-x"), "4px");

  await handle.click(part("prev"));
  await settle();
  assert.equal(spotlight.getAttribute("data-target"), "none");
  assert.equal(spotlight.style.getPropertyValue("--vize-ui-tour-target-x"), "");
  handle.unmount();
  targets.cleanup();
});

test("exposes typed imperative controls and re-resolves late targets on refresh", async () => {
  const targets = createTargets();
  const handle = mountTour({
    steps: [...defaultSteps, { value: "late", title: "Late", target: "#tour-late" }],
  });
  await settle();
  const tour = handle.exposes<TourRootExpose<DemoStep>>();

  assert.equal(tour.id, "onboarding");
  assert.equal(tour.open, false);
  assert.equal(tour.next(), false);
  assert.equal(tour.setOpen(true), true);
  await settle();
  assert.equal(tour.value, "welcome");
  assert.equal(tour.goTo("late"), true);
  await settle();
  assert.equal(tour.targetState, "missing");
  assert.equal(tour.target, null);

  const late = document.createElement("div");
  late.id = "tour-late";
  document.body.append(late);
  tour.refresh();
  await settle();
  assert.equal(tour.targetState, "resolved");
  assert.equal(tour.target, late);
  assert.equal(tour.index, 3);
  assert.equal(tour.total, 4);
  assert.equal(tour.last, true);
  assert.equal(tour.complete(), true);
  await settle();
  assert.equal(tour.open, false);
  assert.equal(tour.dismiss(), false);
  late.remove();
  handle.unmount();
  targets.cleanup();
});

test("respects reduced motion when scrolling targets into view", async () => {
  const targets = createTargets();
  const originalMatchMedia = globalThis.matchMedia;
  globalThis.matchMedia = ((query: string) => ({
    matches: query.includes("reduce"),
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  })) satisfies typeof globalThis.matchMedia;
  try {
    const handle = mountTour({ defaultOpen: true, defaultStep: "profile" });
    await settle();
    assert.equal(targets.scrolls.at(-1)?.behavior, "auto");
    handle.unmount();
    const quiet = mountTour({
      defaultOpen: true,
      defaultStep: "search",
      scrollIntoView: false,
      markTarget: false,
    });
    await settle();
    assert.equal(targets.scrolls.length, 1);
    assert.equal(targets.search.hasAttribute("data-vize-tour-target"), false);
    quiet.unmount();
  } finally {
    globalThis.matchMedia = originalMatchMedia;
    targets.cleanup();
  }
});

interface Deferred<Value> {
  readonly promise: Promise<Value>;
  readonly resolve: (value: Value) => void;
  readonly reject: (reason: unknown) => void;
}

function deferred<Value>(): Deferred<Value> {
  let resolve: (value: Value) => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise<Value>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

test("async beforeEnter keeps navigation pending, disables controls, and re-resolves the target", async () => {
  const targets = createTargets();
  targets.search.remove();
  const gate = deferred<boolean>();
  const calls: TourBeforeEnterContext<DemoStep>[] = [];
  const handle = mountTour({
    defaultOpen: true,
    missingTarget: "skip",
    beforeEnter: (context: TourBeforeEnterContext<DemoStep>) => {
      calls.push(context);
      return context.step.value === "search" ? gate.promise : undefined;
    },
  });
  await settle();
  assert.equal(calls.length, 0, "initial uncontrolled open does not run hooks");

  await handle.click(part("next"));
  await settle();
  assert.equal(calls.at(-1)?.step.value, "search");
  assert.equal(calls.at(-1)?.from?.value, "welcome");
  assert.equal(calls.at(-1)?.direction, "forward");
  assert.equal(handle.root().getAttribute("data-pending"), "true");
  assert.equal(requireContent().getAttribute("data-step"), "welcome");
  assert.equal((part("next") as HTMLButtonElement).disabled, true);
  assert.equal(handle.exposes<TourRootExpose<DemoStep>>().pending, true);

  // The hook renders the target that a `skip` policy would otherwise pass over.
  document.body.append(targets.search);
  gate.resolve(true);
  await settle();
  assert.equal(handle.root().hasAttribute("data-pending"), false);
  assert.equal(requireContent().getAttribute("data-step"), "search");
  assert.equal(requireContent().getAttribute("data-target"), "resolved");
  assert.equal(targets.search.getAttribute("data-vize-tour-target"), "active");
  assert.equal((part("next") as HTMLButtonElement).disabled, false);
  handle.unmount();
  targets.cleanup();
});

test("beforeEnter false cancels and failures report navigation-error while keeping the step", async () => {
  const targets = createTargets();
  let outcome: "cancel" | "async-cancel" | "throw" | "reject" = "cancel";
  const failure = new Error("route failed");
  const handle = mountTour({
    defaultOpen: true,
    beforeEnter: () => {
      if (outcome === "cancel") return false;
      if (outcome === "async-cancel") return Promise.resolve(false);
      if (outcome === "throw") throw failure;
      return Promise.reject(failure);
    },
  });
  await settle();
  const tour = handle.exposes<TourRootExpose<DemoStep>>();

  assert.equal(tour.next(), false);
  await settle();
  assert.equal(tour.value, "welcome");
  outcome = "async-cancel";
  assert.equal(tour.next(), true, "asynchronous requests are accepted while pending");
  await settle();
  outcome = "throw";
  assert.equal(tour.goTo("profile"), false);
  outcome = "reject";
  tour.next();
  await settle();
  assert.equal(tour.value, "welcome");
  assert.equal(tour.pending, false);
  assert.deepEqual(handle.wrapper.emitted("navigation-cancel"), [
    ["search", "welcome"],
    ["search", "welcome"],
  ]);
  assert.deepEqual(handle.wrapper.emitted("navigation-error"), [
    [failure, "profile", "welcome"],
    [failure, "search", "welcome"],
  ]);
  assert.equal(handle.wrapper.emitted("update:step"), undefined);
  handle.unmount();
  targets.cleanup();
});

test("a newer navigation aborts the pending hook and the latest request wins", async () => {
  const targets = createTargets();
  const gates = new Map<string, Deferred<boolean>>();
  const signals: AbortSignal[] = [];
  const handle = mountTour({
    defaultOpen: true,
    steps: defaultSteps.map((step) => ({
      ...step,
      beforeEnter: ({ step: entering, signal }: TourBeforeEnterContext) => {
        signals.push(signal);
        const gate = deferred<boolean>();
        gates.set(entering.value, gate);
        return gate.promise;
      },
    })),
  });
  await settle();
  const tour = handle.exposes<TourRootExpose<DemoStep>>();

  tour.goTo("search");
  tour.goTo("profile");
  assert.equal(signals[0]?.aborted, true);
  assert.equal(signals[1]?.aborted, false);
  gates.get("profile")?.resolve(true);
  await settle();
  gates.get("search")?.resolve(true);
  await settle();
  assert.equal(tour.value, "profile");
  assert.deepEqual(handle.wrapper.emitted("update:step"), [["profile"]]);

  tour.previous();
  assert.equal(tour.pending, true);
  tour.dismiss();
  assert.equal(signals.at(-1)?.aborted, true, "dismissal aborts the pending hook");
  gates.get("search")?.resolve(true);
  await settle();
  assert.equal(tour.open, false);
  assert.equal(tour.value, "profile");
  assert.equal(tour.pending, false);
  handle.unmount();
  targets.cleanup();
});

test("afterLeave runs after steps change and when the tour closes", async () => {
  const targets = createTargets();
  const left: string[] = [];
  const handle = mountTour({
    defaultOpen: true,
    afterLeave: ({ step, to }: TourAfterLeaveContext<DemoStep>) =>
      left.push(`${step.value}->${to?.value ?? "closed"}`),
  });
  await settle();
  const tour = handle.exposes<TourRootExpose<DemoStep>>();
  tour.next();
  tour.next();
  tour.dismiss();
  assert.deepEqual(left, ["welcome->search", "search->profile", "profile->closed"]);
  handle.unmount();
  targets.cleanup();
});

test("messages override progress and fallback control labels", async () => {
  const targets = createTargets();
  const handle = mountInteraction(TourRoot, {
    props: {
      steps: defaultSteps,
      defaultOpen: true,
      messages: {
        progress: (current: number, total: number) => `Schritt ${current} von ${total}`,
        previous: "Zurück",
        next: "Weiter",
        finish: "Fertig",
        skip: "Überspringen",
      },
    },
    slots: {
      default: () =>
        h(TourContent, { portalDisabled: true, ariaLabel: "Tour" }, () => [
          h(TourProgress),
          h(TourPrev),
          h(TourNext),
          h(TourClose, { reason: "skip" }),
          h(TourClose),
        ]),
    },
  });
  await settle();
  assert.equal(part("progress").textContent, "Schritt 1 von 3");
  assert.equal(part("prev").textContent, "Zurück");
  assert.equal(part("next").textContent, "Weiter");
  const closes = document.querySelectorAll('[data-vize-ui="tour-close"]');
  assert.equal(closes[0]?.textContent, "Überspringen");
  assert.equal(closes[1]?.textContent, "", "unset labels stay empty");
  handle.exposes<TourRootExpose<DemoStep>>().goTo("profile");
  await settle();
  assert.equal(part("next").textContent, "Fertig");
  handle.unmount();
  targets.cleanup();
});

test("compound parts require a matching root provider", () => {
  for (const component of [
    TourContent,
    TourTitle,
    TourDescription,
    TourStep,
    TourProgress,
    TourPrev,
    TourNext,
    TourClose,
    TourSpotlight,
  ]) {
    const originalWarn = console.warn;
    console.warn = () => undefined;
    try {
      assert.throws(
        () => mountInteraction(component, { props: { value: "x" } }),
        /VIZE_UI_CONTEXT_MISSING: Tour requires a matching provider/,
      );
    } finally {
      console.warn = originalWarn;
    }
  }
});
