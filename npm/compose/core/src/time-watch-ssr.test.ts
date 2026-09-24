/**
 * Server-rendering contract for the timing and watch helpers: rendering the
 * same component twice produces identical markup, the markup matches what a
 * hydrating client computes from the same injected inputs, and no host timer
 * or frame is ever requested on the server.
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h, shallowRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { FakeClock, FakeFrames } from "./testing/fake-clock.ts";
import { until } from "./until.ts";
import { useCountdown } from "./use-countdown.ts";
import { formatDate, useDateFormat } from "./use-date-format.ts";
import { useInterval, useIntervalFn } from "./use-interval.ts";
import { useNow, useTimestamp } from "./use-now.ts";
import { useRafFn } from "./use-raf-fn.ts";
import { formatTimeAgo, useTimeAgo } from "./use-time-ago.ts";
import { useTimeout, useTimeoutFn } from "./use-timeout.ts";
import { watchDebounced } from "./watch-debounced.ts";
import { watchIgnorable } from "./watch-ignorable.ts";
import { watchOnce } from "./watch-once.ts";
import { watchPausable } from "./watch-pausable.ts";
import { watchThrottled } from "./watch-throttled.ts";
import { whenever } from "./whenever.ts";

const renderedAt = Date.UTC(2026, 9, 24, 9, 30);

async function renderTwice(setup: (clock: FakeClock, frames: FakeFrames) => () => unknown) {
  const outputs: string[] = [];
  for (let run = 0; run < 2; run += 1) {
    const clock = new FakeClock();
    const frames = new FakeFrames();
    const app = createSSRApp(
      defineComponent({
        setup: () => {
          const render = setup(clock, frames);
          return () => h("output", String(render()));
        },
      }),
    );
    outputs.push(await renderToString(app));
    assert.equal(clock.size, 0, "no timer may start during server rendering");
    assert.equal(frames.pending.size, 0, "no frame may be requested during server rendering");
  }
  assert.equal(outputs[0], outputs[1], "server output must be deterministic");
  return outputs[0];
}

void test("interval and timeout report the client's initial state without timers", async () => {
  const html = await renderTwice((clock) => {
    const interval = useIntervalFn(() => undefined, 100, { scheduler: clock.interval });
    const counter = useInterval(100, { scheduler: clock.interval });
    const pending = useTimeoutFn(() => undefined, 100, { scheduler: clock.timeout });
    const ready = useTimeout(100, { scheduler: clock.timeout });
    return () =>
      [interval.isActive.value, counter.value, pending.isPending.value, ready.value].join("|");
  });
  assert.equal(html, "<output>true|0|true|false</output>");
});

void test("frame loops only track their requested state", async () => {
  const html = await renderTwice((_clock, frames) => {
    const loop = useRafFn(() => undefined, { scheduler: frames });
    return () => loop.isActive.value;
  });
  assert.equal(html, "<output>true</output>");
});

void test("clocks render the injected initial instant", async () => {
  const html = await renderTwice((clock) => {
    const now = useNow({ initial: renderedAt, scheduler: clock.interval });
    const timestamp = useTimestamp({ initial: renderedAt, scheduler: clock.interval });
    return () => `${now.value.toISOString()}|${timestamp.value}`;
  });
  assert.equal(html, `<output>2026-10-24T09:30:00.000Z|${renderedAt}</output>`);
});

void test("countdowns keep their initial count", async () => {
  const html = await renderTwice((clock) => {
    const { remaining } = useCountdown(60, { immediate: true, scheduler: clock.interval });
    return () => remaining.value;
  });
  assert.equal(html, "<output>60</output>");
});

void test("relative and absolute dates match the client's formatting", async () => {
  const createdAt = renderedAt - 3 * 3_600_000;
  const html = await renderTwice((clock) => {
    const ago = useTimeAgo(createdAt, { initialNow: renderedAt, scheduler: clock.interval });
    const label = useDateFormat(createdAt, "YYYY-MM-DD HH:mm", { timeZone: "Asia/Tokyo" });
    return () => `${ago.value}|${label.value}`;
  });
  const client = `${formatTimeAgo(createdAt, renderedAt)}|${formatDate(createdAt, "YYYY-MM-DD HH:mm", { timeZone: "Asia/Tokyo" })}`;
  assert.equal(html, `<output>${client}</output>`);
  assert.equal(client, "3 hours ago|2026-10-24 15:30");
});

void test("watch helpers are inert or synchronous during setup and never schedule", async () => {
  const html = await renderTwice((clock) => {
    const source = shallowRef(0);
    const log: string[] = [];
    watchDebounced(source, (value) => log.push(`debounced:${value}`), {
      debounce: 100,
      flush: "sync",
      scheduler: clock.timeout,
    });
    watchThrottled(source, (value) => log.push(`throttled:${value}`), {
      throttle: 100,
      flush: "sync",
      scheduler: clock.timeout,
    });
    watchPausable(source, (value) => log.push(`pausable:${value}`), { flush: "sync" });
    watchIgnorable(source, (value) => log.push(`ignorable:${value}`), { flush: "sync" });
    watchOnce(source, (value) => log.push(`once:${value}`), { flush: "sync" });
    whenever(source, (value) => log.push(`whenever:${value}`), { flush: "sync" });
    source.value = 1;
    return () => log.join(",");
  });
  assert.equal(
    html,
    "<output>debounced:1,throttled:1,pausable:1,ignorable:1,once:1,whenever:1</output>",
  );
});

void test("until resolves inside async setup when the value is already there", async () => {
  const app = createSSRApp(
    defineComponent({
      async setup() {
        const status = shallowRef("ready");
        const value = await until(status).toBe("ready");
        return () => h("output", value);
      },
    }),
  );
  assert.equal(await renderToString(app), "<output>ready</output>");
});
