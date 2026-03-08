import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link ti-fw" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-share ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots ti-fw" })
const _hoisted_7 = { class: "title" }
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
import { computed, onDeactivated, onUnmounted, ref, watch, shallowRef, defineAsyncComponent } from 'vue'
import * as Misskey from 'misskey-js'
import { utils } from '@syuilo/aiscript'
import { compareVersions } from 'compare-versions'
import { url } from '@@/js/config.js'
import type { Ref } from 'vue'
import type { AsUiComponent, AsUiRoot } from '@/aiscript/ui.js'
import type { MenuItem } from '@/types/menu.js'
import type { Interpreter } from '@syuilo/aiscript'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkAsUi from '@/components/MkAsUi.vue'
import { registerAsUiLib } from '@/aiscript/ui.js'
import { aiScriptReadline, createAiScriptEnv } from '@/aiscript/api.js'
import MkFolder from '@/components/MkFolder.vue'
import MkCode from '@/components/MkCode.vue'
import { prefer } from '@/preferences.js'
import { $i } from '@/i.js'
import { isSupportShare } from '@/utility/navigator.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { pleaseLogin } from '@/utility/please-login.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'flash',
  props: {
    id: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const flash = ref<Misskey.entities.Flash | null>(null);
const error = ref<any>(null);
function fetchFlash() {
	flash.value = null;
	misskeyApi('flash/show', {
		flashId: props.id,
	}).then(_flash => {
		flash.value = _flash;
	}).catch(err => {
		error.value = err;
	});
}
function share(ev: PointerEvent) {
	if (!flash.value) return;
	const menuItems: MenuItem[] = [];
	menuItems.push({
		text: i18n.ts.shareWithNote,
		icon: 'ti ti-pencil',
		action: shareWithNote,
	});
	if (isSupportShare()) {
		menuItems.push({
			text: i18n.ts.share,
			icon: 'ti ti-share',
			action: shareWithNavigator,
		});
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
function copyLink() {
	if (!flash.value) return;
	copyToClipboard(`${url}/play/${flash.value.id}`);
}
function shareWithNavigator() {
	if (!flash.value) return;
	navigator.share({
		title: flash.value.title,
		text: flash.value.summary,
		url: `${url}/play/${flash.value.id}`,
	});
}
function shareWithNote() {
	if (!flash.value) return;
	os.post({
		initialText: `${flash.value.title}\n${url}/play/${flash.value.id}`,
		instant: true,
	});
}
async function like() {
	if (!flash.value) return;
	const isLoggedIn = await pleaseLogin();
	if (!isLoggedIn) return;
	os.apiWithDialog('flash/like', {
		flashId: flash.value.id,
	}).then(() => {
		flash.value!.isLiked = true;
		flash.value!.likedCount++;
	});
}
async function unlike() {
	if (!flash.value) return;
	const isLoggedIn = await pleaseLogin();
	if (!isLoggedIn) return;
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unlikeConfirm,
	});
	if (confirm.canceled) return;
	os.apiWithDialog('flash/unlike', {
		flashId: flash.value.id,
	}).then(() => {
		flash.value!.isLiked = false;
		flash.value!.likedCount--;
	});
}
watch(() => props.id, fetchFlash, { immediate: true });
const started = ref(false);
const aiscript = shallowRef<Interpreter | null>(null);
const root = ref<AsUiRoot>();
const components = ref<Ref<AsUiComponent>[]>([]);
function start() {
	started.value = true;
	run();
}
function getIsLegacy(version: string | null): boolean {
	if (version == null) return true;
	try {
		return compareVersions(version, '1.0.0') < 0;
	} catch {
		return false;
	}
}
async function run() {
	if (aiscript.value) aiscript.value.abort();
	if (!flash.value) return;
	const version = utils.getLangVersion(flash.value.script);
	const isLegacy = getIsLegacy(version);
	const { Interpreter, Parser, values } = (isLegacy ? (await import('@syuilo/aiscript-0-19-0')) : await import('@syuilo/aiscript')) as typeof import('@syuilo/aiscript');
	const parser = new Parser();
	components.value = [];
	const interpreter = new Interpreter({
		...createAiScriptEnv({
			storageKey: 'flash:' + flash.value.id,
		}),
		...registerAsUiLib(components.value, (_root) => {
			root.value = _root.value;
		}),
		THIS_ID: values.STR(flash.value.id),
		THIS_URL: values.STR(`${url}/play/${flash.value.id}`),
	}, {
		in: aiScriptReadline,
		out: () => {
			// nop
		},
		log: () => {
			// nop
		},
	});
	aiscript.value = interpreter;
	let ast;
	try {
		ast = parser.parse(flash.value.script);
	} catch (err) {
		os.alert({
			type: 'error',
			text: 'Syntax error :(',
		});
		return;
	}
	try {
		await interpreter.exec(ast);
	} catch (err: any) {
		os.alert({
			type: 'error',
			title: 'AiScript Error',
			text: err.message,
		});
	}
}
async function reportAbuse() {
	if (!flash.value) return;
	const pageUrl = `${url}/play/${flash.value.id}`;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkAbuseReportWindow.vue').then(x => x.default), {
		user: flash.value.user,
		initialComment: `Play: ${pageUrl}\n-----\n`,
	}, {
		closed: () => dispose(),
	});
}
function showMenu(ev: PointerEvent) {
	if (!flash.value) return;
	const menu: MenuItem[] = [
		...($i && $i.id !== flash.value.userId ? [
			{
				icon: 'ti ti-exclamation-circle',
				text: i18n.ts.reportAbuse,
				action: reportAbuse,
			},
			...($i.isModerator || $i.isAdmin ? [
				{
					type: 'divider' as const,
				},
				{
					icon: 'ti ti-trash',
					text: i18n.ts.delete,
					danger: true,
					action: () => os.confirm({
						type: 'warning',
						text: i18n.ts.deleteConfirm,
					}).then(({ canceled }) => {
						if (canceled || !flash.value) return;
						os.apiWithDialog('flash/delete', { flashId: flash.value.id });
					}),
				},
			] : []),
		] : []),
	];
	os.popupMenu(menu, ev.currentTarget ?? ev.target);
}
function reset() {
	if (aiscript.value) aiscript.value.abort();
	started.value = false;
}
onDeactivated(() => {
	reset();
});
onUnmounted(() => {
	reset();
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: flash.value ? flash.value.title : 'Play',
	...flash.value ? {
		avatar: flash.value.user,
		path: `/play/${flash.value.id}`,
		share: {
			title: flash.value.title,
			text: flash.value.summary,
		},
	} : {},
}));

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkAd = _resolveComponent("MkAd")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          _createVNode(_Transition, {
            name: _unref(prefer).s.animation ? 'fade' : '',
            mode: "out-in"
          }, {
            default: _withCtx(() => [
              (flash.value)
                ? (_openBlock(), _createElementBlock("div", { key: flash.value.id }, [
                  _createVNode(_Transition, {
                    name: _unref(prefer).s.animation ? 'zoom' : '',
                    mode: "out-in"
                  }, {
                    default: _withCtx(() => [
                      (started.value)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.started)
                        }, [
                          _createElementVNode("div", { class: "main _panel" }, [
                            (root.value)
                              ? (_openBlock(), _createBlock(MkAsUi, {
                                key: 0,
                                component: root.value,
                                components: components.value
                              }, null, 8 /* PROPS */, ["component", "components"]))
                              : _createCommentVNode("v-if", true)
                          ]),
                          _createElementVNode("div", { class: "actions _panel" }, [
                            _createElementVNode("div", { class: "items" }, [
                              _createVNode(MkButton, {
                                class: "button",
                                rounded: "",
                                onClick: reset
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_1
                                ]),
                                _: 1 /* STABLE */
                              })
                            ]),
                            _createElementVNode("div", { class: "items" }, [
                              (flash.value.isLiked)
                                ? (_openBlock(), _createBlock(MkButton, {
                                  key: 0,
                                  asLike: "",
                                  class: "button",
                                  rounded: "",
                                  primary: "",
                                  onClick: _cache[0] || (_cache[0] = ($event: any) => (unlike()))
                                }, {
                                  default: _withCtx(() => [
                                    _hoisted_2,
                                    (flash.value?.likedCount && flash.value.likedCount > 0)
                                      ? (_openBlock(), _createElementBlock("span", {
                                        key: 0,
                                        style: "margin-left: 6px;"
                                      }, _toDisplayString(flash.value.likedCount), 1 /* TEXT */))
                                      : _createCommentVNode("v-if", true)
                                  ]),
                                  _: 1 /* STABLE */
                                }))
                                : (_openBlock(), _createBlock(MkButton, {
                                  key: 1,
                                  asLike: "",
                                  class: "button",
                                  rounded: "",
                                  onClick: _cache[1] || (_cache[1] = ($event: any) => (like()))
                                }, {
                                  default: _withCtx(() => [
                                    _hoisted_3,
                                    (flash.value?.likedCount && flash.value.likedCount > 0)
                                      ? (_openBlock(), _createElementBlock("span", {
                                        key: 0,
                                        style: "margin-left: 6px;"
                                      }, _toDisplayString(flash.value.likedCount), 1 /* TEXT */))
                                      : _createCommentVNode("v-if", true)
                                  ]),
                                  _: 1 /* STABLE */
                                })),
                              _createVNode(MkButton, {
                                class: "button",
                                rounded: "",
                                onClick: copyLink
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_4
                                ]),
                                _: 1 /* STABLE */
                              }),
                              _createVNode(MkButton, {
                                class: "button",
                                rounded: "",
                                onClick: share
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_5
                                ]),
                                _: 1 /* STABLE */
                              }),
                              (_unref($i) && _unref($i).id !== flash.value.user.id)
                                ? (_openBlock(), _createBlock(MkButton, {
                                  key: 0,
                                  class: "button",
                                  rounded: "",
                                  onMousedown: showMenu
                                }, {
                                  default: _withCtx(() => [
                                    _hoisted_6
                                  ]),
                                  _: 1 /* STABLE */
                                }))
                                : _createCommentVNode("v-if", true)
                            ])
                          ])
                        ]))
                        : (_openBlock(), _createElementBlock("div", {
                          key: 1,
                          class: _normalizeClass(_ctx.$style.ready)
                        }, [
                          _createElementVNode("div", { class: "_panel main" }, [
                            _createElementVNode("div", _hoisted_7, _toDisplayString(flash.value.title), 1 /* TEXT */),
                            _createElementVNode("div", { class: "summary" }, [
                              _createVNode(_component_Mfm, { text: flash.value.summary }, null, 8 /* PROPS */, ["text"])
                            ]),
                            _createVNode(MkButton, {
                              class: "start",
                              gradate: "",
                              rounded: "",
                              large: "",
                              onClick: start
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode("Play")
                              ]),
                              _: 1 /* STABLE */
                            }),
                            _createElementVNode("div", { class: "info" }, [
                              _createElementVNode("span", null, [
                                _hoisted_8,
                                _createTextVNode(" " + _toDisplayString(flash.value.likedCount), 1 /* TEXT */)
                              ])
                            ])
                          ])
                        ]))
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["name"]),
                  _createVNode(MkFolder, {
                    defaultOpen: false,
                    "max-height": 280,
                    class: "_margin"
                  }, {
                    icon: _withCtx(() => [
                      _hoisted_9
                    ]),
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._play.viewSource), 1 /* TEXT */)
                    ]),
                    default: _withCtx(() => [
                      _createVNode(MkCode, {
                        code: flash.value.script,
                        lang: "is",
                        class: "_monospace"
                      }, null, 8 /* PROPS */, ["code"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["defaultOpen", "max-height"]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.footer)
                  }, [
                    _createVNode(_component_Mfm, { text: `By @${flash.value.user.username}` }, null, 8 /* PROPS */, ["text"]),
                    _createElementVNode("div", { class: "date" }, [
                      (flash.value.createdAt != flash.value.updatedAt)
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                          _hoisted_10,
                          _createTextVNode(),
                          _toDisplayString(_unref(i18n).ts.updatedAt),
                          _createTextVNode(": "),
                          _createVNode(_component_MkTime, {
                            time: flash.value.updatedAt,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ]))
                        : _createCommentVNode("v-if", true),
                      _createElementVNode("div", null, [
                        _hoisted_11,
                        _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.createdAt) + ": ", 1 /* TEXT */),
                        _createVNode(_component_MkTime, {
                          time: flash.value.createdAt,
                          mode: "detail"
                        }, null, 8 /* PROPS */, ["time"])
                      ])
                    ])
                  ]),
                  (_unref($i) && _unref($i).id === flash.value.userId)
                    ? (_openBlock(), _createBlock(_component_MkA, {
                      key: 0,
                      to: `/play/${flash.value.id}/edit`,
                      style: "color: var(--MI_THEME-accent);"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._play.editThisPage), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]))
                    : _createCommentVNode("v-if", true),
                  _createVNode(_component_MkAd, { preferForms: ['horizontal', 'horizontal-big'] }, null, 8 /* PROPS */, ["preferForms"])
                ]))
                : (error.value)
                  ? (_openBlock(), _createBlock(_component_MkError, {
                    key: 1,
                    onRetry: _cache[2] || (_cache[2] = ($event: any) => (fetchFlash()))
                  }))
                : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["name"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
