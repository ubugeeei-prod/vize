import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withModifiers as _withModifiers } from "vue"

import { computed, inject, useTemplateRef } from 'vue'
import { url } from '@@/js/config.js'
import * as os from '@/os.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'

export type MkABehavior = 'window' | 'browser' | null;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkA',
  props: {
    to: { type: String, required: true },
    activeClass: { type: String, required: false, default: null },
    behavior: { type: String, required: false, default: null }
  },
  setup(__props: any, { expose: __expose }) {

const props = __props
const behavior = props.behavior ?? inject<MkABehavior>('linkNavigationBehavior', null);
const el = useTemplateRef('el');
const router = useRouter();
const active = computed(() => {
	if (props.activeClass == null) return false;
	const resolved = router.resolve(props.to);
	if (resolved == null) return false;
	if (resolved.route.path === router.currentRoute.value.path) return true;
	if (resolved.route.name == null) return false;
	if (router.currentRoute.value.name == null) return false;
	return resolved.route.name === router.currentRoute.value.name;
});
function onContextmenu(ev: PointerEvent) {
	const selection = window.getSelection();
	if (selection && selection.toString() !== '') return;
	os.contextMenu([{
		type: 'label',
		text: props.to,
	}, {
		icon: 'ti ti-app-window',
		text: i18n.ts.openInWindow,
		action: () => {
			os.pageWindow(props.to);
		},
	}, {
		icon: 'ti ti-player-eject',
		text: i18n.ts.showInPage,
		action: () => {
			router.pushByPath(props.to, 'forcePage');
		},
	}, { type: 'divider' }, {
		icon: 'ti ti-external-link',
		text: i18n.ts.openInNewTab,
		action: () => {
			window.open(props.to, '_blank', 'noopener');
		},
	}, {
		icon: 'ti ti-link',
		text: i18n.ts.copyLink,
		action: () => {
			copyToClipboard(`${url}${props.to}`);
		},
	}], ev);
}
function openWindow() {
	os.pageWindow(props.to);
}
function nav(ev: PointerEvent) {
	// 制御キーとの組み合わせは無視（shiftを除く）
	if (ev.metaKey || ev.altKey || ev.ctrlKey) return;
	ev.preventDefault();
	if (behavior === 'browser') {
		window.location.href = props.to;
		return;
	}
	if (behavior === 'window') {
		return openWindow();
	}
	if (ev.shiftKey) {
		return openWindow();
	}
	router.pushByPath(props.to, ev.ctrlKey ? 'forcePage' : null);
}
__expose({ $el: el })

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("a", {
      ref_key: "el", ref: el,
      href: __props.to,
      class: _normalizeClass(active.value ? __props.activeClass : null),
      onClick: nav,
      onContextmenu: _withModifiers(onContextmenu, ["prevent","stop"])
    }, [ _renderSlot(_ctx.$slots, "default") ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["href"]))
}
}

})
