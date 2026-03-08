import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
import { onUnmounted, ref } from 'vue'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { prefer } from '@/preferences.js'
import { store } from '@/store.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'stream-indicator',
  setup(__props) {

const zIndex = os.claimZIndex('high');
const hasDisconnected = ref(false);
function onDisconnected() {
	hasDisconnected.value = true;
}
function resetDisconnected() {
	hasDisconnected.value = false;
}
function reload() {
	window.location.reload();
}
if (store.s.realtimeMode) {
	useStream().on('_disconnected_', onDisconnected);
	onUnmounted(() => {
		useStream().off('_disconnected_', onDisconnected);
	});
}

return (_ctx: any,_cache: any) => {
  return (hasDisconnected.value && _unref(prefer).s.serverDisconnectedBehavior === 'quiet')
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(["_panel _shadow", _ctx.$style.root]),
        onClick: resetDisconnected
      }, [ _createElementVNode("div", null, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.disconnectedFromServer), 1 /* TEXT */) ]), _createElementVNode("div", {
          class: _normalizeClass(["_buttons", _ctx.$style.command])
        }, [ _createVNode(MkButton, {
            small: "",
            primary: "",
            onClick: reload
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.reload), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }), _createVNode(MkButton, { small: "" }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.doNothing), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
