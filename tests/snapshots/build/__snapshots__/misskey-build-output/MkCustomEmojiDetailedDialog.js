import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { useTemplateRef } from 'vue'
import MkLink from '@/components/MkLink.vue'
import { i18n } from '@/i18n.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCustomEmojiDetailedDialog',
  props: {
    emoji: { type: null, required: true }
  },
  emits: ["ok", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const dialogEl = useTemplateRef('dialogEl');
function cancel() {
	emit('cancel');
	dialogEl.value!.close();
}

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialogEl", ref: dialogEl,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(":" + _toDisplayString(__props.emoji.name) + ":", 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_spacer" }, [
          _createElementVNode("div", { style: "display: flex; flex-direction: column; gap: 1em;" }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.emojiImgWrapper)
            }, [
              _createVNode(_component_MkCustomEmoji, {
                name: __props.emoji.name,
                normal: true,
                useOriginalSize: true,
                style: "height: 100%;"
              }, null, 8 /* PROPS */, ["name", "normal", "useOriginalSize"])
            ]),
            _createVNode(MkKeyValue, { copy: `:${__props.emoji.name}:` }, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.emoji.name), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"]),
            _createVNode(MkKeyValue, null, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.tags), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                (__props.emoji.aliases.length === 0)
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts.none), 1 /* TEXT */))
                  : (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.aliases)
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.emoji.aliases, (alias) => {
                      return (_openBlock(), _createElementBlock("span", {
                        key: alias,
                        class: _normalizeClass(_ctx.$style.alias)
                      }, _toDisplayString(alias), 1 /* TEXT */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]))
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkKeyValue, null, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.category), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.emoji.category ?? _unref(i18n).ts.none), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkKeyValue, null, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.emoji.isSensitive ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkKeyValue, null, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.localOnly), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.emoji.localOnly ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkKeyValue, null, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.license), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createVNode(_component_Mfm, { text: __props.emoji.license ?? _unref(i18n).ts.none }, null, 8 /* PROPS */, ["text"])
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkKeyValue, { copy: __props.emoji.url }, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.emojiUrl), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                _createVNode(MkLink, {
                  url: __props.emoji.url,
                  target: "_blank"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(__props.emoji.url), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["url"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 512 /* NEED_PATCH */))
}
}

})
