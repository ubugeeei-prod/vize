import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "color: var(--MI_THEME-accent)" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-circle-x" })
import { ref, computed, nextTick } from 'vue'
import type { Extension } from '@/components/MkExtensionInstaller.vue'
import type { AiScriptPluginMeta } from '@/plugin.js'
import MkLoading from '@/components/global/MkLoading.vue'
import MkExtensionInstaller from '@/components/MkExtensionInstaller.vue'
import MkButton from '@/components/MkButton.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkUrl from '@/components/global/MkUrl.vue'
import FormSection from '@/components/form/section.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { parsePluginMeta, installPlugin } from '@/plugin.js'
import { parseThemeCode, installTheme } from '@/theme.js'
import { unisonReload } from '@/utility/unison-reload.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'install-extensions',
  setup(__props) {

const uiPhase = ref<'fetching' | 'confirm' | 'error'>('fetching');
const errorKV = ref<{
	title?: string;
	description?: string;
}>({
	title: '',
	description: '',
});
const url = ref<string | null>(null);
const hash = ref<string | null>(null);
const data = ref<Extension | null>(null);
function close_(): void {
	if (window.history.length === 1) {
		window.close();
	} else {
		window.history.back();
	}
}
async function _fetch_() {
	if (!url.value || !hash.value) {
		errorKV.value = {
			title: i18n.ts._externalResourceInstaller._errors._invalidParams.title,
			description: i18n.ts._externalResourceInstaller._errors._invalidParams.description,
		};
		uiPhase.value = 'error';
		return;
	}
	const res = await misskeyApi('fetch-external-resources', {
		url: url.value,
		hash: hash.value,
	}).catch((err) => {
		switch (err.id) {
			case 'bb774091-7a15-4a70-9dc5-6ac8cf125856':
				errorKV.value = {
					title: i18n.ts._externalResourceInstaller._errors._failedToFetch.title,
					description: i18n.ts._externalResourceInstaller._errors._failedToFetch.parseErrorDescription,
				};
				uiPhase.value = 'error';
				break;
			case '693ba8ba-b486-40df-a174-72f8279b56a4':
				errorKV.value = {
					title: i18n.ts._externalResourceInstaller._errors._hashUnmatched.title,
					description: i18n.ts._externalResourceInstaller._errors._hashUnmatched.description,
				};
				uiPhase.value = 'error';
				break;
			default:
				errorKV.value = {
					title: i18n.ts._externalResourceInstaller._errors._failedToFetch.title,
					description: i18n.ts._externalResourceInstaller._errors._failedToFetch.fetchErrorDescription,
				};
				uiPhase.value = 'error';
				break;
		}
		throw new Error(err.code);
	});
	if (!res) {
		errorKV.value = {
			title: i18n.ts._externalResourceInstaller._errors._failedToFetch.title,
			description: i18n.ts._externalResourceInstaller._errors._failedToFetch.fetchErrorDescription,
		};
		uiPhase.value = 'error';
		return;
	}
	switch (res.type) {
		case 'plugin':
			try {
				const meta = await parsePluginMeta(res.data);
				data.value = {
					type: 'plugin',
					meta,
					raw: res.data,
				};
			} catch (err) {
				errorKV.value = {
					title: i18n.ts._externalResourceInstaller._errors._pluginParseFailed.title,
					description: i18n.ts._externalResourceInstaller._errors._pluginParseFailed.description,
				};
				console.error(err);
				uiPhase.value = 'error';
				return;
			}
			break;
		case 'theme':
			try {
				const metaRaw = parseThemeCode(res.data);
				const { id, props, desc: description, ...meta } = metaRaw;
				data.value = {
					type: 'theme',
					meta: {
						// description, // 使用されていない
						...meta,
					},
					raw: res.data,
				};
			} catch (err) {
				if (!(err instanceof Error)) {
					throw err;
				}
				switch (err.message.toLowerCase()) {
					case 'this theme is already installed':
						errorKV.value = {
							title: i18n.ts._externalResourceInstaller._errors._themeParseFailed.title,
							description: i18n.ts._theme.alreadyInstalled,
						};
						break;
					default:
						errorKV.value = {
							title: i18n.ts._externalResourceInstaller._errors._themeParseFailed.title,
							description: i18n.ts._externalResourceInstaller._errors._themeParseFailed.description,
						};
						break;
				}
				console.error(err);
				uiPhase.value = 'error';
				return;
			}
			break;
		default:
			errorKV.value = {
				title: i18n.ts._externalResourceInstaller._errors._resourceTypeNotSupported.title,
				description: i18n.ts._externalResourceInstaller._errors._resourceTypeNotSupported.description,
			};
			uiPhase.value = 'error';
			return;
	}
	uiPhase.value = 'confirm';
}
async function install() {
	if (!data.value) return;
	switch (data.value.type) {
		case 'plugin':
			if (!data.value.meta) return;
			try {
				await installPlugin(data.value.raw, data.value.meta as AiScriptPluginMeta);
				os.success();
				window.setTimeout(() => {
					close_();
				}, 3000);
			} catch (err) {
				errorKV.value = {
					title: i18n.ts._externalResourceInstaller._errors._pluginInstallFailed.title,
					description: i18n.ts._externalResourceInstaller._errors._pluginInstallFailed.description,
				};
				console.error(err);
				uiPhase.value = 'error';
			}
			break;
		case 'theme':
			if (!data.value.meta) return;
			await installTheme(data.value.raw);
			os.success();
			window.setTimeout(() => {
				close_();
			}, 3000);
	}
}
const urlParams = new URLSearchParams(window.location.search);
url.value = urlParams.get('url');
hash.value = urlParams.get('hash');
_fetch_();
definePage(() => ({
	title: i18n.ts._externalResourceInstaller.title,
	icon: 'ti ti-download',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithAnimBg = _resolveComponent("PageWithAnimBg")

  return (_openBlock(), _createBlock(_component_PageWithAnimBg, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 550px; --MI_SPACER-max: 50px;"
        }, [
          (uiPhase.value === 'fetching')
            ? (_openBlock(), _createBlock(MkLoading, { key: 0 }))
            : (uiPhase.value === 'confirm' && data.value)
              ? (_openBlock(), _createBlock(MkExtensionInstaller, {
                key: 1,
                extension: data.value,
                onConfirm: _cache[0] || (_cache[0] = ($event: any) => (install())),
                onCancel: _cache[1] || (_cache[1] = ($event: any) => (close_()))
              }, {
                additionalInfo: _withCtx(() => [
                  _createVNode(FormSection, null, {
                    default: _withCtx(() => [
                      _createElementVNode("div", { class: "_gaps_s" }, [
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._externalResourceInstaller._vendorInfo.endpoint), 1 /* TEXT */)
                          ]),
                          value: _withCtx(() => [
                            (url.value)
                              ? (_openBlock(), _createBlock(MkUrl, {
                                key: 0,
                                url: url.value,
                                showUrlPreview: false
                              }, null, 8 /* PROPS */, ["url", "showUrlPreview"]))
                              : _createCommentVNode("v-if", true)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._externalResourceInstaller._vendorInfo.hashVerify), 1 /* TEXT */)
                          ]),
                          value: _withCtx(() => [
                            _hoisted_1
                          ]),
                          _: 1 /* STABLE */
                        })
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["extension"]))
            : (uiPhase.value === 'error')
              ? (_openBlock(), _createElementBlock("div", {
                key: 2,
                class: _normalizeClass(["_gaps_m", [_ctx.$style.extInstallerRoot, _ctx.$style.error]])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.extInstallerIconWrapper)
                }, [
                  _hoisted_2
                ]),
                _createElementVNode("h2", {
                  class: _normalizeClass(_ctx.$style.extInstallerTitle)
                }, _toDisplayString(errorKV.value?.title), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.extInstallerNormDesc)
                }, _toDisplayString(errorKV.value?.description), 1 /* TEXT */),
                _createElementVNode("div", { class: "_buttonsCenter" }, [
                  _createVNode(MkButton, {
                    onClick: _cache[2] || (_cache[2] = ($event: any) => (close_()))
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.close), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
