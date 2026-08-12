import assert from "node:assert/strict";
import { test } from "node:test";

import { effectScope } from "vue";

import { createDisposalScope, DISPOSAL_ERROR_CODE, DisposalError } from "./disposal-scope.ts";

void test("runs cleanups once in last-in, first-out order", () => {
  const owner = createDisposalScope({ scope: false });
  const calls: number[] = [];
  owner.add(() => calls.push(1));
  owner.add(() => calls.push(2));
  owner.add(() => calls.push(3));

  assert.equal(owner.size, 3);
  assert.equal(owner.disposed, false);
  owner.dispose();
  owner.dispose();

  assert.deepEqual(calls, [3, 2, 1]);
  assert.equal(owner.size, 0);
  assert.equal(owner.disposed, true);
});

void test("unregisters a cleanup without running it", () => {
  const owner = createDisposalScope({ scope: false });
  let calls = 0;
  const registration = owner.add(() => {
    calls += 1;
  });

  assert.equal(registration.active, true);
  assert.equal(registration.unregister(), true);
  assert.equal(registration.active, false);
  assert.equal(registration.unregister(), false);
  assert.equal(owner.size, 0);
  owner.dispose();
  assert.equal(calls, 0);
});

void test("attempts every cleanup and reports original failures in execution order", () => {
  const owner = createDisposalScope({ scope: false });
  const first = new Error("first");
  const second = { code: "second" };
  let successfulCleanup = false;
  owner.add(() => {
    throw first;
  });
  owner.add(() => {
    successfulCleanup = true;
  });
  owner.add(() => {
    throw second;
  });

  assert.throws(
    () => owner.dispose(),
    (error: unknown) => {
      assert.ok(error instanceof DisposalError);
      assert.equal(error.code, DISPOSAL_ERROR_CODE);
      assert.equal(error.name, "DisposalError");
      assert.deepEqual(error.errors, [second, first]);
      assert.match(error.message, /2 cleanup operation\(s\) failed/);
      return true;
    },
  );
  assert.equal(successfulCleanup, true);
  assert.equal(owner.size, 0);
  assert.doesNotThrow(() => owner.dispose());
});

void test("runs late registrations immediately and returns an inactive handle", () => {
  const owner = createDisposalScope({ scope: false });
  let calls = 0;
  owner.dispose();

  const registration = owner.add(() => {
    calls += 1;
  });

  assert.equal(calls, 1);
  assert.equal(registration.active, false);
  assert.equal(registration.unregister(), false);
  assert.equal(owner.size, 0);
});

void test("wraps late and reentrant failures without nesting disposal errors", () => {
  const owner = createDisposalScope({ scope: false });
  const lateFailure = new Error("late");
  owner.dispose();

  assert.throws(
    () =>
      owner.add(() => {
        throw new DisposalError([lateFailure]);
      }),
    (error: unknown) => {
      assert.ok(error instanceof DisposalError);
      assert.deepEqual(error.errors, [lateFailure]);
      return true;
    },
  );

  const reentrantOwner = createDisposalScope({ scope: false });
  const reentrantFailure = new Error("reentrant");
  reentrantOwner.add(() => {
    reentrantOwner.add(() => {
      throw reentrantFailure;
    });
  });
  assert.throws(
    () => reentrantOwner.dispose(),
    (error: unknown) => {
      assert.ok(error instanceof DisposalError);
      assert.deepEqual(error.errors, [reentrantFailure]);
      return true;
    },
  );
});

void test("disposes children with the parent and releases children disposed early", () => {
  const parent = createDisposalScope({ scope: false });
  const early = parent.child();
  const owned = parent.child();
  let earlyCalls = 0;
  let ownedCalls = 0;
  early.add(() => {
    earlyCalls += 1;
  });
  owned.add(() => {
    ownedCalls += 1;
  });

  assert.equal(parent.size, 2);
  early.dispose();
  assert.equal(parent.size, 1);
  assert.equal(earlyCalls, 1);
  parent.dispose();

  assert.equal(owned.disposed, true);
  assert.equal(ownedCalls, 1);
  assert.equal(earlyCalls, 1);
});

void test("flattens child cleanup failures into the parent disposal error", () => {
  const parent = createDisposalScope({ scope: false });
  const child = parent.child();
  const childFailure = new Error("child cleanup");
  child.add(() => {
    throw childFailure;
  });

  assert.throws(
    () => parent.dispose(),
    (error: unknown) => {
      assert.ok(error instanceof DisposalError);
      assert.deepEqual(error.errors, [childFailure]);
      return true;
    },
  );
  assert.equal(parent.disposed, true);
  assert.equal(child.disposed, true);
});

void test("creates an already-disposed child after its parent is disposed", () => {
  const parent = createDisposalScope({ scope: false });
  parent.dispose();

  const child = parent.child();
  let calls = 0;
  assert.equal(child.disposed, true);
  child.add(() => {
    calls += 1;
  });
  assert.equal(calls, 1);
});

void test("joins an active reactive scope by default", () => {
  const reactiveOwner = effectScope();
  let owner: ReturnType<typeof createDisposalScope> | undefined;
  let calls = 0;
  reactiveOwner.run(() => {
    owner = createDisposalScope();
    owner.add(() => {
      calls += 1;
    });
  });

  reactiveOwner.stop();
  assert.equal(owner?.disposed, true);
  assert.equal(calls, 1);
});

void test("keeps default ownership with the caller outside a reactive scope", () => {
  const owner = createDisposalScope();
  let calls = 0;
  owner.add(() => {
    calls += 1;
  });

  assert.equal(owner.disposed, false);
  assert.equal(calls, 0);
  owner.dispose();
  assert.equal(calls, 1);
});

void test("keeps ownership with the caller when reactive-scope joining is disabled", () => {
  const reactiveOwner = effectScope();
  let owner: ReturnType<typeof createDisposalScope> | undefined;
  let calls = 0;
  reactiveOwner.run(() => {
    owner = createDisposalScope({ scope: false });
    owner.add(() => {
      calls += 1;
    });
  });

  reactiveOwner.stop();
  assert.equal(owner?.disposed, false);
  assert.equal(calls, 0);
  owner?.dispose();
  assert.equal(calls, 1);
});
