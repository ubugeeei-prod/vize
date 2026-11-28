import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import DescriptionsItem from "../descriptions-item.vue";
import Descriptions from "../descriptions.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountDescriptions() {
  return mountInteraction(Descriptions, {
    props: { label: "Invoice" },
    slots: {
      default: () => [
        h(DescriptionsItem, { label: "Status" }, () => "Paid"),
        h(DescriptionsItem, { label: "Total" }, () => "$1,280"),
      ],
    },
  });
}

test("names the list and pairs each term with its detail", () => {
  const handle = mountDescriptions();
  const list = handle.getByRole("group", { name: "Invoice" });

  assert.equal(list.tagName, "DL");
  assert.equal(list.getAttribute("data-vize-ui"), "descriptions");
  assert.equal(handle.getByRole("term", { name: "Status" }).textContent, "Status");
  assert.equal(handle.getByRole("definition", { name: "Paid" }).textContent, "Paid");
  assert.equal(handle.getByRole("term", { name: "Total" }).textContent, "Total");
  assert.equal(handle.getByRole("definition", { name: "$1,280" }).textContent, "$1,280");
  handle.unmount();
});

test("passes the term to the detail slot", () => {
  const handle = mountInteraction(Descriptions, {
    props: { label: "Invoice" },
    slots: {
      default: () =>
        h(
          DescriptionsItem,
          { label: "Status" },
          {
            default: (props: { label: string }) =>
              h("span", { "data-slot-label": props.label }, "Paid"),
          },
        ),
    },
  });

  assert.equal(
    handle.root().querySelector("[data-slot-label]")?.getAttribute("data-slot-label"),
    "Status",
  );
  handle.unmount();
});
