import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "_monospace" }
const _hoisted_2 = { class: "_monospace" }
const _hoisted_3 = { class: "_monospace" }
const _hoisted_4 = { class: "_monospace" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { computed, defineAsyncComponent, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkObjectView from '@/components/MkObjectView.vue'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import FormSection from '@/components/form/section.vue'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import MkInfo from '@/components/MkInfo.vue'
import bytes from '@/filters/bytes.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { iAmAdmin, iAmModerator } from '@/i.js'
import MkTabs from '@/components/MkTabs.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'admin-file.root',
  props: {
    file: { type: null, required: true },
    info: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const tab = ref('overview');
const isSensitive = ref(props.file.isSensitive);
const usageTab = ref<'note' | 'chat'>('note');
const XNotes = defineAsyncComponent(() => import('./drive.file.notes.vue'));
const XChat = defineAsyncComponent(() => import('./admin-file.chat.vue'));
async function del() {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: props.file.name }),
	});
	if (canceled) return;
	os.apiWithDialog('drive/files/delete', {
		fileId: props.file.id,
	});
}
async function toggleSensitive() {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: isSensitive.value ? i18n.ts.unmarkAsSensitiveConfirm : i18n.ts.markAsSensitiveConfirm,
	});
	if (canceled) return;
	isSensitive.value = !isSensitive.value;
	os.apiWithDialog('drive/files/update', {
		fileId: props.file.id,
		isSensitive: !props.file.isSensitive,
	});
}
const headerActions = computed(() => [{
	text: i18n.ts.openInNewTab,
	icon: 'ti ti-external-link',
	handler: () => {
		window.open(props.file.url, '_blank', 'noopener');
	},
}]);
const headerTabs = computed(() => [{
	key: 'overview',
	title: i18n.ts.overview,
	icon: 'ti ti-info-circle',
}, iAmModerator ? {
	key: 'usage',
	title: i18n.ts._fileViewer.usage,
	icon: 'ti ti-plus',
} : null, iAmModerator ? {
	key: 'ip',
	title: 'IP',
	icon: 'ti ti-password',
} : null, {
	key: 'raw',
	title: 'Raw data',
	icon: 'ti ti-code',
}].filter(x => x != null));

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        (__props.file)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
          }, [
            (tab.value === 'overview')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "cxqhhsmd _gaps_m"
              }, [
                _createElementVNode("a", {
                  class: "thumbnail",
                  href: __props.file.url,
                  target: "_blank"
                }, [
                  _createVNode(MkDriveFileThumbnail, {
                    class: "thumbnail",
                    file: __props.file,
                    fit: "contain"
                  }, null, 8 /* PROPS */, ["file"])
                ], 8 /* PROPS */, ["href"]),
                _createElementVNode("div", null, [
                  _createVNode(MkKeyValue, {
                    copy: __props.file.type,
                    oneline: "",
                    style: "margin: 1em 0;"
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode("MIME Type")
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", _hoisted_1, _toDisplayString(__props.file.type), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["copy"]),
                  _createVNode(MkKeyValue, {
                    oneline: "",
                    style: "margin: 1em 0;"
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode("Size")
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", _hoisted_2, _toDisplayString(bytes(__props.file.size)), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkKeyValue, {
                    copy: __props.file.id,
                    oneline: "",
                    style: "margin: 1em 0;"
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode("ID")
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", _hoisted_3, _toDisplayString(__props.file.id), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["copy"]),
                  _createVNode(MkKeyValue, {
                    copy: __props.file.md5,
                    oneline: "",
                    style: "margin: 1em 0;"
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode("MD5")
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", _hoisted_4, _toDisplayString(__props.file.md5), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["copy"]),
                  _createVNode(MkKeyValue, {
                    oneline: "",
                    style: "margin: 1em 0;"
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.createdAt), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", { class: "_monospace" }, [
                        _createVNode(_component_MkTime, {
                          time: __props.file.createdAt,
                          mode: "detail",
                          style: "display: block;"
                        }, null, 8 /* PROPS */, ["time"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                (__props.file.user)
                  ? (_openBlock(), _createBlock(_component_MkA, {
                    key: 0,
                    class: "user",
                    to: `/admin/user/${__props.file.user.id}`
                  }, {
                    default: _withCtx(() => [
                      _createVNode(MkUserCardMini, { user: __props.file.user }, null, 8 /* PROPS */, ["user"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"]))
                  : _createCommentVNode("v-if", true),
                _createElementVNode("div", null, [
                  _createVNode(MkSwitch, {
                    modelValue: isSensitive.value,
                    "onUpdate:modelValue": toggleSensitive
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"])
                ]),
                _createElementVNode("div", null, [
                  _createVNode(MkButton, {
                    danger: "",
                    onClick: del
                  }, {
                    default: _withCtx(() => [
                      _hoisted_5,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]))
              : (tab.value === 'usage' && __props.info)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: "_gaps_m"
                }, [
                  _createVNode(MkTabs, {
                    tabs: [{
  					key: 'note',
  					title: 'Note',
  				}, {
  					key: 'chat',
  					title: 'Chat',
  				}],
                    tab: usageTab.value,
                    "onUpdate:tab": _cache[1] || (_cache[1] = ($event: any) => ((usageTab).value = $event))
                  }, null, 8 /* PROPS */, ["tabs", "tab"]),
                  (usageTab.value === 'note')
                    ? (_openBlock(), _createBlock(XNotes, {
                      key: 0,
                      fileId: props.file.id
                    }, null, 8 /* PROPS */, ["fileId"]))
                    : (usageTab.value === 'chat')
                      ? (_openBlock(), _createBlock(XChat, {
                        key: 1,
                        fileId: props.file.id
                      }, null, 8 /* PROPS */, ["fileId"]))
                    : _createCommentVNode("v-if", true)
                ]))
              : (tab.value === 'ip' && __props.info)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  class: "_gaps_m"
                }, [
                  (!_unref(iAmAdmin))
                    ? (_openBlock(), _createBlock(MkInfo, {
                      key: 0,
                      warn: ""
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.requireAdminForView), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }))
                    : _createCommentVNode("v-if", true),
                  (__props.info.requestIp)
                    ? (_openBlock(), _createBlock(MkKeyValue, {
                      key: 0,
                      class: "_monospace",
                      copy: __props.info.requestIp,
                      oneline: ""
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode("IP")
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(__props.info.requestIp), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["copy"]))
                    : _createCommentVNode("v-if", true),
                  (__props.info.requestHeaders)
                    ? (_openBlock(), _createBlock(FormSection, { key: 0 }, {
                      label: _withCtx(() => [
                        _createTextVNode("Headers")
                      ]),
                      default: _withCtx(() => [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.info.requestHeaders, (v, k) => {
                          return (_openBlock(), _createBlock(MkKeyValue, {
                            key: k,
                            class: "_monospace"
                          }, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(k), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createTextVNode(_toDisplayString(v), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 1024 /* DYNAMIC_SLOTS */))
                        }), 128 /* KEYED_FRAGMENT */))
                      ]),
                      _: 1 /* STABLE */
                    }))
                    : _createCommentVNode("v-if", true)
                ]))
              : (tab.value === 'raw')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 3,
                  class: "_gaps_m"
                }, [
                  (__props.info)
                    ? (_openBlock(), _createBlock(MkObjectView, {
                      key: 0,
                      tall: "",
                      value: __props.info
                    }, null, 8 /* PROPS */, ["value"]))
                    : _createCommentVNode("v-if", true)
                ]))
              : _createCommentVNode("v-if", true)
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
