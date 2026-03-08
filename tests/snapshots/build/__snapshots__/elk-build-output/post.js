import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'post',
  setup(__props) {

const router = useRouter()
const route = useRoute()
onMounted(async () => {
  // TODO: login check
  await openPublishDialog('intent', getDefaultDraftItem({
    status: route.query.text as string,
    sensitive: route.query.sensitive === 'true' || route.query.sensitive === null,
    spoilerText: route.query.spoiler_text as string,
    visibility: route.query.visibility as mastodon.v1.StatusVisibility,
    language: route.query.language as string,
    quotedStatusId: route.query.quote as string,
  }), true)
  // TODO: need a better idea 👀
  await router.replace('/home')
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _renderSlot(_ctx.$slots, "default") ]))
}
}

})
