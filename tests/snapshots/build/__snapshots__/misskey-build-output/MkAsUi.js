import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import type { AsUiComponent, AsUiRoot, AsUiPostFormButton } from '@/aiscript/ui.js'
import * as os from '@/os.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkPostForm from '@/components/MkPostForm.vue'
import { useMkSelect } from '@/composables/use-mkselect.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAsUi',
  props: {
    component: { type: null, required: true },
    components: { type: Array, required: true },
    size: { type: String, required: false, default: 'medium' },
    align: { type: String, required: false, default: 'left' }
  },
  setup(__props: any) {

const props = __props
const c = props.component;
function g(id: string) {
	const v = props.components.find(x => x.value.id === id)?.value;
	if (v) return v;
	return {
		id: 'dummy',
		type: 'root',
		children: [],
	} as AsUiRoot;
}
const containerStyle = computed(() => {
	if (c.type !== 'container') return undefined;

	// width, color, styleのうち一つでも指定があれば、枠線がちゃんと表示されるようにwidthとstyleのデフォルト値を設定
	// radiusは単に角を丸める用途もあるため除外
	const isBordered = c.borderWidth ?? c.borderColor ?? c.borderStyle;

	const border = isBordered ? {
		borderWidth: `${c.borderWidth ?? 1}px`,
		borderColor: c.borderColor ?? 'var(--MI_THEME-divider)',
		borderStyle: c.borderStyle ?? 'solid',
	} : undefined;

	return {
		textAlign: c.align,
		backgroundColor: c.bgColor,
		color: c.fgColor,
		padding: c.padding ? `${c.padding}px` : 0,
		borderRadius: (c.borderRadius ?? (c.rounded ? 8 : 0)) + 'px',
		...border,
	};
});
const valueForSwitch = ref('default' in c && typeof c.default === 'boolean' ? c.default : false);
function onSwitchUpdate(v: boolean) {
	valueForSwitch.value = v;
	if ('onChange' in c && c.onChange) {
		c.onChange(v as never);
	}
}
const {
	model: valueForSelect,
	def: selectDef,
} = useMkSelect({
	items: computed(() => {
		if (c.type !== 'select') return [];
		return (c.items ?? []).map(item => ({
			value: item.value,
			label: item.text,
		}));
	}),
	initialValue: (c.type === 'select' && 'default' in c && typeof c.default !== 'boolean') ? c.default ?? null : null,
});
function onSelectUpdate(v: string | null) {
	valueForSelect.value = v;
	if ('onChange' in c && c.onChange) {
		c.onChange(v as never);
	}
}
function openPostForm() {
	const form = (c as AsUiPostFormButton).form;
	if (!form) return;
	os.post({
		initialText: form.text,
		initialCw: form.cw,
		initialVisibility: form.visibility,
		initialLocalOnly: form.localOnly,
		instant: true,
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkAsUi = _resolveComponent("MkAsUi")
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createElementBlock("div", null, [ (_unref(c).type === 'root') ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.root)
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(c).children, (child) => {
            return (_openBlock(), _createElementBlock(_Fragment, { key: child }, [
              (!g(child).hidden)
                ? (_openBlock(), _createBlock(_component_MkAsUi, {
                  key: 0,
                  component: g(child),
                  components: props.components,
                  size: __props.size
                }, null, 8 /* PROPS */, ["component", "components", "size"]))
                : _createCommentVNode("v-if", true)
            ], 64 /* STABLE_FRAGMENT */))
          }), 128 /* KEYED_FRAGMENT */)) ])) : (_unref(c).type === 'text') ? (_openBlock(), _createElementBlock("span", {
            key: 1,
            class: _normalizeClass({ [_ctx.$style.fontSerif]: _unref(c).font === 'serif', [_ctx.$style.fontMonospace]: _unref(c).font === 'monospace' }),
            style: _normalizeStyle({ fontSize: _unref(c).size ? `${_unref(c).size * 100}%` : undefined, fontWeight: _unref(c).bold ? 'bold' : undefined, color: _unref(c).color })
          }, _toDisplayString(_unref(c).text), 1 /* TEXT */)) : (_unref(c).type === 'mfm') ? (_openBlock(), _createBlock(_component_Mfm, {
            key: 2,
            class: _normalizeClass({ [_ctx.$style.fontSerif]: _unref(c).font === 'serif', [_ctx.$style.fontMonospace]: _unref(c).font === 'monospace' }),
            style: _normalizeStyle({ fontSize: _unref(c).size ? `${_unref(c).size * 100}%` : null, fontWeight: _unref(c).bold ? 'bold' : null, color: _unref(c).color ?? null }),
            text: _unref(c).text ?? '',
            onClickEv: _cache[0] || (_cache[0] = ($event: any) => (_unref(c).onClickEv))
          }, null, 14 /* CLASS, STYLE, PROPS */, ["text"])) : (_unref(c).type === 'button') ? (_openBlock(), _createBlock(MkButton, {
            key: 3,
            primary: _unref(c).primary,
            rounded: _unref(c).rounded,
            disabled: _unref(c).disabled,
            small: __props.size === 'small',
            inline: "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(c).onClick))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(c).text), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["primary", "rounded", "disabled", "small"])) : (_unref(c).type === 'buttons') ? (_openBlock(), _createElementBlock("div", {
            key: 4,
            class: "_buttons",
            style: _normalizeStyle({ justifyContent: __props.align })
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(c).buttons, (button) => {
              return (_openBlock(), _createBlock(MkButton, { primary: button.primary, rounded: button.rounded, disabled: button.disabled, inline: "", small: __props.size === 'small', onClick: _cache[2] || (_cache[2] = ($event: any) => (button.onClick)) }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(button.text), 1 /* TEXT */)
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["primary", "rounded", "disabled", "small"]))
            }), 256 /* UNKEYED_FRAGMENT */)) ])) : (_unref(c).type === 'switch') ? (_openBlock(), _createBlock(MkSwitch, {
            key: 5,
            modelValue: valueForSwitch.value,
            "onUpdate:modelValue": onSwitchUpdate
          }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(c).label) ? {
                name: "label",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).label), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined, (_unref(c).caption) ? {
                name: "caption",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).caption), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["modelValue"])) : (_unref(c).type === 'textarea') ? (_openBlock(), _createBlock(MkTextarea, {
            key: 6,
            modelValue: _unref(c).default ?? null,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => (_unref(c).onInput))
          }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(c).label) ? {
                name: "label",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).label), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined, (_unref(c).caption) ? {
                name: "caption",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).caption), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["modelValue"])) : (_unref(c).type === 'textInput') ? (_openBlock(), _createBlock(MkInput, {
            key: 7,
            small: __props.size === 'small',
            modelValue: _unref(c).default ?? null,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => (_unref(c).onInput))
          }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(c).label) ? {
                name: "label",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).label), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined, (_unref(c).caption) ? {
                name: "caption",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).caption), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["small", "modelValue"])) : (_unref(c).type === 'numberInput') ? (_openBlock(), _createBlock(MkInput, {
            key: 8,
            small: __props.size === 'small',
            modelValue: _unref(c).default ?? null,
            type: "number",
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => (_unref(c).onInput))
          }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(c).label) ? {
                name: "label",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).label), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined, (_unref(c).caption) ? {
                name: "caption",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).caption), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["small", "modelValue"])) : (_unref(c).type === 'select') ? (_openBlock(), _createBlock(MkSelect, {
            key: 9,
            small: __props.size === 'small',
            modelValue: _unref(valueForSelect),
            items: _unref(selectDef),
            "onUpdate:modelValue": onSelectUpdate
          }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(c).label) ? {
                name: "label",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).label), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined, (_unref(c).caption) ? {
                name: "caption",
                fn: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(c).caption), 1 /* TEXT */)
                ]),
                key: "0"
              } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["small", "modelValue", "items"])) : (_unref(c).type === 'postFormButton') ? (_openBlock(), _createBlock(MkButton, {
            key: 10,
            primary: _unref(c).primary,
            rounded: _unref(c).rounded,
            small: __props.size === 'small',
            inline: "",
            onClick: openPostForm
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(c).text), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["primary", "rounded", "small"])) : (_unref(c).type === 'postForm') ? (_openBlock(), _createElementBlock("div", {
            key: 11,
            class: _normalizeClass(_ctx.$style.postForm)
          }, [ _createVNode(MkPostForm, {
              fixed: "",
              instant: true,
              initialText: _unref(c).form?.text,
              initialCw: _unref(c).form?.cw,
              initialVisibility: _unref(c).form?.visibility,
              initialLocalOnly: _unref(c).form?.localOnly
            }, null, 8 /* PROPS */, ["instant", "initialText", "initialCw", "initialVisibility", "initialLocalOnly"]) ])) : (_unref(c).type === 'folder') ? (_openBlock(), _createBlock(MkFolder, {
            key: 12,
            defaultOpen: _unref(c).opened
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(c).title), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(c).children, (child) => {
                return (_openBlock(), _createElementBlock(_Fragment, { key: child }, [
                  (!g(child).hidden)
                    ? (_openBlock(), _createBlock(_component_MkAsUi, {
                      key: 0,
                      component: g(child),
                      components: props.components,
                      size: __props.size
                    }, null, 8 /* PROPS */, ["component", "components", "size"]))
                    : _createCommentVNode("v-if", true)
                ], 64 /* STABLE_FRAGMENT */))
              }), 128 /* KEYED_FRAGMENT */))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["defaultOpen"])) : (_unref(c).type === 'container') ? (_openBlock(), _createElementBlock("div", {
            key: 13,
            class: _normalizeClass([_ctx.$style.container, { [_ctx.$style.fontSerif]: _unref(c).font === 'serif', [_ctx.$style.fontMonospace]: _unref(c).font === 'monospace' }]),
            style: _normalizeStyle(containerStyle.value)
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(c).children, (child) => {
              return (_openBlock(), _createElementBlock(_Fragment, { key: child }, [
                (!g(child).hidden)
                  ? (_openBlock(), _createBlock(_component_MkAsUi, {
                    key: 0,
                    component: g(child),
                    components: props.components,
                    size: __props.size,
                    align: _unref(c).align
                  }, null, 8 /* PROPS */, ["component", "components", "size", "align"]))
                  : _createCommentVNode("v-if", true)
              ], 64 /* STABLE_FRAGMENT */))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
