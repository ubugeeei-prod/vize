import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent } from "vue"

import type { CommonRouteTabOption } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountTabs',
  setup(__props) {

const { t } = useI18n()
const route = useRoute()
const server = computed(() => route.params.server as string)
const account = computed(() => route.params.account as string)
const tabs = computed<CommonRouteTabOption[]>(() => [
  {
    name: 'account-index',
    to: {
      name: 'account-index',
      params: { server: server.value, account: account.value },
    },
    display: t('tab.posts'),
    icon: 'i-ri:file-list-2-line',
  },
  {
    name: 'account-replies',
    to: {
      name: 'account-replies',
      params: { server: server.value, account: account.value },
    },
    display: t('tab.posts_with_replies'),
    icon: 'i-ri:chat-1-line',
  },
  {
    name: 'account-media',
    to: {
      name: 'account-media',
      params: { server: server.value, account: account.value },
    },
    display: t('tab.media'),
    icon: 'i-ri:camera-2-line',
  },
])

return (_ctx: any,_cache: any) => {
  const _component_CommonRouteTabs = _resolveComponent("CommonRouteTabs")

  return (_openBlock(), _createBlock(_component_CommonRouteTabs, {
      force: "",
      replace: "",
      options: tabs.value,
      "prevent-scroll-top": "",
      command: "",
      border: "base b"
    }, null, 8 /* PROPS */, ["options"]))
}
}

})
