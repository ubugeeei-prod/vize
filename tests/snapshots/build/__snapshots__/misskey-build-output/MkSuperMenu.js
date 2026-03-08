import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_2 = { class: "text" }
const _hoisted_3 = { class: "text" }
const _hoisted_4 = { class: "text" }
const _hoisted_5 = { style: "opacity: 0.7; font-size: 90%; word-break: break-word;" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_7 = { style: "word-break: break-word;" }
import { useTemplateRef, ref, watch, nextTick, computed } from 'vue'
import { getScrollContainer } from '@@/js/scroll.js'
import type { SearchIndexItem } from '@/utility/inapp-search.js'
import MkInput from '@/components/MkInput.vue'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'
import { initIntlString, compareStringIncludes } from '@/utility/intl-string.js'

import type { Awaitable } from '@/types/misc.js';

export type SuperMenuDef = {
	title?: string;
	items: ({
		type: 'a';
		href: string;
		target?: string;
		icon?: string;
		text: string;
		danger?: boolean;
		active?: boolean;
	} | {
		type: 'button';
		icon?: string;
		text: string;
		danger?: boolean;
		active?: boolean;
		action: (ev: PointerEvent) => Awaitable<void>;
	} | {
		type?: 'link';
		to: string;
		icon?: string;
		text: string;
		danger?: boolean;
		active?: boolean;
	})[];
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSuperMenu',
  props: {
    def: { type: Array, required: true },
    grid: { type: Boolean, required: false },
    searchIndex: { type: Array, required: false }
  },
  setup(__props: any) {

const props = __props
initIntlString();
const router = useRouter();
const rootEl = useTemplateRef('rootEl');
const searchQuery = ref('');
const rawSearchQuery = ref('');
const searchSelectedIndex = ref<null | number>(null);
const searchResult = ref<{
	id: string;
	path: string;
	label: string;
	icon?: string;
	isRoot: boolean;
	parentLabels: string[];
}[]>([]);
const searchIndexItemByIdComputed = computed(() => props.searchIndex && new Map<string, SearchIndexItem>(props.searchIndex.map(i => [i.id, i])));
watch(searchQuery, (value) => {
	rawSearchQuery.value = value;
});
watch(rawSearchQuery, (value) => {
	searchResult.value = [];
	searchSelectedIndex.value = null;
	if (value === '') {
		return;
	}
	const searchIndexItemById = searchIndexItemByIdComputed.value;
	if (searchIndexItemById != null) {
		const addSearchResult = (item: SearchIndexItem) => {
			let path: string | undefined = item.path;
			let icon: string | undefined = item.icon;
			const parentLabels: string[] = [];

			for (let current = searchIndexItemById.get(item.parentId ?? '');
				current != null;
				current = searchIndexItemById.get(current.parentId ?? '')) {
				path ??= current.path;
				icon ??= current.icon;
				parentLabels.push(current.label);
			}

			if (_DEV_ && path == null) throw new Error('path is null for ' + item.id);

			searchResult.value.push({
				id: item.id,
				path: path ?? '/', // never gets `/`
				label: item.label,
				parentLabels: parentLabels.toReversed(),
				icon,
				isRoot: item.parentId == null,
			});
		};
		// label, keywords, texts の順に優先して表示
		let items = Array.from(searchIndexItemById.values());
		for (const item of items) {
			if (compareStringIncludes(item.label, value)) {
				addSearchResult(item);
				items = items.filter(i => i.id !== item.id);
			}
		}
		for (const item of items) {
			if (item.keywords.some((x) => compareStringIncludes(x, value))) {
				addSearchResult(item);
				items = items.filter(i => i.id !== item.id);
			}
		}
		for (const item of items) {
			if (item.texts.some((x) => compareStringIncludes(x, value))) {
				addSearchResult(item);
				items = items.filter(i => i.id !== item.id);
			}
		}
	}
});
function searchOnInput(ev: InputEvent) {
	searchSelectedIndex.value = null;
	rawSearchQuery.value = (ev.target as HTMLInputElement).value;
}
function searchOnKeyDown(ev: KeyboardEvent) {
	if (ev.isComposing) return;
	if (ev.key === 'Enter' && searchSelectedIndex.value != null) {
		ev.preventDefault();
		router.pushByPath(searchResult.value[searchSelectedIndex.value].path + '#' + searchResult.value[searchSelectedIndex.value].id);
	} else if (ev.key === 'ArrowDown') {
		ev.preventDefault();
		const current = searchSelectedIndex.value ?? -1;
		searchSelectedIndex.value = current + 1 >= searchResult.value.length ? 0 : current + 1;
	} else if (ev.key === 'ArrowUp') {
		ev.preventDefault();
		const current = searchSelectedIndex.value ?? 0;
		searchSelectedIndex.value = current - 1 < 0 ? searchResult.value.length - 1 : current - 1;
	}
	if (ev.key === 'ArrowDown' || ev.key === 'ArrowUp') {
		nextTick(() => {
			if (!rootEl.value) return;
			const selectedEl = rootEl.value.querySelector<HTMLElement>('.searchResultItem.selected');
			if (selectedEl != null) {
				const scrollContainer = getScrollContainer(selectedEl);
				if (!scrollContainer) return;
				scrollContainer.scrollTo({
					top: selectedEl.offsetTop - scrollContainer.clientHeight / 2 + selectedEl.clientHeight / 2,
					behavior: 'instant',
				});
			}
		});
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(["rrevdjwu", { grid: __props.grid }])
    }, [ (__props.searchIndex && __props.searchIndex.length > 0) ? (_openBlock(), _createBlock(MkInput, {
          key: 0,
          placeholder: _unref(i18n).ts.search,
          type: "search",
          style: "margin-bottom: 16px;",
          onInputPassive: searchOnInput,
          onKeydown: searchOnKeyDown,
          modelValue: searchQuery.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((searchQuery).value = $event))
        }, {
          prefix: _withCtx(() => [
            _hoisted_1
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["placeholder", "modelValue"])) : _createCommentVNode("v-if", true), (rawSearchQuery.value == '') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.def, (group) => {
            return (_openBlock(), _createElementBlock("div", { class: "group" }, [
              (group.title)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "title"
                }, _toDisplayString(group.title), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createElementVNode("div", { class: "items" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.items, (item, i) => {
                  return (_openBlock(), _createElementBlock(_Fragment, null, [
                    (item.type === 'a')
                      ? (_openBlock(), _createElementBlock("a", {
                        key: 0,
                        href: item.href,
                        target: item.target,
                        class: _normalizeClass(["_button item", { danger: item.danger, active: item.active }])
                      }, [
                        (item.icon)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "icon"
                          }, [
                            _createElementVNode("i", {
                              class: _normalizeClass(["ti-fw", item.icon])
                            }, null, 2 /* CLASS */)
                          ]))
                          : _createCommentVNode("v-if", true),
                        _createElementVNode("span", _hoisted_2, _toDisplayString(item.text), 1 /* TEXT */)
                      ]))
                      : (item.type === 'button')
                        ? (_openBlock(), _createElementBlock("button", {
                          key: 1,
                          class: _normalizeClass(["_button item", { danger: item.danger, active: item.active }]),
                          disabled: item.active,
                          onClick: _cache[1] || (_cache[1] = ev => item.action(ev))
                        }, [
                          (item.icon)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              class: "icon"
                            }, [
                              _createElementVNode("i", {
                                class: _normalizeClass(["ti-fw", item.icon])
                              }, null, 2 /* CLASS */)
                            ]))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("span", _hoisted_3, _toDisplayString(item.text), 1 /* TEXT */)
                        ]))
                      : (_openBlock(), _createBlock(_component_MkA, {
                        key: 2,
                        to: item.to,
                        class: _normalizeClass(["_button item", { danger: item.danger, active: item.active }])
                      }, {
                        default: _withCtx(() => [
                          (item.icon)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              class: "icon"
                            }, [
                              _createElementVNode("i", {
                                class: _normalizeClass(["ti-fw", item.icon])
                              }, null, 2 /* CLASS */)
                            ]))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("span", _hoisted_4, _toDisplayString(item.text), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 10 /* CLASS, PROPS */, ["to"]))
                  ], 64 /* STABLE_FRAGMENT */))
                }), 256 /* UNKEYED_FRAGMENT */))
              ])
            ]))
          }), 256 /* UNKEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(searchResult.value, (item, index) => {
            return (_openBlock(), _createElementBlock("div", null, [
              _createVNode(_component_MkA, {
                to: _ctx.item.path + '#' + _ctx.item.id,
                class: _normalizeClass(["_button searchResultItem", { selected: searchSelectedIndex.value !== null && searchSelectedIndex.value === _ctx.index }])
              }, {
                default: _withCtx(() => [
                  (_ctx.item.icon)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: "icon"
                    }, [
                      _createElementVNode("i", {
                        class: _normalizeClass(["ti-fw", _ctx.item.icon])
                      })
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("span", { class: "text" }, [
                    (_ctx.item.isRoot)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _toDisplayString(_ctx.item.label)
                      ], 64 /* STABLE_FRAGMENT */))
                      : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                        _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.item.parentLabels.join(' > ')), 1 /* TEXT */),
                        _hoisted_6,
                        _createElementVNode("span", _hoisted_7, _toDisplayString(_ctx.item.label), 1 /* TEXT */)
                      ], 64 /* STABLE_FRAGMENT */))
                  ])
                ]),
                _: 2 /* DYNAMIC */
              }, 10 /* CLASS, PROPS */, ["to"])
            ]))
          }), 256 /* UNKEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) ], 2 /* CLASS */))
}
}

})
