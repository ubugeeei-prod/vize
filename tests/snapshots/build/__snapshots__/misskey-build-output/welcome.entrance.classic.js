import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "_monospace" }
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import XTimeline from './welcome.timeline.vue'
import MkMarqueeText from '@/components/MkMarqueeText.vue'
import MkFeaturedPhotos from '@/components/MkFeaturedPhotos.vue'
import misskeysvg from '/client-assets/misskey.svg'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import MkVisitorDashboard from '@/components/MkVisitorDashboard.vue'
import { getProxiedImageUrl } from '@/utility/media-proxy.js'
import { instance as meta } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'welcome.entrance.classic',
  setup(__props) {

const instances = ref<Misskey.entities.FederationInstance[]>();
function getInstanceIcon(instance: Misskey.entities.FederationInstance): string {
	if (!instance.iconUrl) {
		return '';
	}
	return getProxiedImageUrl(instance.iconUrl, 'preview');
}
misskeyApiGet('federation/instances', {
	sort: '+pubSub',
	limit: 20,
	blocked: false,
}).then(_instances => {
	instances.value = _instances;
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_unref(meta))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createVNode(MkFeaturedPhotos, {
          class: _normalizeClass(_ctx.$style.bg)
        }), _createVNode(XTimeline, {
          class: _normalizeClass(_ctx.$style.tl)
        }), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.shape1)
        }), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.shape2)
        }), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.logoWrapper)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.poweredBy)
          }, "Powered by"), _createElementVNode("img", {
            src: misskeysvg,
            class: _normalizeClass(_ctx.$style.misskey)
          }, null, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.contents)
        }, [ _createVNode(MkVisitorDashboard) ]), (instances.value && instances.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.federation)
          }, [ _createVNode(MkMarqueeText, { duration: 40 }, {
              default: _withCtx(() => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(instances.value, (instance) => {
                  return (_openBlock(), _createBlock(_component_MkA, {
                    key: instance.id,
                    class: _normalizeClass(_ctx.$style.federationInstance),
                    to: `/instance-info/${instance.host}`,
                    behavior: "window"
                  }, {
                    default: _withCtx(() => [
                      (instance.iconUrl)
                        ? (_openBlock(), _createElementBlock("img", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.federationInstanceIcon),
                          src: getInstanceIcon(instance),
                          alt: ""
                        }))
                        : _createCommentVNode("v-if", true),
                      _createElementVNode("span", _hoisted_1, _toDisplayString(instance.host), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["duration"]) ])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
