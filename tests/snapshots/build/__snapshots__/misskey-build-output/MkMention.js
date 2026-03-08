import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { toUnicode } from 'punycode.js'
import { computed } from 'vue'
import { host as localHost } from '@@/js/config.js'
import type { MkABehavior } from '@/components/global/MkA.vue'
import { $i } from '@/i.js'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMention',
  props: {
    username: { type: String, required: true },
    host: { type: String, required: true },
    navigationBehavior: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const canonical = props.host === localHost ? `@${props.username}` : `@${props.username}@${toUnicode(props.host)}`;
const url = `/${canonical}`;
const isMe = $i && (
	`@${props.username}@${toUnicode(props.host)}` === `@${$i.username}@${toUnicode(localHost)}`.toLowerCase()
);
const avatarUrl = computed(() => prefer.s.disableShowingAnimatedImages || prefer.s.dataSaver.avatar
	? getStaticImageUrl(`/avatar/@${props.username}@${props.host}`)
	: `/avatar/@${props.username}@${props.host}`,
);

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _directive_user_preview = _resolveDirective("user-preview")

  return _withDirectives((_openBlock(), _createBlock(_component_MkA, {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.isMe]: _unref(isMe) }]),
      to: _unref(url),
      behavior: __props.navigationBehavior
    }, {
      default: _withCtx(() => [
        _createElementVNode("img", {
          class: _normalizeClass(_ctx.$style.icon),
          src: avatarUrl.value,
          alt: ""
        }, null, 8 /* PROPS */, ["src"]),
        _createElementVNode("span", null, [
          _createElementVNode("span", null, "@" + _toDisplayString(__props.username), 1 /* TEXT */),
          ((__props.host != _unref(localHost)))
            ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: _normalizeClass(_ctx.$style.host)
            }, "@" + _toDisplayString(_unref(toUnicode)(__props.host)), 1 /* TEXT */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["to", "behavior"])), [ [_directive_user_preview, _unref(canonical)] ])
}
}

})
