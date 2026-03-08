import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, unref as _unref } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPreviewCard',
  props: {
    card: { type: null, required: true },
    smallPictureOnly: { type: Boolean, required: false },
    root: { type: Boolean, required: false }
  },
  setup(__props: any) {

const providerName = __props.card.providerName
const gitHubCards = usePreferences('experimentalGitHubCards')

return (_ctx: any,_cache: any) => {
  const _component_LazyStatusPreviewGitHub = _resolveComponent("LazyStatusPreviewGitHub")
  const _component_LazyStatusPreviewStackBlitz = _resolveComponent("LazyStatusPreviewStackBlitz")
  const _component_StatusPreviewCardNormal = _resolveComponent("StatusPreviewCardNormal")

  return (_unref(gitHubCards) && _unref(providerName) === 'GitHub')
      ? (_openBlock(), _createBlock(_component_LazyStatusPreviewGitHub, {
        key: 0,
        card: __props.card
      }, null, 8 /* PROPS */, ["card"]))
      : (_unref(gitHubCards) && _unref(providerName) === 'StackBlitz')
        ? (_openBlock(), _createBlock(_component_LazyStatusPreviewStackBlitz, {
          key: 1,
          card: __props.card,
          "small-picture-only": __props.smallPictureOnly,
          root: __props.root
        }, null, 8 /* PROPS */, ["card", "small-picture-only", "root"]))
      : (_openBlock(), _createBlock(_component_StatusPreviewCardNormal, {
        key: 2,
        card: __props.card,
        "small-picture-only": __props.smallPictureOnly,
        root: __props.root
      }, null, 8 /* PROPS */, ["card", "small-picture-only", "root"]))
}
}

})
