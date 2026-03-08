import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-external-link" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { computed, provide, watch, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import XBlocks from './page-editor.blocks.vue'
import { genId } from '@/utility/id.js'
import MkButton from '@/components/MkButton.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInput from '@/components/MkInput.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { selectFile } from '@/utility/drive.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { $i } from '@/i.js'
import { mainRouter } from '@/router.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { getPageBlockList } from '@/pages/page-editor/common.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page-editor',
  props: {
    initPageId: { type: String, required: false },
    initPageName: { type: String, required: false },
    initUser: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const tab = ref('settings');
const author = ref<Misskey.entities.User | null>($i);
const readonly = ref(false);
const page = ref<Misskey.entities.Page | null>(null);
const pageId = ref<string | null>(null);
const currentName = ref<string | null>(null);
const title = ref('');
const summary = ref<string | null>(null);
const name = ref(Date.now().toString());
const eyeCatchingImage = ref<Misskey.entities.DriveFile | null>(null);
const eyeCatchingImageId = ref<string | null>(null);
const {
	model: font,
	def: fontDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._pages.fontSansSerif, value: 'sans-serif' },
		{ label: i18n.ts._pages.fontSerif, value: 'serif' },
	],
	initialValue: 'sans-serif',
});
const content = ref<Misskey.entities.Page['content']>([]);
const alignCenter = ref(false);
const hideTitleWhenPinned = ref(false);
provide('readonly', readonly.value);
watch(eyeCatchingImageId, async () => {
	if (eyeCatchingImageId.value == null) {
		eyeCatchingImage.value = null;
	} else {
		eyeCatchingImage.value = await misskeyApi('drive/files/show', {
			fileId: eyeCatchingImageId.value,
		});
	}
});
function getSaveOptions(): Misskey.entities.PagesCreateRequest {
	return {
		title: title.value.trim(),
		name: name.value.trim(),
		summary: summary.value,
		font: font.value,
		script: '',
		hideTitleWhenPinned: hideTitleWhenPinned.value,
		alignCenter: alignCenter.value,
		content: content.value,
		variables: [],
		eyeCatchingImageId: eyeCatchingImageId.value,
	};
}
async function save() {
	const options = getSaveOptions();
	if (pageId.value) {
		const updateOptions: Misskey.entities.PagesUpdateRequest = {
			pageId: pageId.value,
			...options,
		};
		await os.apiWithDialog('pages/update', updateOptions, undefined, {
			'2298a392-d4a1-44c5-9ebb-ac1aeaa5a9ab': {
				title: i18n.ts.somethingHappened,
				text: i18n.ts._pages.nameAlreadyExists,
			},
		});
		currentName.value = name.value.trim();
	} else {
		const created = await os.apiWithDialog('pages/create', options, undefined, {
			'4650348e-301c-499a-83c9-6aa988c66bc1': {
				title: i18n.ts.somethingHappened,
				text: i18n.ts._pages.nameAlreadyExists,
			},
		});
		pageId.value = created.id;
		currentName.value = name.value.trim();
		mainRouter.replace('/pages/edit/:initPageId', {
			params: {
				initPageId: pageId.value,
			},
		});
	}
}
async function del() {
	if (!pageId.value) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: title.value.trim() }),
	});
	if (canceled) return;
	await os.apiWithDialog('pages/delete', {
		pageId: pageId.value,
	});
	mainRouter.replace('/pages');
}
async function duplicate() {
	title.value = title.value + ' - copy';
	name.value = name.value + '-copy';
	const created = await os.apiWithDialog('pages/create', getSaveOptions(), undefined, {
		'4650348e-301c-499a-83c9-6aa988c66bc1': {
			title: i18n.ts.somethingHappened,
			text: i18n.ts._pages.nameAlreadyExists,
		},
	});
	pageId.value = created.id;
	currentName.value = name.value.trim();
	mainRouter.push('/pages/edit/:initPageId', {
		params: {
			initPageId: pageId.value,
		},
	});
}
async function add() {
	const { canceled, result: type } = await os.select({
		title: i18n.ts._pages.chooseBlock,
		items: getPageBlockList(),
	});
	if (canceled || type == null) return;
	const id = genId();
	// TODO: page-editor.el.section.vueのと共通化
	if (type === 'text') {
		content.value.push({
			id,
			type,
			text: '',
		});
	} else if (type === 'section') {
		content.value.push({
			id,
			type,
			title: '',
			children: [],
		});
	} else if (type === 'image') {
		content.value.push({
			id,
			type,
			fileId: null,
		});
	} else if (type === 'note') {
		content.value.push({
			id,
			type,
			detailed: false,
			note: null,
		});
	}
}
function setEyeCatchingImage(ev: PointerEvent) {
	selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
	}).then(file => {
		eyeCatchingImageId.value = file.id;
	});
}
function removeEyeCatchingImage() {
	eyeCatchingImageId.value = null;
}
async function init() {
	if (props.initPageId) {
		page.value = await misskeyApi('pages/show', {
			pageId: props.initPageId,
		});
	} else if (props.initPageName && props.initUser) {
		page.value = await misskeyApi('pages/show', {
			name: props.initPageName,
			username: props.initUser,
		});
		readonly.value = true;
	}
	if (page.value) {
		author.value = page.value.user;
		pageId.value = page.value.id;
		title.value = page.value.title;
		name.value = page.value.name;
		currentName.value = page.value.name;
		summary.value = page.value.summary;
		font.value = page.value.font;
		hideTitleWhenPinned.value = page.value.hideTitleWhenPinned;
		alignCenter.value = page.value.alignCenter;
		content.value = page.value.content;
		eyeCatchingImageId.value = page.value.eyeCatchingImageId;
	} else {
		const id = genId();
		content.value = [{
			id,
			type: 'text',
			text: 'Hello World!',
		}];
	}
}
init();
const headerActions = computed(() => []);
const headerTabs = computed(() => [{
	key: 'settings',
	title: i18n.ts._pages.pageSetting,
	icon: 'ti ti-settings',
}, {
	key: 'contents',
	title: i18n.ts._pages.contents,
	icon: 'ti ti-note',
}]);
definePage(() => ({
	title: props.initPageId ? i18n.ts._pages.editPage
	: props.initPageName && props.initUser ? i18n.ts._pages.readPage
	: i18n.ts._pages.newPage,
	icon: 'ti ti-pencil',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          _createElementVNode("div", { class: "jqqmcavi" }, [
            (pageId.value && author.value != null)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                class: "button",
                inline: "",
                link: "",
                to: `/@${ author.value.username }/pages/${ currentName.value }`
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.viewPage), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]))
              : _createCommentVNode("v-if", true),
            (!readonly.value)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                inline: "",
                primary: "",
                class: "button",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (pageId.value)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                inline: "",
                class: "button",
                onClick: duplicate
              }, {
                default: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.duplicate), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (pageId.value && !readonly.value)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                inline: "",
                class: "button",
                danger: "",
                onClick: del
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ]),
          (tab.value === 'settings')
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(MkInput, {
                  modelValue: title.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((title).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.title), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkInput, {
                  modelValue: summary.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((summary).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.summary), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkInput, {
                  modelValue: name.value,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((name).value = $event))
                }, {
                  prefix: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(url)) + "/@" + _toDisplayString(author.value?.username ?? '???') + "/pages/", 1 /* TEXT */)
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.url), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkSwitch, {
                  modelValue: alignCenter.value,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((alignCenter).value = $event))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.alignCenter), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkSelect, {
                  items: _unref(fontDef),
                  modelValue: _unref(font),
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((font).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.font), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["items", "modelValue"]),
                _createVNode(MkSwitch, {
                  modelValue: hideTitleWhenPinned.value,
                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((hideTitleWhenPinned).value = $event))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.hideTitleWhenPinned), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createElementVNode("div", { class: "eyeCatch" }, [
                  (eyeCatchingImageId.value == null && !readonly.value)
                    ? (_openBlock(), _createBlock(MkButton, {
                      key: 0,
                      onClick: setEyeCatchingImage
                    }, {
                      default: _withCtx(() => [
                        _hoisted_5,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.eyeCatchingImageSet), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }))
                    : (eyeCatchingImage.value)
                      ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                        _createElementVNode("img", {
                          src: eyeCatchingImage.value.url,
                          alt: eyeCatchingImage.value.name,
                          style: "max-width: 100%;"
                        }, null, 8 /* PROPS */, ["src", "alt"]),
                        (!readonly.value)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            onClick: _cache[7] || (_cache[7] = ($event: any) => (removeEyeCatchingImage()))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_6,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.eyeCatchingImageRemove), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true)
                      ]))
                    : _createCommentVNode("v-if", true)
                ])
              ])
            ]))
            : (tab.value === 'contents')
              ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.contents)
                }, [
                  _createVNode(XBlocks, {
                    class: "content",
                    modelValue: content.value,
                    "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((content).value = $event))
                  }, null, 8 /* PROPS */, ["modelValue"]),
                  (!readonly.value)
                    ? (_openBlock(), _createBlock(MkButton, {
                      key: 0,
                      rounded: "",
                      class: "add",
                      onClick: _cache[9] || (_cache[9] = ($event: any) => (add()))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_7
                      ]),
                      _: 1 /* STABLE */
                    }))
                    : _createCommentVNode("v-if", true)
                ])
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
