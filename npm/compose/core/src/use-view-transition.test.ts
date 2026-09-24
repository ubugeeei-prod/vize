import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useViewTransition } from "./use-view-transition.ts";
import type {
  StartViewTransitionArgument,
  ViewTransitionDocumentHost,
  ViewTransitionLike,
} from "./use-view-transition.ts";

class FakeTransition implements ViewTransitionLike {
  readonly updateCallbackDone: Promise<void>;
  readonly ready: Promise<void>;
  readonly finished: Promise<void>;
  skipped = false;
  finish: () => void = () => undefined;

  constructor(update: (() => unknown) | undefined) {
    this.updateCallbackDone = Promise.resolve().then(async () => {
      await update?.();
    });
    this.ready = this.updateCallbackDone.then(() => {
      if (this.skipped) throw new Error("AbortError");
    });
    this.finished = new Promise<void>((resolve) => {
      this.finish = resolve;
    });
  }

  skipTransition(): void {
    this.skipped = true;
    this.finish();
  }
}

function createDocument(rejectTypes = false): {
  document: ViewTransitionDocumentHost;
  calls: StartViewTransitionArgument[];
  transitions: FakeTransition[];
} {
  const calls: StartViewTransitionArgument[] = [];
  const transitions: FakeTransition[] = [];
  return {
    calls,
    transitions,
    document: {
      startViewTransition: (argument) => {
        if (argument === undefined) throw new TypeError("missing");
        if (typeof argument !== "function" && rejectTypes) throw new TypeError("not callable");
        calls.push(argument);
        const transition = new FakeTransition(
          typeof argument === "function" ? argument : argument.update,
        );
        transitions.push(transition);
        return transition;
      },
    },
  };
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 6; index += 1) await Promise.resolve();
}

void test("runs the update inside a transition and resolves its result", async () => {
  const { document, calls, transitions } = createDocument();
  const view = useViewTransition({ document, reducedMotion: false });
  let updated = false;
  const result = view.start(async () => {
    updated = true;
    return 42;
  });

  assert.equal(view.supported.value, true);
  assert.equal(view.isTransitioning.value, true);
  assert.equal(view.transition.value, transitions[0]);
  assert.equal(typeof calls[0], "function");
  assert.equal(await result, 42);
  assert.equal(updated, true);

  transitions[0]?.finish();
  await flushPromises();
  assert.equal(view.isTransitioning.value, false);
  assert.equal(view.transition.value, null);
});

void test("passes types with the options form and falls back without support", async () => {
  const typed = createDocument();
  const view = useViewTransition({ document: typed.document, reducedMotion: false });
  assert.equal(await view.start(() => "a", { types: ["forward"] }), "a");
  const [argument] = typed.calls;
  assert.ok(argument && typeof argument !== "function");
  assert.deepEqual(argument.types, ["forward"]);

  const legacy = createDocument(true);
  const fallback = useViewTransition({ document: legacy.document, reducedMotion: false });
  assert.equal(await fallback.start(() => "b", { types: ["forward"] }), "b");
  assert.equal(typeof legacy.calls[0], "function");
});

void test("runs the update directly when unsupported or reduced motion is preferred", async () => {
  const { document, calls } = createDocument();
  const reduced = ref(true);
  const view = useViewTransition({ document, reducedMotion: reduced });
  assert.equal(await view.start(() => 1), 1);
  assert.equal(calls.length, 0);
  assert.equal(view.transition.value, null);

  const ignoring = useViewTransition({
    document,
    reducedMotion: reduced,
    respectReducedMotion: false,
  });
  assert.equal(await ignoring.start(() => 2), 2);
  assert.equal(calls.length, 1);

  const missing = useViewTransition({ document: null });
  assert.equal(missing.supported.value, false);
  assert.equal(await missing.start(() => 3), 3);
});

void test("rejects with the update error", async () => {
  const { document } = createDocument();
  const view = useViewTransition({ document, reducedMotion: false });
  await assert.rejects(
    view.start(() => {
      throw new Error("update failed");
    }),
    /update failed/,
  );
});

void test("a newer transition keeps the state until it finishes", async () => {
  const { document, transitions } = createDocument();
  const view = useViewTransition({ document, reducedMotion: false });
  await view.start(() => 1);
  await view.start(() => 2);
  transitions[0]?.finish();
  await flushPromises();
  assert.equal(view.transition.value, transitions[1]);
  assert.equal(view.isTransitioning.value, true);
});

void test("skip and scope disposal skip the running transition", async () => {
  const { document, transitions } = createDocument();
  const view = useViewTransition({ document, reducedMotion: false });
  await view.start(() => undefined);
  view.skip();
  assert.equal(transitions[0]?.skipped, true);
  await flushPromises();
  assert.equal(view.isTransitioning.value, false);

  const scope = effectScope();
  const scoped = scope.run(() => useViewTransition({ document, reducedMotion: false }));
  assert.ok(scoped);
  await scoped.start(() => undefined);
  scope.stop();
  assert.equal(transitions[1]?.skipped, true);
});

void test("server rendering runs updates directly", async () => {
  const state = await renderComposableOnServer(() => {
    const view = useViewTransition();
    return {
      supported: view.supported,
      isTransitioning: view.isTransitioning,
      transition: view.transition,
    };
  });
  assert.equal(state, '{"supported":false,"isTransitioning":false,"transition":null}');
});
