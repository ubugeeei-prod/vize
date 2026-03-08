import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkModal from '@/components/MkModal.vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { updateCurrentAccountPartial } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAnnouncementDialog',
  props: {
    announcement: { type: null, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const rootEl = useTemplateRef('rootEl');
const bottomEl = useTemplateRef('bottomEl');
const modal = useTemplateRef('modal');
async function ok() {
	if (props.announcement.needConfirmationToRead) {
		const confirm = await os.confirm({
			type: 'question',
			title: i18n.ts._announcement.readConfirmTitle,
			text: i18n.tsx._announcement.readConfirmText({ title: props.announcement.title }),
		});
		if (confirm.canceled) return;
	}
	modal.value?.close();
	misskeyApi('i/read-announcement', { announcementId: props.announcement.id });
	updateCurrentAccountPartial({
		unreadAnnouncements: $i!.unreadAnnouncements.filter(a => a.id !== props.announcement.id),
	});
}
function onBgClick() {
	rootEl.value?.animate([{
		offset: 0,
		transform: 'scale(1)',
	}, {
		offset: 0.5,
		transform: 'scale(1.1)',
	}, {
		offset: 1,
		transform: 'scale(1)',
	}], {
		duration: 100,
	});
}
const hasReachedBottom = ref(false);
onMounted(() => {
	if (bottomEl.value && rootEl.value) {
		const bottomElRect = bottomEl.value.getBoundingClientRect();
		const rootElRect = rootEl.value.getBoundingClientRect();
		if (
			bottomElRect.top >= rootElRect.top &&
			bottomElRect.top <= (rootElRect.bottom - 66) // 66 ≒ 75 * 0.9 (modalのアニメーション分)
		) {
			hasReachedBottom.value = true;
			return;
		}
		const observer = new IntersectionObserver(entries => {
			for (const entry of entries) {
				if (entry.isIntersecting) {
					hasReachedBottom.value = true;
					observer.disconnect();
				}
			}
		}, {
			root: rootEl.value,
			rootMargin: '0px 0px -75px 0px',
		});
		observer.observe(bottomEl.value);
	}
});

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      zPriority: 'middle',
      preferType: 'dialog',
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed'))),
      onClick: onBgClick
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          ref_key: "rootEl", ref: rootEl,
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.header)
          }, [
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.icon)
            }, [
              (__props.announcement.icon === 'info')
                ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: "ti ti-info-circle"
                }))
                : (__props.announcement.icon === 'warning')
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-alert-triangle",
                    style: "color: var(--MI_THEME-warn);"
                  }))
                : (__props.announcement.icon === 'error')
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 2,
                    class: "ti ti-circle-x",
                    style: "color: var(--MI_THEME-error);"
                  }))
                : (__props.announcement.icon === 'success')
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 3,
                    class: "ti ti-check",
                    style: "color: var(--MI_THEME-success);"
                  }))
                : _createCommentVNode("v-if", true)
            ]),
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.title)
            }, _toDisplayString(__props.announcement.title), 1 /* TEXT */)
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.text)
          }, [
            _createVNode(_component_Mfm, { text: __props.announcement.text }, null, 8 /* PROPS */, ["text"])
          ]),
          _createElementVNode("div", { ref_key: "bottomEl", ref: bottomEl }, null, 512 /* NEED_PATCH */),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.footer)
          }, [
            _createVNode(MkButton, {
              primary: "",
              full: "",
              disabled: !hasReachedBottom.value,
              onClick: ok
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(hasReachedBottom.value ? _unref(i18n).ts.close : _unref(i18n).ts.scrollToClose), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["disabled"])
          ])
        ], 512 /* NEED_PATCH */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["zPriority", "preferType"]))
}
}

})
