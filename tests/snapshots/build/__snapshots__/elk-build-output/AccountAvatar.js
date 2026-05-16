import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, mergeProps as _mergeProps } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountAvatar',
  props: {
    account: { type: null, required: true },
    square: { type: Boolean, required: false }
  },
  setup(__props: any) {

const loaded = ref(false)
const error = ref(false)
const preferredMotion = usePreferredReducedMotion()
const accountAvatarSrc = computed(() => {
  return preferredMotion.value === 'reduce' ? (__props.account?.avatarStatic ?? __props.account.avatar) : __props.account.avatar
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("img", _mergeProps(_ctx.$attrs, {
      key: __props.account.avatar,
      width: "400",
      height: "400",
      "select-none": "",
      src: (error.value || !loaded.value) ? 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7' : accountAvatarSrc.value,
      alt: _ctx.$t('account.avatar_description', [__props.account.username]),
      loading: "lazy",
      class: ["account-avatar object-cover", (loaded.value ? 'bg-base' : 'bg-gray:10') + (__props.square ? ' ' : ' rounded-full')],
      style: { 'clip-path': __props.square ? `url(#avatar-mask)` : 'none' },
      onLoad: _cache[0] || (_cache[0] = ($event: any) => (loaded.value = true)),
      onError: _cache[1] || (_cache[1] = ($event: any) => (error.value = true))
    }), null, 48 /* FULL_PROPS, NEED_HYDRATION */, ["src", "alt"]))
}
}

})
