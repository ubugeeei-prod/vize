import assert from "node:assert/strict";
import { test } from "node:test";

import { useStepper } from "./use-stepper.ts";

void test("navigates named steps", () => {
  const wizard = useStepper(["account", "billing", "review"]);
  assert.equal(wizard.current.value, "account");
  assert.equal(wizard.isFirst.value, true);
  assert.equal(wizard.previous.value, undefined);
  wizard.goToNext();
  assert.equal(wizard.current.value, "billing");
  assert.equal(wizard.next.value, "review");
  assert.equal(wizard.isNext("review"), true);
  assert.equal(wizard.isPrevious("account"), true);
  assert.equal(wizard.isBefore("account"), true);
  assert.equal(wizard.isAfter("review"), true);
  wizard.goTo("review");
  wizard.goToNext();
  assert.equal(wizard.isLast.value, true);
  assert.equal(wizard.goBackTo("review"), false);
  assert.equal(wizard.goBackTo("account"), true);
  wizard.goToPrevious();
  assert.equal(wizard.index.value, 0);
  assert.equal(wizard.at(1), "billing");
  assert.equal(wizard.at(5), undefined);
});

void test("object steps expose per-step values and honour the initial step", () => {
  const checkout = useStepper(
    {
      cart: { title: "Cart" },
      shipping: { title: "Shipping" },
      payment: { title: "Payment" },
    },
    "shipping",
  );
  assert.deepEqual(checkout.stepNames, ["cart", "shipping", "payment"]);
  assert.equal(checkout.currentValue.value.title, "Shipping");
  assert.equal(checkout.get("payment").title, "Payment");
  assert.equal(checkout.isCurrent("shipping"), true);
});

void test("rejects empty and unknown steps", () => {
  assert.throws(() => useStepper([]), /VIZE_COMPOSE_STEPPER_EMPTY/);
  const wizard = useStepper(["a", "b"]);
  assert.throws(() => wizard.goTo(String("c") as "a"), /VIZE_COMPOSE_STEPPER_UNKNOWN_STEP/);
});
