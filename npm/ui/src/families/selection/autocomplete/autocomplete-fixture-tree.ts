import { h } from "vue";
import type { VNode } from "vue";

import ComboboxInput from "../combobox/combobox-input.vue";
import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import AutocompleteRoot from "./autocomplete-root.vue";
import type { AutocompleteSlotState } from "./autocomplete-types.ts";

/** Canonical open tree shared by SSR tests and runtime fixtures. */
export function renderAutocompleteTree(): VNode {
  const recent: readonly string[] = ["vize", "vapor"];
  return h(
    AutocompleteRoot<string>,
    { defaultHistory: recent, id: "site-search", items: ["vue", "vite", "vitest"], name: "q" },
    {
      default: (state: AutocompleteSlotState<string>) => [
        h(ComboboxInput, { ariaLabel: "Search" }),
        h(SelectContent, { forceMount: true }, () => [
          ...state.suggestions.map((entry) =>
            h(SelectItem<string>, { key: entry, value: entry }, () => entry),
          ),
          h(
            "button",
            {
              "data-clear": "",
              hidden: !state.showingHistory,
              onClick: state.clearHistory,
              type: "button",
            },
            "Clear",
          ),
        ]),
      ],
    },
  );
}
