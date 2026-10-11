import assert from "node:assert/strict";
import test from "node:test";
import { waitForFirstCurrentVariant } from "./literal-variant-navigation.ts";

// This timer-free contract control is separate from the real native browser tests.
class ControlledNavigation {
  readonly names = ["__proto__", "constructor", "hasOwnProperty"];
  readonly waitArguments: unknown[][] = [];
  private selected = 2;
  private pending = new Set<{
    matches: () => number[];
    resolve: () => void;
    reject: (error: Error) => void;
  }>();

  readonly items = new ControlledLocator(this, () => this.names.map((_, index) => index));
  readonly current = new ControlledLocator(this, () => [this.selected]);

  get selectedName() {
    return this.names[this.selected];
  }

  select(index: number) {
    this.selected = index;
    for (const waiter of this.pending) {
      if (waiter.matches().length) {
        this.pending.delete(waiter);
        waiter.resolve();
      }
    }
  }

  wait(matches: () => number[], args: unknown[]) {
    this.waitArguments.push(args);
    if (matches().length) return Promise.resolve();
    return new Promise<void>((resolve, reject) => {
      this.pending.add({ matches, resolve, reject });
    });
  }

  refuse(error: Error) {
    for (const waiter of this.pending) waiter.reject(error);
    this.pending.clear();
  }
}

class ControlledLocator {
  constructor(
    private readonly state: ControlledNavigation,
    private readonly matches: () => number[],
  ) {}

  first() {
    return new ControlledLocator(this.state, () => this.matches().slice(0, 1));
  }

  and(other: ControlledLocator) {
    assert.equal(other.state, this.state);
    return new ControlledLocator(this.state, () =>
      this.matches().filter((index) => other.matches().includes(index)),
    );
  }

  waitFor(...args: unknown[]) {
    return this.state.wait(this.matches, args);
  }
}

void test("ANY-current can finish while FIRST-target still waits for its own current state", async () => {
  const state = new ControlledNavigation();
  let firstFinished = false;
  await state.current.first().waitFor();
  assert.equal(state.selectedName, "hasOwnProperty");
  const first = waitForFirstCurrentVariant(state.items, state.current).then(() => {
    firstFinished = true;
  });
  await Promise.resolve();
  assert.equal(firstFinished, false);
  state.select(1);
  await Promise.resolve();
  assert.equal(state.selectedName, "constructor");
  assert.equal(firstFinished, false);
  state.select(0);
  await first;
  assert.equal(state.selectedName, "__proto__");
  assert.equal(firstFinished, true);
  assert.deepEqual(state.waitArguments, [[], []]);
});

void test("FIRST-target preserves an upstream wait error", async () => {
  const state = new ControlledNavigation();
  const waiting = waitForFirstCurrentVariant(state.items, state.current);
  const error = new Error("upstream selector failure");
  state.refuse(error);
  await assert.rejects(waiting, (actual) => actual === error);
  assert.deepEqual(state.waitArguments, [[]]);
});

void test("FIRST-target preserves deadline refusal without setting a new timeout", async () => {
  const state = new ControlledNavigation();
  const waiting = waitForFirstCurrentVariant(state.items, state.current);
  const error = new Error("upstream default deadline exceeded");
  state.refuse(error);
  await assert.rejects(waiting, (actual) => actual === error);
  assert.deepEqual(state.waitArguments, [[]]);
});
