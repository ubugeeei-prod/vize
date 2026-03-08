import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import { version } from '@@/js/config.js'
import MkButton from '@/components/MkButton.vue'
import MkLink from '@/components/MkLink.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { unisonReload } from '@/utility/unison-reload.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { miLocalStorage } from '@/local-storage.js'
import { prefer } from '@/preferences.js'
import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: '_error_',
  props: {
    error: { type: Error, required: false }
  },
  setup(__props: any) {

const props = __props
const loaded = ref(false);
const serverIsDead = ref(false);
const meta = ref<Misskey.entities.MetaResponse | null>(null);
misskeyApi('meta', {
	detail: false,
}).then(res => {
	loaded.value = true;
	serverIsDead.value = false;
	meta.value = res;
	miLocalStorage.setItem('v', res.version);
}, () => {
	loaded.value = true;
	serverIsDead.value = true;
});
function reload() {
	unisonReload();
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.error,
	icon: 'ti ti-alert-triangle',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ (!loaded.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true), _createVNode(_Transition, {
        name: _unref(prefer).s.animation ? '_transition_zoom' : '',
        appear: ""
      }, {
        default: _withCtx(() => [
          _withDirectives(_createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.root)
          }, [
            (_unref(instance).serverErrorImageUrl)
              ? (_openBlock(), _createElementBlock("img", {
                key: 0,
                src: _unref(instance).serverErrorImageUrl,
                draggable: "false",
                class: _normalizeClass(_ctx.$style.img)
              }))
              : _createCommentVNode("v-if", true),
            _createElementVNode("div", { class: "_gaps" }, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, [
                  _hoisted_1,
                  _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.pageLoadError), 1 /* TEXT */)
                ])
              ]),
              (meta.value && (_unref(version) === meta.value.version))
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts.pageLoadErrorDescription), 1 /* TEXT */))
                : (serverIsDead.value)
                  ? (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts.serverIsDead), 1 /* TEXT */))
                : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [
                  _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.newVersionOfClientAvailable), 1 /* TEXT */),
                  _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.youShouldUpgradeClient), 1 /* TEXT */),
                  _createVNode(MkButton, {
                    style: "margin: 8px auto;",
                    onClick: reload
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.reload), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ], 64 /* STABLE_FRAGMENT */)),
              _createElementVNode("div", null, [
                _createVNode(MkLink, {
                  url: "https://misskey-hub.net/docs/for-users/resources/troubleshooting/",
                  target: "_blank"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.troubleshooting), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              (__props.error)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  style: "opacity: 0.7;"
                }, "ERROR: " + _toDisplayString(__props.error), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ])
          ], 512 /* NEED_PATCH */), [
            [_vShow, loaded.value]
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["name"]) ], 64 /* STABLE_FRAGMENT */))
}
}

})
