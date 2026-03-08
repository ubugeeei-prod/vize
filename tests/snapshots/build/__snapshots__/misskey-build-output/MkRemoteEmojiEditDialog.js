import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { computed, ref, useTemplateRef } from 'vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkWindow from '@/components/MkWindow.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRemoteEmojiEditDialog',
  props: {
    emoji: { type: Object, required: true }
  },
  emits: ["done", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const windowEl = useTemplateRef('windowEl');
const name = computed(() => props.emoji.name);
const host = computed(() => props.emoji.host);
const license = computed(() => props.emoji.license);
const imgUrl = computed(() => props.emoji.url);
async function done() {
	await os.apiWithDialog('admin/emoji/copy', {
		emojiId: props.emoji.id,
	});
	emit('done');
	windowEl.value?.close();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkWindow, {
      ref_key: "windowEl", ref: windowEl,
      initialWidth: 400,
      initialHeight: 500,
      canResize: true,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (_unref(windowEl)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(":" + _toDisplayString(name.value) + ":", 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { style: "display: flex; flex-direction: column; min-height: 100%;" }, [
          _createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px; flex-grow: 1;"
          }, [
            _createElementVNode("div", { class: "_gaps_m" }, [
              (imgUrl.value != null)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.imgs)
                }, [
                  _createElementVNode("div", {
                    style: "background: #000;",
                    class: _normalizeClass(_ctx.$style.imgContainer)
                  }, [
                    _createElementVNode("img", {
                      src: imgUrl.value,
                      class: _normalizeClass(_ctx.$style.img),
                      alt: name.value
                    }, null, 8 /* PROPS */, ["src", "alt"])
                  ]),
                  _createElementVNode("div", {
                    style: "background: #222;",
                    class: _normalizeClass(_ctx.$style.imgContainer)
                  }, [
                    _createElementVNode("img", {
                      src: imgUrl.value,
                      class: _normalizeClass(_ctx.$style.img),
                      alt: name.value
                    }, null, 8 /* PROPS */, ["src", "alt"])
                  ]),
                  _createElementVNode("div", {
                    style: "background: #ddd;",
                    class: _normalizeClass(_ctx.$style.imgContainer)
                  }, [
                    _createElementVNode("img", {
                      src: imgUrl.value,
                      class: _normalizeClass(_ctx.$style.img),
                      alt: name.value
                    }, null, 8 /* PROPS */, ["src", "alt"])
                  ]),
                  _createElementVNode("div", {
                    style: "background: #fff;",
                    class: _normalizeClass(_ctx.$style.imgContainer)
                  }, [
                    _createElementVNode("img", {
                      src: imgUrl.value,
                      class: _normalizeClass(_ctx.$style.img),
                      alt: name.value
                    }, null, 8 /* PROPS */, ["src", "alt"])
                  ])
                ]))
                : _createCommentVNode("v-if", true),
              _createVNode(MkKeyValue, null, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(name.value), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkKeyValue, null, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.host), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(host.value), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkKeyValue, null, {
                key: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.license), 1 /* TEXT */)
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(license.value), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.footer)
          }, [
            _createVNode(MkButton, {
              primary: "",
              rounded: "",
              style: "margin: 0 auto;",
              onClick: done
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.import), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["initialWidth", "initialHeight", "canResize"]))
}
}

})
