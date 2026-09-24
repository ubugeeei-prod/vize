import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { afterEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import MasterDetail from "./master-detail.vue";

function setViewport(width: number): void {
  window.happyDOM.setViewport({ width, height: 800 });
  window.dispatchEvent(new Event("resize"));
}

afterEach(() => setViewport(1024));

const slots = {
  master: ({ select, selected }: { select: (key: string) => void; selected: string | null }) =>
    ["inbox", "sent"].map((key) =>
      h(
        "button",
        {
          type: "button",
          "data-key": key,
          "aria-current": selected === key ? "true" : undefined,
          onClick: () => select(key),
        },
        key,
      ),
    ),
  detail: ({ selected, back }: { selected: string; back: () => void }) => [
    h("h2", selected),
    h("button", { type: "button", "data-back": "", onClick: back }, "Back"),
  ],
  empty: () => "Pick a mailbox",
};

test("split layout shows both panes and an empty placeholder", async () => {
  setViewport(1200);
  const wrapper = mount(MasterDetail<string>, {
    props: { masterLabel: "Mailboxes", detailLabel: "Messages" },
    slots,
  });
  await nextTick();
  const root = wrapper.get('[data-vize-ui="master-detail"]');
  assert.equal(root.attributes("data-layout"), "split");
  assert.match(root.attributes("style") ?? "", /grid-template-columns: minmax\(16rem, 1fr\) 2fr/);
  assert.equal(wrapper.get('[data-part="empty"]').text(), "Pick a mailbox");
  assert.equal(wrapper.get('[data-part="master"]').attributes("aria-label"), "Mailboxes");

  await wrapper.get('[data-key="sent"]').trigger("click");
  assert.equal(wrapper.get('[data-part="detail"] h2').text(), "sent");
  assert.ok(wrapper.find('[data-part="master"]').exists());
  assert.deepEqual(wrapper.emitted("update:selected"), [["sent"]]);
  wrapper.unmount();
});

test("stacked layout swaps panes, moves focus, and returns with Back or Escape", async () => {
  setViewport(500);
  const wrapper = mount(MasterDetail<string>, { slots, attachTo: document.body });
  await nextTick();
  assert.equal(wrapper.get('[data-vize-ui="master-detail"]').attributes("data-layout"), "stacked");
  assert.equal(wrapper.find('[data-part="detail"]').exists(), false);
  assert.equal(wrapper.find('[data-part="empty"]').exists(), false);

  await wrapper.get('[data-key="inbox"]').trigger("click");
  await nextTick();
  assert.equal(wrapper.find('[data-part="master"]').exists(), false);
  const detail = wrapper.get('[data-part="detail"]');
  assert.equal(document.activeElement, detail.element);

  await detail.trigger("keydown", { key: "Escape" });
  await nextTick();
  assert.equal(wrapper.find('[data-part="detail"]').exists(), false);
  assert.equal(document.activeElement, wrapper.get('[data-part="master"]').element);

  await wrapper.get('[data-key="sent"]').trigger("click");
  await wrapper.get("[data-back]").trigger("click");
  await nextTick();
  assert.ok(wrapper.find('[data-part="master"]').exists());
  assert.deepEqual(wrapper.emitted("update:selected"), [["inbox"], [null], ["sent"], [null]]);
  wrapper.unmount();
});

test("controlled selection and layout changes are reported", async () => {
  setViewport(1200);
  const wrapper = mount(MasterDetail<string>, {
    props: { selected: "inbox", splitAt: 900 },
    slots,
  });
  await nextTick();
  assert.equal(wrapper.get('[data-part="detail"] h2').text(), "inbox");
  await wrapper.get('[data-key="sent"]').trigger("click");
  assert.equal(wrapper.get('[data-part="detail"] h2').text(), "inbox");
  await wrapper.setProps({ selected: "sent" });
  assert.equal(wrapper.get('[data-part="detail"] h2').text(), "sent");

  setViewport(800);
  await nextTick();
  assert.deepEqual(wrapper.emitted("layoutChange"), [["split"], ["stacked"]]);
  assert.equal(wrapper.find('[data-part="master"]').exists(), false);
  wrapper.unmount();
});
