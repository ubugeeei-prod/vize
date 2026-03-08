import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, resolveDirective as _resolveDirective, withDirectives as _withDirectives, normalizeClass as _normalizeClass } from "vue"

import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserOnlineIndicator',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const text = computed(() => {
	switch (props.user.onlineStatus) {
		case 'online': return i18n.ts.online;
		case 'active': return i18n.ts.active;
		case 'offline': return i18n.ts.offline;
		case 'unknown': return i18n.ts.unknown;
	}
});

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, {
  		[_ctx.$style.status_online]: __props.user.onlineStatus === 'online',
  		[_ctx.$style.status_active]: __props.user.onlineStatus === 'active',
  		[_ctx.$style.status_offline]: __props.user.onlineStatus === 'offline',
  		[_ctx.$style.status_unknown]: __props.user.onlineStatus === 'unknown',
  	}])
    }, null, 2 /* CLASS */)), [ [_directive_tooltip, text.value] ])
}
}

})
