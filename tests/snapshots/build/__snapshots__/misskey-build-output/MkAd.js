import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { ref, computed } from 'vue'
import { url as local, host } from '@@/js/config.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import MkButton from '@/components/MkButton.vue'
import { store } from '@/store.js'
import * as os from '@/os.js'
import { $i } from '@/i.js'
import { prefer } from '@/preferences.js'

type Ad = (typeof instance)['ads'][number];

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAd',
  props: {
    preferForms: { type: Array, required: false },
    specify: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const showMenu = ref(false);
const toggleMenu = (): void => {
	showMenu.value = !showMenu.value;
};
const choseAd = (): Ad | null => {
	if (props.specify) {
		return props.specify;
	}

	const allAds = instance.ads.map(ad => store.s.mutedAds.includes(ad.id) ? {
		...ad,
		ratio: 0,
	} : ad);

	let ads = props.preferForms ? allAds.filter(ad => props.preferForms!.includes(ad.place)) : allAds;

	if (ads.length === 0) {
		ads = allAds.filter(ad => ad.place === 'square');
	}

	const lowPriorityAds = ads.filter(ad => ad.ratio === 0);
	ads = ads.filter(ad => ad.ratio !== 0);

	if (ads.length === 0) {
		if (lowPriorityAds.length !== 0) {
			return lowPriorityAds[Math.floor(Math.random() * lowPriorityAds.length)];
		} else {
			return null;
		}
	}

	const totalFactor = ads.reduce((a, b) => a + b.ratio, 0);
	const r = Math.random() * totalFactor;

	let stackedFactor = 0;
	for (const ad of ads) {
		if (r >= stackedFactor && r <= stackedFactor + ad.ratio) {
			return ad;
		} else {
			stackedFactor += ad.ratio;
		}
	}

	return null;
};
const chosen = ref(choseAd());
const self = computed(() => chosen.value?.url.startsWith(local));
const shouldHide = ref(!prefer.s.forceShowAds && $i && $i.policies.canHideAds && (props.specify == null));
function reduceFrequency(): void {
	if (chosen.value == null) return;
	if (store.s.mutedAds.includes(chosen.value.id)) return;
	store.push('mutedAds', chosen.value.id);
	os.success();
	chosen.value = choseAd();
	showMenu.value = false;
}

return (_ctx: any,_cache: any) => {
  return (chosen.value && !shouldHide.value)
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (!showMenu.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.main, {
  			[_ctx.$style.form_square]: chosen.value.place === 'square',
  			[_ctx.$style.form_horizontal]: chosen.value.place === 'horizontal',
  			[_ctx.$style.form_horizontalBig]: chosen.value.place === 'horizontal-big',
  			[_ctx.$style.form_vertical]: chosen.value.place === 'vertical',
  		}])
          }, [ _createVNode(_resolveDynamicComponent(self.value ? 'MkA' : 'a'), _mergeProps(self.value ? {
  				to: chosen.value.url.substring(_unref(local).length),
  			} : {
  				href: chosen.value.url,
  				rel: 'nofollow noopener',
  				target: '_blank',
  			}, {
              class: _ctx.$style.link
            }), {
              default: _withCtx(() => [
                _createElementVNode("img", {
                  src: chosen.value.imageUrl,
                  class: _normalizeClass(_ctx.$style.img)
                }, null, 8 /* PROPS */, ["src"]),
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.i]),
                  onClick: _withModifiers(toggleMenu, ["prevent","stop"])
                }, [
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-info-circle", _ctx.$style.iIcon])
                  })
                ])
              ]),
              _: 1 /* STABLE */
            }, 16 /* FULL_PROPS */) ])) : (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: _normalizeClass(_ctx.$style.menu)
          }, [ _createElementVNode("div", null, "Ads by " + _toDisplayString(_unref(host)), 1 /* TEXT */), _createTextVNode("\n\t\t"), _createTextVNode("\n\t\t"), (chosen.value.ratio !== 0) ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                class: _normalizeClass(_ctx.$style.menuButton),
                onClick: reduceFrequency
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._ad.reduceFrequencyOfThisAd), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })) : _createCommentVNode("v-if", true), _createElementVNode("button", {
              class: "_textButton",
              onClick: toggleMenu
            }, _toDisplayString(_unref(i18n).ts._ad.back), 1 /* TEXT */) ])) ]))
      : _createCommentVNode("v-if", true)
}
}

})
