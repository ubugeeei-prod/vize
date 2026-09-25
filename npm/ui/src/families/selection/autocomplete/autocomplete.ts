/**
 * Address- and search-style Autocomplete: a thin Combobox preset with free
 * text, open-on-focus, and recent history. `AutocompleteInput`,
 * `AutocompleteContent`, `AutocompleteItem`, `AutocompleteEmpty`, and
 * `AutocompleteLoading` are the Combobox parts under preset names.
 */
export { default as AutocompleteContent } from "../select/select-content.vue";
export { default as AutocompleteItem } from "../select/select-item.vue";
export { default as AutocompleteInput } from "../combobox/combobox-input.vue";
export { default as AutocompleteEmpty } from "../combobox/combobox-empty.vue";
export { default as AutocompleteLoading } from "../combobox/combobox-loading.vue";
export { default as Autocomplete, default as AutocompleteRoot } from "./autocomplete-root.vue";
export {
  createWebStorageHistory,
  pushAutocompleteHistory,
  removeAutocompleteHistory,
} from "./autocomplete-history.ts";
export type { WebStorageHistoryOptions } from "./autocomplete-history.ts";
export type {
  AutocompleteHistoryStorage,
  AutocompleteRootExpose,
  AutocompleteRootProps,
  AutocompleteSlotState,
} from "./autocomplete-types.ts";
