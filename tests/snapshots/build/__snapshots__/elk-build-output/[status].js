import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: '[status]',
  setup(__props) {

definePageMeta({
  name: 'status-by-id',
  middleware: async (to) => {
    const params = to.params
    const id = params.status as string
    const status = await fetchStatus(id)
    return getStatusRoute(status)
  },
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div"))
}
}

})
