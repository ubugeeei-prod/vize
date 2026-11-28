import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { formatStatistic } from "../statistic-format.ts";
import Statistic from "../statistic.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("formats a prefixed value and names the group from its title", () => {
  const handle = mountInteraction(Statistic, {
    props: { title: "Revenue", value: 1280, prefix: "$", suffix: "/mo" },
  });
  const group = handle.getByRole("group", { name: "Revenue" });

  assert.equal(group.getAttribute("data-vize-ui"), "statistic");
  assert.equal(group.getAttribute("data-notation"), "standard");
  assert.equal(group.querySelector("[data-part='value']")?.textContent, "$1,280/mo");
  handle.unmount();
});

test("puts the minus sign before the prefix", () => {
  assert.equal(
    formatStatistic({
      value: -1280,
      locale: "en-US",
      maximumFractionDigits: 0,
      notation: "standard",
      prefix: "$",
      suffix: "",
    }),
    "-$1,280",
  );
  const handle = mountInteraction(Statistic, {
    props: { title: "Refunds", value: -1280, prefix: "$" },
  });
  assert.equal(
    handle.getByRole("group", { name: "Refunds" }).querySelector("[data-part='value']")
      ?.textContent,
    "-$1,280",
  );
  handle.unmount();
});

test("formats compact notation", () => {
  assert.equal(
    formatStatistic({
      value: 1200,
      locale: "en-US",
      maximumFractionDigits: 1,
      notation: "compact",
      prefix: "",
      suffix: "",
    }),
    "1.2K",
  );
  const handle = mountInteraction(Statistic, {
    props: { title: "Requests", value: 1200, notation: "compact", maximumFractionDigits: 1 },
  });
  assert.equal(handle.getByRole("group").querySelector("[data-part='value']")?.textContent, "1.2K");
  handle.unmount();
});

test("defaults locale, fraction digits, and notation", () => {
  const handle = mountInteraction(Statistic, { props: { title: "Seats", value: 1280 } });
  const group = handle.getByRole("group", { name: "Seats" });

  assert.equal(group.getAttribute("data-notation"), "standard");
  assert.equal(group.querySelector("[data-part='value']")?.textContent, "1,280");
  handle.unmount();
});
