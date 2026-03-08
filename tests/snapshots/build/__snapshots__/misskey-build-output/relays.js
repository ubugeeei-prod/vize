import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'relays',
  setup(__props) {

const relays = ref<Misskey.entities.AdminRelaysListResponse>([]);
async function addRelay() {
	const { canceled, result: inbox } = await os.inputText({
		title: i18n.ts.addRelay,
		type: 'url',
		placeholder: i18n.ts.inboxUrl,
	});
	if (canceled || inbox == null) return;
	misskeyApi('admin/relays/add', {
		inbox,
	}).then(() => {
		refresh();
	}).catch((err: any) => {
		os.alert({
			type: 'error',
			text: err.message || err,
		});
	});
}
function remove(inbox: string) {
	misskeyApi('admin/relays/remove', {
		inbox,
	}).then(() => {
		refresh();
	}).catch((err: any) => {
		os.alert({
			type: 'error',
			text: err.message || err,
		});
	});
}
function refresh() {
	misskeyApi('admin/relays/list').then(relayList => {
		relays.value = relayList;
	});
}
refresh();
const headerActions = computed(() => [{
	asFullButton: true,
	icon: 'ti ti-plus',
	text: i18n.ts.addRelay,
	handler: addRelay,
}]);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.relays,
	icon: 'ti ti-planet',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(_component_SearchMarker, {
            path: "/admin/relays",
            label: _unref(i18n).ts.relays,
            keywords: ['relays'],
            icon: "ti ti-planet"
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(relays.value, (relay) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: relay.inbox,
                    class: "relaycxt _panel",
                    style: "padding: 16px;"
                  }, [
                    _createElementVNode("div", null, _toDisplayString(relay.inbox), 1 /* TEXT */),
                    _createElementVNode("div", { style: "margin: 8px 0;" }, [
                      (relay.status === 'accepted')
                        ? (_openBlock(), _createElementBlock("i", {
                          key: 0,
                          class: _normalizeClass(["ti ti-check", _ctx.$style.icon]),
                          style: "color: var(--MI_THEME-success);"
                        }))
                        : (relay.status === 'rejected')
                          ? (_openBlock(), _createElementBlock("i", {
                            key: 1,
                            class: _normalizeClass(["ti ti-ban", _ctx.$style.icon]),
                            style: "color: var(--MI_THEME-error);"
                          }))
                        : (_openBlock(), _createElementBlock("i", {
                          key: 2,
                          class: _normalizeClass(["ti ti-clock", _ctx.$style.icon])
                        })),
                      _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts._relayStatus[relay.status]), 1 /* TEXT */)
                    ]),
                    _createVNode(MkButton, {
                      class: "button",
                      inline: "",
                      danger: "",
                      onClick: _cache[0] || (_cache[0] = ($event: any) => (remove(relay.inbox)))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_1,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.remove), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    })
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["label", "keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
