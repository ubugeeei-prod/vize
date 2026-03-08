import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-home" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
import { nextTick, useTemplateRef, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkModal from '@/components/MkModal.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkVisibilityPicker',
  props: {
    currentVisibility: { type: null, required: true },
    isSilenced: { type: Boolean, required: true },
    anchorElement: { type: null, required: false },
    isReplyVisibilitySpecified: { type: Boolean, required: false }
  },
  emits: ["changeVisibility", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const modal = useTemplateRef('modal');
const v = ref(props.currentVisibility);
function choose(visibility: typeof Misskey.noteVisibilities[number]): void {
	v.value = visibility;
	emit('changeVisibility', visibility);
	nextTick(() => {
		if (modal.value) modal.value.close();
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      zPriority: 'high',
      anchorElement: __props.anchorElement,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(modal)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed'))),
      onEsc: _cache[2] || (_cache[2] = ($event: any) => (_unref(modal)?.close()))
    }, {
      default: _withCtx(({ type }) => [
        _createElementVNode("div", {
          class: _normalizeClass(["_popup", { [_ctx.$style.root]: true, [_ctx.$style.asDrawer]: type === 'drawer' }])
        }, [
          _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.label, _ctx.$style.item])
          }, _toDisplayString(_unref(i18n).ts.visibility), 1 /* TEXT */),
          _createElementVNode("button", {
            key: "public",
            disabled: __props.isSilenced || __props.isReplyVisibilitySpecified,
            class: _normalizeClass(["_button", [_ctx.$style.item, { [_ctx.$style.active]: v.value === 'public' }]]),
            "data-index": "1",
            onClick: _cache[3] || (_cache[3] = ($event: any) => (choose('public')))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.icon)
            }, [
              _hoisted_1
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.body)
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemTitle)
              }, _toDisplayString(_unref(i18n).ts._visibility.public), 1 /* TEXT */),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemDescription)
              }, _toDisplayString(_unref(i18n).ts._visibility.publicDescription), 1 /* TEXT */)
            ])
          ], 10 /* CLASS, PROPS */, ["disabled"]),
          _createElementVNode("button", {
            key: "home",
            disabled: __props.isReplyVisibilitySpecified,
            class: _normalizeClass(["_button", [_ctx.$style.item, { [_ctx.$style.active]: v.value === 'home' }]]),
            "data-index": "2",
            onClick: _cache[4] || (_cache[4] = ($event: any) => (choose('home')))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.icon)
            }, [
              _hoisted_2
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.body)
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemTitle)
              }, _toDisplayString(_unref(i18n).ts._visibility.home), 1 /* TEXT */),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemDescription)
              }, _toDisplayString(_unref(i18n).ts._visibility.homeDescription), 1 /* TEXT */)
            ])
          ], 10 /* CLASS, PROPS */, ["disabled"]),
          _createElementVNode("button", {
            key: "followers",
            disabled: __props.isReplyVisibilitySpecified,
            class: _normalizeClass(["_button", [_ctx.$style.item, { [_ctx.$style.active]: v.value === 'followers' }]]),
            "data-index": "3",
            onClick: _cache[5] || (_cache[5] = ($event: any) => (choose('followers')))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.icon)
            }, [
              _hoisted_3
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.body)
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemTitle)
              }, _toDisplayString(_unref(i18n).ts._visibility.followers), 1 /* TEXT */),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemDescription)
              }, _toDisplayString(_unref(i18n).ts._visibility.followersDescription), 1 /* TEXT */)
            ])
          ], 10 /* CLASS, PROPS */, ["disabled"]),
          _createElementVNode("button", {
            key: "specified",
            class: _normalizeClass(["_button", [_ctx.$style.item, { [_ctx.$style.active]: v.value === 'specified' }]]),
            "data-index": "4",
            onClick: _cache[6] || (_cache[6] = ($event: any) => (choose('specified')))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.icon)
            }, [
              _hoisted_4
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.body)
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemTitle)
              }, _toDisplayString(_unref(i18n).ts._visibility.specified), 1 /* TEXT */),
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.itemDescription)
              }, _toDisplayString(_unref(i18n).ts._visibility.specifiedDescription), 1 /* TEXT */)
            ])
          ], 2 /* CLASS */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["zPriority", "anchorElement"]))
}
}

})
