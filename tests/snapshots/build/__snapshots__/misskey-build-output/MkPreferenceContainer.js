import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { ref } from 'vue'
import type { PREF_DEF } from '@/preferences/def.js'
import * as os from '@/os.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPreferenceContainer',
  props: {
    k: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const isAccountOverrided = ref(prefer.isAccountOverrided(props.k));
const isSyncEnabled = ref(prefer.isSyncEnabled(props.k));
function showMenu(ev: PointerEvent, contextmenu?: boolean) {
	const i = window.setInterval(() => {
		isAccountOverrided.value = prefer.isAccountOverrided(props.k);
		isSyncEnabled.value = prefer.isSyncEnabled(props.k);
	}, 100);
	if (contextmenu) {
		os.contextMenu(prefer.getPerPrefMenu(props.k), ev).then(() => {
			window.clearInterval(i);
		});
	} else {
		os.popupMenu(prefer.getPerPrefMenu(props.k), ev.currentTarget ?? ev.target, {
			onClosing: () => {
				window.clearInterval(i);
			},
		});
	}
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root),
      onContextmenu: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (showMenu($event, true)), ["prevent","stop"]))
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ _renderSlot(_ctx.$slots, "default") ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.menu)
      }, [ (isSyncEnabled.value) ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: "ti ti-cloud-cog",
            style: "color: var(--MI_THEME-accent); opacity: 0.7;"
          })) : _createCommentVNode("v-if", true), (isAccountOverrided.value) ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: "ti ti-user-cog",
            style: "color: var(--MI_THEME-accent); opacity: 0.7;"
          })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.buttons)
        }, [ _createElementVNode("button", {
            class: "_button",
            style: "color: var(--MI_THEME-fg)",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (showMenu($event)))
          }, [ _hoisted_1 ]) ]) ]) ], 32 /* NEED_HYDRATION */))
}
}

})
