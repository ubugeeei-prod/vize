import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { style: "margin-bottom: 4px;" }
const _hoisted_2 = { style: "margin-right: 1em;" }
import * as Misskey from 'misskey-js'
import { computed, markRaw, ref, watch } from 'vue'
import tinycolor from 'tinycolor2'
import type { StyleValue } from 'vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkPagination from '@/components/MkPagination.vue'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import { i18n } from '@/i18n.js'
import bytes from '@/filters/bytes.js'
import { definePage } from '@/page.js'
import MkSelect from '@/components/MkSelect.vue'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { useGlobalEvent } from '@/events.js'
import { getDriveFileMenu } from '@/utility/get-drive-file-menu.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive-cleaner',
  setup(__props) {

const sortMode = ref<Misskey.entities.DriveFilesRequest['sort']>('+size');
const paginator = markRaw(new Paginator('drive/files', {
	limit: 10,
	computedParams: computed(() => ({ sort: sortMode.value })),
}));
const capacity = ref<number>(0);
const usage = ref<number>(0);
const fetching = ref(true);
const {
	model: sortModeSelect,
	def: sortModeSelectDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._drivecleaner.orderBySizeDesc, value: 'sizeDesc' },
		{ label: i18n.ts._drivecleaner.orderByCreatedAtAsc, value: 'createdAtAsc' },
	],
	initialValue: 'sizeDesc',
});
fetchDriveInfo();
watch(sortModeSelect, () => {
	switch (sortModeSelect.value) {
		case 'sizeDesc':
			sortMode.value = '+size';
			fetchDriveInfo();
			break;
		case 'createdAtAsc':
			sortMode.value = '-createdAt';
			fetchDriveInfo();
			break;
	}
});
function fetchDriveInfo(): void {
	fetching.value = true;
	misskeyApi('drive').then(info => {
		capacity.value = info.capacity;
		usage.value = info.usage;
		fetching.value = false;
	});
}
function genUsageBar(fsize: number): StyleValue {
	return {
		width: `${fsize / usage.value * 100}%`,
		background: tinycolor({ h: 180 - (fsize / usage.value * 180), s: 0.7, l: 0.5 }).toHslString(),
	};
}
function onClick(ev: PointerEvent, file: Misskey.entities.DriveFile) {
	os.popupMenu(getDriveFileMenu(file), (ev.currentTarget ?? ev.target ?? undefined) as HTMLElement | undefined);
}
function onContextMenu(ev: PointerEvent, file: Misskey.entities.DriveFile): void {
	os.contextMenu(getDriveFileMenu(file), ev);
}
useGlobalEvent('driveFilesDeleted', (files) => {
	for (const f of files) {
		paginator.removeItem(f.id);
	}
});
definePage(() => ({
	title: i18n.ts.drivecleaner,
	icon: 'ti ti-trash',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkSelect, {
        items: _unref(sortModeSelectDef),
        modelValue: _unref(sortModeSelect),
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((sortModeSelect).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.sort), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["items", "modelValue"]), (!fetching.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(MkPagination, { paginator: _unref(paginator) }, {
            default: _withCtx(({items}) => [
              _createElementVNode("div", { class: "_gaps" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (file) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: file.id,
                    class: "_button",
                    onClick: _cache[1] || (_cache[1] = $event => onClick($event, file)),
                    onContextmenu: _cache[2] || (_cache[2] = _withModifiers($event => onContextMenu($event, file), ["stop"]))
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.file)
                    }, [
                      (file.isSensitive)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "sensitive-label"
                        }, _toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      _createVNode(MkDriveFileThumbnail, {
                        class: _normalizeClass(_ctx.$style.fileThumbnail),
                        file: file,
                        fit: "contain"
                      }, null, 8 /* PROPS */, ["file"]),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.fileBody)
                      }, [
                        _createElementVNode("div", _hoisted_1, _toDisplayString(file.name), 1 /* TEXT */),
                        _createElementVNode("div", null, [
                          _createElementVNode("span", _hoisted_2, _toDisplayString(file.type), 1 /* TEXT */),
                          _createElementVNode("span", null, _toDisplayString(bytes(file.size)), 1 /* TEXT */)
                        ]),
                        _createElementVNode("div", null, [
                          _createElementVNode("span", null, [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.registeredDate) + ": ", 1 /* TEXT */),
                            _createVNode(_component_MkTime, {
                              time: file.createdAt,
                              mode: "detail"
                            }, null, 8 /* PROPS */, ["time"])
                          ])
                        ]),
                        (_unref(sortModeSelect) === 'sizeDesc')
                          ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                            _createElementVNode("div", {
                              class: _normalizeClass(_ctx.$style.meter)
                            }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.meterValue),
                                style: _normalizeStyle(genUsageBar(file.size))
                              }, null, 4 /* STYLE */)
                            ])
                          ]))
                          : _createCommentVNode("v-if", true)
                      ])
                    ])
                  ], 32 /* NEED_HYDRATION */))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["paginator"]) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createVNode(_component_MkLoading) ])) ]))
}
}

})
