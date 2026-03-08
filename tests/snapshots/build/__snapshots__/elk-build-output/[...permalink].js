import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

import { hasProtocol, parseURL } from 'ufo'

export default /*@__PURE__*/_defineComponent({
  __name: '[...permalink]',
  setup(__props) {

definePageMeta({
  middleware: async (to) => {
    const permalink = Array.isArray(to.params.permalink)
      ? to.params.permalink.join('/')
      : to.params.permalink
    if (hasProtocol(permalink)) {
      const { host, pathname } = parseURL(permalink)
      if (host)
        return `/${host}${pathname}`
    }
    // We've reached a page that doesn't exist
    return false
  },
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div"))
}
}

})
