import { defineComponent, h, ref } from "vue";
import type { PropType } from "vue";

import MenuArrow from "./menu-arrow.vue";
import MenuCheckboxItem from "./menu-checkbox-item.vue";
import MenuContent from "./menu-content.vue";
import MenuGroup from "./menu-group.vue";
import MenuItem from "./menu-item.vue";
import MenuItemIndicator from "./menu-item-indicator.vue";
import MenuLabel from "./menu-label.vue";
import MenuRadioGroup from "./menu-radio-group.vue";
import MenuRadioItem from "./menu-radio-item.vue";
import MenuRoot from "./menu-root.vue";
import MenuSeparator from "./menu-separator.vue";
import MenuSub from "./menu-sub.vue";
import MenuSubContent from "./menu-sub-content.vue";
import MenuSubTrigger from "./menu-sub-trigger.vue";
import MenuTrigger from "./menu-trigger.vue";
import type { MenuSelectEvent } from "./menu-types.ts";

/** One log line per observable menu event, in dispatch order. */
export type MenuLog = string[];

/** Shared test tree: groups, disabled items, checkbox, radio group, and a submenu. */
export const MenuFixture = defineComponent({
  name: "MenuFixture",
  props: {
    log: { type: Array as PropType<MenuLog>, default: () => [] },
    rootProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    contentProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    preventSelect: { type: Boolean, default: false },
  },
  setup(props) {
    const size = ref<"small" | "large" | null>("small");
    const hidden = ref(false);
    const onSelect = (name: string) => (event: MenuSelectEvent) => {
      props.log.push(`select:${name}`);
      if (props.preventSelect) event.preventDefault();
    };
    return () =>
      h("div", { "data-testid": "fixture" }, [
        h("button", { type: "button", "data-testid": "before" }, "Before"),
        h(
          MenuRoot,
          {
            id: "actions",
            ...props.rootProps,
            "onOpen-change": (value: boolean) => props.log.push(`open:${value}`),
          },
          () => [
            h(MenuTrigger, null, () => "Actions"),
            h(MenuContent, { ...props.contentProps }, () => [
              h(MenuGroup, null, () => [
                h(MenuLabel, null, () => "File"),
                h(MenuItem, { onSelect: onSelect("new") }, () => "New file"),
                h(MenuItem, { onSelect: onSelect("open") }, () => "Open"),
                h(MenuItem, { disabled: true, onSelect: onSelect("delete") }, () => "Delete"),
              ]),
              h(MenuSeparator),
              h(
                MenuCheckboxItem,
                {
                  modelValue: hidden.value,
                  "onUpdate:modelValue": (value: boolean) => {
                    hidden.value = value;
                    props.log.push(`hidden:${value}`);
                  },
                  closeOnSelect: false,
                },
                () => [h(MenuItemIndicator, null, () => "✓"), "Show hidden"],
              ),
              h(
                MenuRadioGroup<"small" | "large">,
                {
                  modelValue: size.value,
                  "onUpdate:modelValue": (value: "small" | "large" | null) => {
                    size.value = value;
                    props.log.push(`size:${String(value)}`);
                  },
                },
                () => [
                  h(MenuRadioItem<"small" | "large">, { value: "small" }, () => [
                    h(MenuItemIndicator, null, () => "•"),
                    "Small",
                  ]),
                  h(MenuRadioItem<"small" | "large">, { value: "large" }, () => [
                    h(MenuItemIndicator, null, () => "•"),
                    "Large",
                  ]),
                ],
              ),
              h(MenuSub, null, () => [
                h(MenuSubTrigger, null, () => "Share"),
                h(MenuSubContent, null, () => [
                  h(MenuItem, { onSelect: onSelect("email") }, () => "Email"),
                  h(MenuItem, { onSelect: onSelect("link") }, () => "Copy link"),
                ]),
              ]),
              h(MenuArrow),
            ]),
          ],
        ),
        h("button", { type: "button", "data-testid": "after" }, "After"),
      ]);
  },
});
