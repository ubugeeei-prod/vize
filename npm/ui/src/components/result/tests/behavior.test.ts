import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import Result from "../result.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("announces an error as an alert", () => {
  const handle = mountInteraction(Result, {
    props: { status: "error", title: "Payment failed", description: "The card was declined." },
  });
  const alert = handle.getByRole("alert", { name: "Payment failed" });

  assert.equal(alert.getAttribute("data-vize-ui"), "result");
  assert.equal(alert.getAttribute("data-status"), "error");
  assert.equal(
    alert.querySelector("[data-part='description']")?.textContent,
    "The card was declined.",
  );
  assert.equal(handle.queryByRole("status"), null);
  handle.unmount();
});

test("announces success as status", () => {
  const handle = mountInteraction(Result, {
    props: { status: "success", title: "Payment received" },
  });

  assert.equal(
    handle.getByRole("status", { name: "Payment received" }).getAttribute("data-status"),
    "success",
  );
  assert.equal(handle.queryByRole("alert"), null);
  handle.unmount();
});

test("renders not-found and empty as status, not alerts", () => {
  for (const status of ["not-found", "empty"] as const) {
    const handle = mountInteraction(Result, { props: { status, title: status } });
    assert.equal(handle.getByRole("status", { name: status }).getAttribute("data-status"), status);
    assert.equal(handle.queryByRole("alert"), null);
    handle.unmount();
  }
});

test("omits an empty description", () => {
  const handle = mountInteraction(Result, { props: { title: "Saved" } });
  const status = handle.getByRole("status", { name: "Saved" });

  assert.equal(status.getAttribute("data-status"), "info");
  assert.equal(status.querySelector("[data-part='description']"), null);
  handle.unmount();
});

test("passes the status to the actions slot", () => {
  const handle = mountInteraction(Result, {
    props: { status: "warning", title: "Review" },
    slots: {
      default: (props: { status: string }) => h("span", { "data-slot-status": props.status }, "Go"),
    },
  });

  assert.equal(
    handle.root().querySelector("[data-slot-status]")?.getAttribute("data-slot-status"),
    "warning",
  );
  handle.unmount();
});
