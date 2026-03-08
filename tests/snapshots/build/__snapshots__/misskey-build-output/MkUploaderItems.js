import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div")
import { isLink } from '@@/js/is-link.js'
import type { UploaderItem } from '@/composables/use-uploader.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import bytes from '@/filters/bytes.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUploaderItems',
  props: {
    items: { type: Array, required: true }
  },
  emits: ["showMenu", "showMenuViaContextmenu"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
function onContextmenu(item: UploaderItem, ev: PointerEvent) {
	if (ev.target && isLink(ev.target as HTMLElement)) return;
	if (window.getSelection()?.toString() !== '') return;
	emit('showMenuViaContextmenu', item, ev);
}
function onThumbnailClick(item: UploaderItem, ev: PointerEvent) {
	// TODO: preview when item is image
}

return (_ctx: any,_cache: any) => {
  const _component_MkCondensedLine = _resolveComponent("MkCondensedLine")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkSystemIcon = _resolveComponent("MkSystemIcon")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_gaps_s", _ctx.$style.root])
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(props.items, (item) => {
        return (_openBlock(), _createElementBlock("div", {
          key: item.id,
          class: _normalizeClass([_ctx.$style.item, { [_ctx.$style.itemWaiting]: item.preprocessing, [_ctx.$style.itemCompleted]: item.uploaded, [_ctx.$style.itemFailed]: item.uploadFailed }]),
          style: _normalizeStyle({
  			'--p': item.progress != null ? `${item.progress.value / item.progress.max * 100}%` : '0%',
  			'--pp': item.preprocessProgress != null ? `${item.preprocessProgress * 100}%` : '100%',
  		}),
          onContextmenu: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (onContextmenu(item, $event)), ["prevent","stop"]))
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.itemInner)
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.itemActionWrapper)
            }, [
              _createVNode(MkButton, {
                iconOnly: true,
                rounded: "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('showMenu', item, $event)))
              }, {
                default: _withCtx(() => [
                  _hoisted_1
                ]),
                _: 2 /* DYNAMIC */
              }, 8 /* PROPS */, ["iconOnly"])
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.itemThumbnail),
              style: _normalizeStyle({ backgroundImage: `url(${ item.thumbnail })` }),
              onClick: _cache[2] || (_cache[2] = ($event: any) => (onThumbnailClick(item, $event)))
            }, null, 4 /* STYLE */),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.itemBody)
            }, [
              _createElementVNode("div", null, [
                (item.isSensitive)
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    style: "color: var(--MI_THEME-warn); margin-right: 0.5em;",
                    class: "ti ti-eye-exclamation"
                  }))
                  : _createCommentVNode("v-if", true),
                _createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(item.name), 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["minScale"])
              ]),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.itemInfo)
              }, [
                _createElementVNode("span", null, _toDisplayString(item.file.type), 1 /* TEXT */),
                (item.compressedSize)
                  ? (_openBlock(), _createElementBlock("span", { key: 0 }, "(" + _toDisplayString(_unref(i18n).tsx._uploader.compressedToX({ x: bytes(item.compressedSize) })) + " = " + _toDisplayString(_unref(i18n).tsx._uploader.savedXPercent({ x: Math.round((1 - item.compressedSize / item.file.size) * 100) })) + ")", 1 /* TEXT */))
                  : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(bytes(item.file.size)), 1 /* TEXT */)),
                (item.preprocessing)
                  ? (_openBlock(), _createElementBlock("span", { key: 0 }, [
                    _toDisplayString(_unref(i18n).ts.preprocessing),
                    _createVNode(_component_MkLoading, {
                      inline: "",
                      em: "",
                      style: "margin-left: 0.5em;"
                    })
                  ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _hoisted_2
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.itemIconWrapper)
            }, [
              (item.uploading)
                ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.itemIcon),
                  type: "waiting"
                }))
                : (item.uploaded)
                  ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.itemIcon),
                    type: "success"
                  }))
                : (item.uploadFailed)
                  ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                    key: 2,
                    class: _normalizeClass(_ctx.$style.itemIcon),
                    type: "error"
                  }))
                : _createCommentVNode("v-if", true)
            ])
          ])
        ], 38 /* CLASS, STYLE, NEED_HYDRATION */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
