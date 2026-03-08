import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import { ref, computed } from 'vue'
import * as mfm from 'mfm-js'
import MkFolder from '@/components/MkFolder.vue'
import MkTabs from '@/components/MkTabs.vue'
import { extractUrlFromMfm } from '@/utility/extract-url-from-mfm'
import MkCode from '@/components/MkCode.vue'
import MkUrlPreview from '@/components/MkUrlPreview.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'qr.read.raw-viewer',
  props: {
    data: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const parsed = computed(() => mfm.parse(props.data));
const urls = computed(() => extractUrlFromMfm(parsed.value));
const tab = ref<'mfm' | 'raw'>('mfm');

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createBlock(MkFolder, {
      defaultOpen: "",
      withSpacer: false
    }, {
      label: _withCtx(() => [
        _createTextVNode(_toDisplayString(__props.data.split('\n')[0]), 1 /* TEXT */)
      ]),
      header: _withCtx(() => [
        _createVNode(MkTabs, {
          tabs: [
  				{
  					key: 'mfm',
  					title: _unref(i18n).ts._qr.mfm,
  					icon: 'ti ti-align-left',
  				},
  				{
  					key: 'raw',
  					title: _unref(i18n).ts._qr.raw,
  					icon: 'ti ti-code',
  				},
  			],
          tab: tab.value,
          "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
        }, null, 8 /* PROPS */, ["tabs", "tab"])
      ]),
      default: _withCtx(() => [
        _withDirectives(_createElementVNode("div", { class: "_spacer _gaps" }, [
          _createVNode(_component_Mfm, {
            text: __props.data,
            nyaize: false
          }, null, 8 /* PROPS */, ["text", "nyaize"]),
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(urls.value, (url) => {
            return (_openBlock(), _createBlock(MkUrlPreview, {
              key: url,
              url: url,
              compact: true,
              detail: false
            }, null, 8 /* PROPS */, ["url", "compact", "detail"]))
          }), 128 /* KEYED_FRAGMENT */))
        ], 512 /* NEED_PATCH */), [
          [_vShow, tab.value === 'mfm']
        ]),
        _withDirectives(_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 10px; --MI_SPACER-max: 16px;"
        }, [
          _createVNode(MkCode, {
            code: __props.data,
            lang: "text"
          }, null, 8 /* PROPS */, ["code"])
        ], 512 /* NEED_PATCH */), [
          [_vShow, tab.value === 'raw']
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["withSpacer"]))
}
}

})
