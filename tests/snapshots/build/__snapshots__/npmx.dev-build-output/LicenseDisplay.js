import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString } from "vue"

import { parseLicenseExpression } from '#shared/utils/spdx'

export default /*@__PURE__*/_defineComponent({
  __name: 'LicenseDisplay',
  props: {
    license: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const tokens = computed(() => parseLicenseExpression(props.license))
const hasAnyValidLicense = computed(() => tokens.value.some(t => t.type === 'license' && t.url))

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", { class: "inline-flex items-baseline gap-x-1.5 flex-wrap gap-y-0.5" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tokens.value, (token, i) => {
        return (_openBlock(), _createElementBlock(_Fragment, { key: i }, [
          (token.type === 'license' && token.url)
            ? (_openBlock(), _createElementBlock("a", {
              key: 0,
              href: token.url,
              target: "_blank",
              rel: "noopener noreferrer",
              class: "link-subtle",
              title: _ctx.$t('package.license.view_spdx')
            }, _toDisplayString(token.value), 1 /* TEXT */))
            : (token.type === 'license')
              ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(token.value), 1 /* TEXT */))
            : (token.type === 'operator')
              ? (_openBlock(), _createElementBlock("span", {
                key: 2,
                class: "text-4xs"
              }, _toDisplayString(token.value), 1 /* TEXT */))
            : _createCommentVNode("v-if", true)
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */)), (hasAnyValidLicense.value) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          class: "i-lucide:scale w-3.5 h-3.5 text-fg-subtle flex-shrink-0 self-center",
          "aria-hidden": "true"
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
