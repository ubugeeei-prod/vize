import { h } from "vue";
import type { VNode } from "vue";

import TransferListAction from "./transfer-list-action.vue";
import TransferListEmpty from "./transfer-list-empty.vue";
import TransferListItem from "./transfer-list-item.vue";
import TransferListPanel from "./transfer-list-panel.vue";
import TransferListRoot from "./transfer-list-root.vue";
import TransferListSearch from "./transfer-list-search.vue";
import type { TransferListSlotState } from "./transfer-list-types.ts";

/** Canonical server-rendered tree shared by SSR tests and runtime fixtures. */
export function renderTransferListTree(): VNode {
  const fruits: readonly string[] = ["Apple", "Banana", "Cherry"];
  return h(
    TransferListRoot<string>,
    { defaultValue: ["Banana"], id: "fruit-transfer", items: fruits, name: "fruits" },
    {
      default: (state: TransferListSlotState<string>) => [
        h(TransferListSearch, { ariaLabel: "Search available", side: "source" }),
        h(TransferListPanel, { ariaLabel: "Available", side: "source" }, () =>
          state.visibleSource.map((fruit) =>
            h(TransferListItem<string>, { key: fruit, value: fruit }, () => fruit),
          ),
        ),
        h(TransferListEmpty, { side: "source" }, () => "Empty"),
        h(TransferListAction, { action: "move-all-to-target" }, () => ">>"),
        h(TransferListPanel, { ariaLabel: "Selected", side: "target" }, () =>
          state.visibleTarget.map((fruit) =>
            h(TransferListItem<string>, { key: fruit, value: fruit }, () => fruit),
          ),
        ),
      ],
    },
  );
}
