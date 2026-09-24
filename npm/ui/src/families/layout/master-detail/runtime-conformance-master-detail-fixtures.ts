import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { MasterDetail } from "./master-detail.ts";
import type { MasterDetailDetailSlotState } from "./master-detail-types.ts";

// Instantiation expressions pin the generic SFC so `h()` checks props against real key types.
const MailMasterDetail = MasterDetail<string>;

export const masterDetailRuntimeFixture: RuntimeFixture = {
  name: "master-detail",
  sourceFile: "families/layout/master-detail/master-detail.vue",
  render: () =>
    h(
      MailMasterDetail,
      {
        ssrWidth: 1024,
        defaultSelected: "inbox",
        masterLabel: "Mailboxes",
        detailLabel: "Messages",
      },
      {
        master: () => h("ul", [h("li", "Inbox")]),
        detail: ({ selected }: MasterDetailDetailSlotState<string>) => h("h2", selected),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /data-vize-ui="master-detail"/);
    assert.match(html, /data-layout="split"/);
    assert.match(
      html,
      /<section[^>]*data-part="master"[^>]*tabindex="-1"[^>]*aria-label="Mailboxes"/,
    );
    assert.match(html, /aria-label="Messages"[^>]*>[^]*<h2>inbox<\/h2>/);
  },
  assertHydratedDom(host) {
    const master = host.querySelector('[data-part="master"]');
    const detail = host.querySelector('[data-part="detail"]');
    assert.ok(master instanceof HTMLElement && detail instanceof HTMLElement);
    assert.equal(master.getAttribute("aria-label"), "Mailboxes");
    assert.equal(detail.tabIndex, -1);
  },
};
