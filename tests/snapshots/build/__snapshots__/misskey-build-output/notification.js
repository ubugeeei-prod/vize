import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass } from "vue"

import * as Misskey from 'misskey-js'
import XNotification from '@/components/MkNotification.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'notification',
  props: {
    notification: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(XNotification, {
        notification: __props.notification,
        class: "notification _acrylic",
        full: false
      }, null, 8 /* PROPS */, ["notification", "full"]) ]))
}
}

})
