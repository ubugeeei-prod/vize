import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = { style: "pointer-events: none;" }
import { onMounted, nextTick, ref, watch, computed, toRefs, useTemplateRef } from 'vue'
import { useInterval } from '@@/js/use-interval.js'
import type { MenuItem } from '@/types/menu.js'
import * as os from '@/os.js'

type ModelTChecked = MODELT & (
	MODELT extends GetMkSelectValueTypesFromDef<ITEMS>
		? unknown
		: 'Error: The type of model does not match the type of items.'
);

import type { OptionValue } from '@/types/option-value.js';

export type ItemOption<T extends OptionValue = OptionValue> = {
	type?: 'option';
	value: T;
	label: string;
};

export type ItemGroup<T extends OptionValue = OptionValue> = {
	type: 'group';
	label?: string;
	items: ItemOption<T>[];
};

export type MkSelectItem<T extends OptionValue = OptionValue> = ItemOption<T> | ItemGroup<T>;

export type GetMkSelectValueType<T extends MkSelectItem> = T extends ItemGroup
	? T['items'][number]['value']
	: T extends ItemOption
		? T['value']
		: never;

export type GetMkSelectValueTypesFromDef<T extends MkSelectItem[]> = T[number] extends MkSelectItem
	? GetMkSelectValueType<T[number]>
	: never;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSelect',
  props: {
    items: { type: null, required: true },
    required: { type: Boolean, required: false },
    readonly: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false },
    placeholder: { type: String, required: false },
    autofocus: { type: Boolean, required: false },
    inline: { type: Boolean, required: false },
    small: { type: Boolean, required: false },
    large: { type: Boolean, required: false },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const props = __props
const model = _useModel(__props, "modelValue")
const { autofocus } = toRefs(props);
const focused = ref(false);
const opening = ref(false);
const currentValueText = ref<string | null>(null);
const inputEl = useTemplateRef('inputEl');
const prefixEl = useTemplateRef('prefixEl');
const suffixEl = useTemplateRef('suffixEl');
const container = useTemplateRef('container');
const height =
	props.small ? 33 :
	props.large ? 39 :
	36;
const focus = () => container.value?.focus();
// このコンポーネントが作成された時、非表示状態である場合がある
// 非表示状態だと要素の幅などは0になってしまうので、定期的に計算する
useInterval(() => {
	if (inputEl.value == null) return;
	if (prefixEl.value) {
		if (prefixEl.value.offsetWidth) {
			inputEl.value.style.paddingLeft = prefixEl.value.offsetWidth + 'px';
		}
	}
	if (suffixEl.value) {
		if (suffixEl.value.offsetWidth) {
			inputEl.value.style.paddingRight = suffixEl.value.offsetWidth + 'px';
		}
	}
}, 100, {
	immediate: true,
	afterMounted: true,
});
onMounted(() => {
	nextTick(() => {
		if (autofocus.value) {
			focus();
		}
	});
});
watch([model, () => props.items], () => {
	let found: ItemOption | null = null;
	for (const item of props.items) {
		if (item.type === 'group') {
			for (const option of item.items) {
				if (option.value === model.value) {
					found = option;
					break;
				}
			}
		} else {
			if (item.value === model.value) {
				found = item;
				break;
			}
		}
	}
	if (found) {
		currentValueText.value = found.label;
	}
}, { immediate: true, deep: true });
function show() {
	if (opening.value || props.disabled || props.readonly) return;
	focus();
	opening.value = true;
	const menu: MenuItem[] = [];
	for (const item of props.items) {
		if (item.type === 'group') {
			if (item.label != null) {
				menu.push({
					type: 'label',
					text: item.label,
				});
			}
			for (const option of item.items) {
				menu.push({
					text: option.label,
					active: computed(() => model.value === option.value),
					action: () => {
						model.value = option.value as ModelTChecked;
					},
				});
			}
		} else {
			menu.push({
				text: item.label,
				active: computed(() => model.value === item.value),
				action: () => {
					model.value = item.value as ModelTChecked;
				},
			});
		}
	}
	os.popupMenu(menu, container.value, {
		width: container.value?.offsetWidth,
		onClosing: () => {
			opening.value = false;
		},
	});
}

return (_ctx: any,_cache: any) => {
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label),
        onClick: focus
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        ref_key: "container", ref: container,
        tabindex: "0",
        class: _normalizeClass([_ctx.$style.input, { [_ctx.$style.inline]: __props.inline, [_ctx.$style.disabled]: __props.disabled, [_ctx.$style.focused]: focused.value || opening.value }]),
        onFocus: _cache[0] || (_cache[0] = ($event: any) => (focused.value = true)),
        onBlur: _cache[1] || (_cache[1] = ($event: any) => (focused.value = false)),
        onMousedown: _withModifiers(show, ["prevent"]),
        onKeydown: _withKeys(show, ["space","enter"])
      }, [ _createElementVNode("div", {
          ref_key: "prefixEl", ref: prefixEl,
          class: _normalizeClass(_ctx.$style.prefix)
        }, [ _renderSlot(_ctx.$slots, "prefix") ], 512 /* NEED_PATCH */), _createElementVNode("div", {
          ref_key: "inputEl", ref: inputEl,
          tabindex: "-1",
          class: _normalizeClass(_ctx.$style.inputCore),
          disabled: __props.disabled,
          required: __props.required,
          readonly: __props.readonly,
          placeholder: __props.placeholder,
          onMousedown: _cache[2] || (_cache[2] = _withModifiers(() => {}, ["prevent"])),
          onKeydown: _cache[3] || (_cache[3] = _withModifiers(() => {}, ["prevent"]))
        }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(currentValueText.value ?? ''), 1 /* TEXT */), _createElementVNode("div", { style: "display: none;" }, [ _renderSlot(_ctx.$slots, "default") ]) ], 40 /* PROPS, NEED_HYDRATION */, ["disabled", "required", "readonly", "placeholder"]), _createElementVNode("div", {
          ref_key: "suffixEl", ref: suffixEl,
          class: _normalizeClass(_ctx.$style.suffix)
        }, [ _createElementVNode("i", {
            class: _normalizeClass(["ti ti-chevron-down", [_ctx.$style.chevron, { [_ctx.$style.chevronOpening]: opening.value }]])
          }, null, 2 /* CLASS */) ], 512 /* NEED_PATCH */) ], 34 /* CLASS, NEED_HYDRATION */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]) ]))
}
}

})
