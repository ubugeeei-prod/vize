import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrows-sort" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-notes" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div")
import { computed, onMounted, ref, useCssModule } from 'vue'
import * as Misskey from 'misskey-js'
import type { GridSortOrderKey, RequestLogItem } from '@/pages/admin/custom-emojis-manager.impl.js'
import type { GridCellValueChangeEvent, GridEvent } from '@/components/grid/grid-event.js'
import type { GridSetting } from '@/components/grid/grid.js'
import type { SortOrder } from '@/components/MkSortOrderEditor.define.js'
import MkRemoteEmojiEditDialog from '@/components/MkRemoteEmojiEditDialog.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkGrid from '@/components/grid/MkGrid.vue'
import { emptyStrToUndefined, gridSortOrderKeys } from '@/pages/admin/custom-emojis-manager.impl.js'
import MkFolder from '@/components/MkFolder.vue'
import XRegisterLogs from '@/pages/admin/custom-emojis-manager.logs.vue'
import * as os from '@/os.js'
import { deviceKind } from '@/utility/device-kind.js'
import MkPagingButtons from '@/components/MkPagingButtons.vue'
import MkSortOrderEditor from '@/components/MkSortOrderEditor.vue'
import { useLoading } from '@/composables/use-loading.js'

type GridItem = {
	checked: boolean;
	id: string;
	url: string;
	name: string;
	host: string;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'custom-emojis-manager.remote',
  setup(__props) {

function setupGrid(): GridSetting {
	const $style = useCssModule();
	return {
		row: {
			// グリッドの行数をあらかじめ100行確保する
			minimumDefinitionCount: 100,
			styleRules: [
				{
					// チェックされたら背景色を変える
					condition: ({ row }) => gridItems.value[row.index].checked,
					applyStyle: { className: $style.changedRow },
				},
			],
			contextMenuFactory: (row, context) => {
				return [
					{
						type: 'button',
						text: i18n.ts._customEmojisManager._remote.importSelectionRows,
						icon: 'ti ti-download',
						action: async () => {
							const targets = context.rangedRows.map(it => gridItems.value[it.index]);
							await importEmojis(targets);
						},
					},
				];
			},
		},
		cols: [
			{ bindTo: 'checked', icon: 'ti-download', type: 'boolean', editable: true, width: 34 },
			{ bindTo: 'url', icon: 'ti-icons', type: 'image', editable: false, width: 'auto' },
			{ bindTo: 'name', title: 'name', type: 'text', editable: false, width: 'auto' },
			{ bindTo: 'host', title: 'host', type: 'text', editable: false, width: 'auto' },
			{ bindTo: 'license', title: 'license', type: 'text', editable: false, width: 200 },
			{ bindTo: 'uri', title: 'uri', type: 'text', editable: false, width: 'auto' },
			{ bindTo: 'publicUrl', title: 'publicUrl', type: 'text', editable: false, width: 'auto' },
		],
		cells: {
			contextMenuFactory: (col, row, value, context) => {
				return [
					{
						type: 'button',
						text: i18n.ts._customEmojisManager._remote.selectionRowDetail,
						icon: 'ti ti-info-circle',
						action: async () => {
							const target = customEmojis.value[row.index];
							const { dispose } = os.popup(MkRemoteEmojiEditDialog, {
								emoji: {
									id: target.id,
									name: target.name,
									host: target.host!,
									license: target.license,
									url: target.publicUrl,
								},
							}, {
								done: () => {
									dispose();
								},
								closed: () => {
									dispose();
								},
							});
						},
					},
					{
						type: 'button',
						text: i18n.ts._customEmojisManager._remote.importSelectionRangesRows,
						icon: 'ti ti-download',
						action: async () => {
							const targets = context.rangedCells.map(it => gridItems.value[it.row.index]);
							await importEmojis(targets);
						},
					},
				];
			},
		},
	};
}
const loadingHandler = useLoading();
const customEmojis = ref<Misskey.entities.EmojiDetailedAdmin[]>([]);
const allPages = ref<number>(0);
const currentPage = ref<number>(0);
const queryName = ref<string | null>(null);
const queryHost = ref<string | null>(null);
const queryLicense = ref<string | null>(null);
const queryUri = ref<string | null>(null);
const queryPublicUrl = ref<string | null>(null);
const queryLimit = ref<number>(100);
const previousQuery = ref<string | undefined>(undefined);
const sortOrders = ref<SortOrder<GridSortOrderKey>[]>([]);
const requestLogs = ref<RequestLogItem[]>([]);
const gridItems = ref<GridItem[]>([]);
const spMode = computed(() => ['smartphone', 'tablet'].includes(deviceKind));
const checkedItemsCount = computed(() => gridItems.value.filter(it => it.checked).length);
function onSortOrderUpdate(_sortOrders: SortOrder<GridSortOrderKey>[]) {
	sortOrders.value = _sortOrders;
}
async function onSearchRequest() {
	await refreshCustomEmojis();
}
function onQueryResetButtonClicked() {
	queryName.value = null;
	queryHost.value = null;
	queryLicense.value = null;
	queryUri.value = null;
	queryPublicUrl.value = null;
}
async function onPageChanged(pageNumber: number) {
	currentPage.value = pageNumber;
	await refreshCustomEmojis();
}
async function onImportClicked() {
	const targets = gridItems.value.filter(it => it.checked);
	await importEmojis(targets);
}
function onGridEvent(event: GridEvent) {
	switch (event.type) {
		case 'cell-value-change':
			onGridCellValueChange(event);
			break;
	}
}
function onGridCellValueChange(event: GridCellValueChangeEvent) {
	const { row, column, newValue } = event;
	if (gridItems.value.length > row.index && column.setting.bindTo in gridItems.value[row.index]) {
		(gridItems.value[row.index] as any)[column.setting.bindTo] = newValue;
	}
}
async function importEmojis(targets: GridItem[]) {
	const confirm = await os.confirm({
		type: 'info',
		title: i18n.ts._customEmojisManager._remote.confirmImportEmojisTitle,
		text: i18n.tsx._customEmojisManager._remote.confirmImportEmojisDescription({ count: targets.length }),
	});
	if (confirm.canceled) {
		return;
	}
	const result = await os.promiseDialog(
		Promise.all(
			targets.map(item =>
				misskeyApi(
					'admin/emoji/copy',
					{
						emojiId: item.id!,
					})
					.then(() => ({ item, success: true, err: undefined }))
					.catch(err => ({ item, success: false, err })),
			),
		),
	);
	const failedItems = result.filter(it => !it.success);
	if (failedItems.length > 0) {
		await os.alert({
			type: 'error',
			title: i18n.ts.somethingHappened,
			text: i18n.ts._customEmojisManager._gridCommon.alertEmojisRegisterFailedDescription,
		});
	}
	requestLogs.value = result.map(it => ({
		failed: !it.success,
		url: it.item.url,
		name: it.item.name,
		error: it.err ? JSON.stringify(it.err) : undefined,
	}));
	await refreshCustomEmojis();
}
async function refreshCustomEmojis() {
	const query: Misskey.entities.V2AdminEmojiListRequest['query'] = {
		name: emptyStrToUndefined(queryName.value),
		host: emptyStrToUndefined(queryHost.value),
		license: emptyStrToUndefined(queryLicense.value),
		uri: emptyStrToUndefined(queryUri.value),
		publicUrl: emptyStrToUndefined(queryPublicUrl.value),
		hostType: 'remote',
	};
	if (JSON.stringify(query) !== previousQuery.value) {
		currentPage.value = 1;
	}
	const result = await loadingHandler.scope(() => misskeyApi('v2/admin/emoji/list', {
		limit: queryLimit.value,
		query: query,
		page: currentPage.value,
		sortKeys: sortOrders.value.map(({ key, direction }) => `${direction}${key}`) as never[],
	}));
	customEmojis.value = result.emojis;
	allPages.value = result.allPages;
	previousQuery.value = JSON.stringify(query);
	gridItems.value = customEmojis.value.map(it => ({
		checked: false,
		id: it.id,
		url: it.publicUrl,
		name: it.name,
		license: it.license,
		host: it.host!,
	}));
}
onMounted(async () => {
	await refreshCustomEmojis();
});

return (_ctx: any,_cache: any) => {
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")

  return (_openBlock(), _createBlock(_component_MkStickyContainer, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(["_gaps", _ctx.$style.root])
        }, [
          _createVNode(MkFolder, null, {
            icon: _withCtx(() => [
              _hoisted_1
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.searchSettings), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.searchSettingCaption), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps" }, [
                _createElementVNode("div", {
                  class: _normalizeClass([[spMode.value ? _ctx.$style.searchAreaSp : _ctx.$style.searchArea]])
                }, [
                  _createVNode(MkInput, {
                    type: "search",
                    autocapitalize: "off",
                    class: _normalizeClass([_ctx.$style.col1, _ctx.$style.row1]),
                    onEnter: onSearchRequest,
                    modelValue: queryName.value,
                    "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((queryName).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("name")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkInput, {
                    type: "search",
                    autocapitalize: "off",
                    class: _normalizeClass([_ctx.$style.col2, _ctx.$style.row1]),
                    onEnter: onSearchRequest,
                    modelValue: queryHost.value,
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((queryHost).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("host")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkInput, {
                    type: "search",
                    autocapitalize: "off",
                    class: _normalizeClass([_ctx.$style.col3, _ctx.$style.row1]),
                    onEnter: onSearchRequest,
                    modelValue: queryLicense.value,
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((queryLicense).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("license")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkInput, {
                    type: "search",
                    autocapitalize: "off",
                    class: _normalizeClass([_ctx.$style.col1, _ctx.$style.row2]),
                    onEnter: onSearchRequest,
                    modelValue: queryUri.value,
                    "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((queryUri).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("uri")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkInput, {
                    type: "search",
                    autocapitalize: "off",
                    class: _normalizeClass([_ctx.$style.col2, _ctx.$style.row2]),
                    onEnter: onSearchRequest,
                    modelValue: queryPublicUrl.value,
                    "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((queryPublicUrl).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("publicUrl")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"])
                ], 2 /* CLASS */),
                _hoisted_2,
                _createVNode(MkFolder, {
                  spacerMax: 8,
                  spacerMin: 8
                }, {
                  icon: _withCtx(() => [
                    _hoisted_3
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.sortOrder), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkSortOrderEditor, {
                      baseOrderKeyNames: _unref(gridSortOrderKeys),
                      currentOrders: sortOrders.value,
                      onUpdate: onSortOrderUpdate
                    }, null, 8 /* PROPS */, ["baseOrderKeyNames", "currentOrders"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["spacerMax", "spacerMin"]),
                _createVNode(MkInput, {
                  type: "number",
                  max: 100,
                  modelValue: queryLimit.value,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((queryLimit).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.searchLimit), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["max", "modelValue"]),
                _createElementVNode("div", {
                  class: _normalizeClass([[spMode.value ? _ctx.$style.searchButtonsSp : _ctx.$style.searchButtons]])
                }, [
                  _createVNode(MkButton, {
                    primary: "",
                    onClick: onSearchRequest
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkButton, { onClick: onQueryResetButtonClicked }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.reset), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ], 2 /* CLASS */)
              ])
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkFolder, null, {
            icon: _withCtx(() => [
              _hoisted_4
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.registrationLogs), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.registrationLogsCaption), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createVNode(XRegisterLogs, { logs: requestLogs.value }, null, 8 /* PROPS */, ["logs"])
            ]),
            _: 1 /* STABLE */
          }),
          (_unref(loadingHandler).showing.value)
            ? (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(loadingHandler).component.value), {
              key: 0,
              is: _unref(loadingHandler).component.value
            }))
            : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
              (gridItems.value.length === 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  style: "text-align: center"
                }, _toDisplayString(_unref(i18n).ts._customEmojisManager._local._list.emojisNothing), 1 /* TEXT */))
                : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  (gridItems.value.length > 0)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.gridArea)
                    }, [
                      _createVNode(MkGrid, {
                        data: gridItems.value,
                        settings: setupGrid(),
                        onEvent: onGridEvent
                      }, null, 8 /* PROPS */, ["data", "settings"])
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.footer)
                  }, [
                    _hoisted_5,
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.center)
                    }, [
                      _createVNode(MkPagingButtons, {
                        current: currentPage.value,
                        max: allPages.value,
                        buttonCount: 5,
                        onPageChanged: onPageChanged
                      }, null, 8 /* PROPS */, ["current", "max", "buttonCount"])
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.right)
                    }, [
                      _createVNode(MkButton, {
                        primary: "",
                        onClick: onImportClicked
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._remote.importEmojisButton), 1 /* TEXT */),
                          _createTextVNode(" ("),
                          _createTextVNode(_toDisplayString(checkedItemsCount.value), 1 /* TEXT */),
                          _createTextVNode(")\n\t\t\t\t\t\t\t")
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ])
                ], 64 /* STABLE_FRAGMENT */))
            ], 64 /* STABLE_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
