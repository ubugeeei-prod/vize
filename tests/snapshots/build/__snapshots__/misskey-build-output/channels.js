import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { computed, markRaw, onMounted, ref, shallowRef } from 'vue'
import MkChannelPreview from '@/components/MkChannelPreview.vue'
import MkChannelList from '@/components/MkChannelList.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkInput from '@/components/MkInput.vue'
import MkRadios from '@/components/MkRadios.vue'
import MkButton from '@/components/MkButton.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

type SearchType = 'nameAndDescription' | 'nameOnly';

export default /*@__PURE__*/_defineComponent({
  __name: 'channels',
  props: {
    query: { type: String, required: true },
    type: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const key = ref('');
const tab = ref('featured');
const searchQuery = ref('');
const searchType = ref<SearchType>('nameAndDescription');
const channelPaginator = shallowRef();
onMounted(() => {
	searchQuery.value = props.query ?? '';
	searchType.value = props.type ?? 'nameAndDescription';
});
const featuredPaginator = markRaw(new Paginator('channels/featured', {
	limit: 10,
	noPaging: true,
}));
const favoritesPaginator = markRaw(new Paginator('channels/my-favorites', {
	limit: 100,
	noPaging: true,
}));
const followingPaginator = markRaw(new Paginator('channels/followed', {
	limit: 10,
}));
const ownedPaginator = markRaw(new Paginator('channels/owned', {
	limit: 10,
}));
async function search() {
	const query = searchQuery.value.toString().trim();
	if (query == null) return;
	const type = searchType.value.toString().trim();
	if (type !== 'nameAndDescription' && type !== 'nameOnly') {
		console.error(`Unrecognized search type: ${type}`);
		return;
	}
	channelPaginator.value = markRaw(new Paginator('channels/search', {
		limit: 10,
		params: {
			query: searchQuery.value,
			type: type,
		},
	}));
	key.value = query + type;
}
const headerActions = computed(() => []);
const headerTabs = computed(() => [{
	key: 'search',
	title: i18n.ts.search,
	icon: 'ti ti-search',
}, {
	key: 'featured',
	title: i18n.ts._channel.featured,
	icon: 'ti ti-comet',
}, {
	key: 'favorites',
	title: i18n.ts.favorites,
	icon: 'ti ti-star',
}, {
	key: 'following',
	title: i18n.ts._channel.following,
	icon: 'ti ti-eye',
}, {
	key: 'owned',
	title: i18n.ts._channel.owned,
	icon: 'ti ti-edit',
}]);
definePage(() => ({
	title: i18n.ts.channel,
	icon: 'ti ti-device-tv',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      swipable: true,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 1200px;"
        }, [
          (tab.value === 'search')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.searchRoot)
            }, [
              _createElementVNode("div", { class: "_gaps" }, [
                _createVNode(MkInput, {
                  large: true,
                  autofocus: true,
                  type: "search",
                  onEnter: search,
                  modelValue: searchQuery.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((searchQuery).value = $event))
                }, {
                  prefix: _withCtx(() => [
                    _hoisted_1
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["large", "autofocus", "modelValue"]),
                _createVNode(MkRadios, {
                  options: [
  						{ value: 'nameAndDescription', label: _unref(i18n).ts._channel.nameAndDescription },
  						{ value: 'nameOnly', label: _unref(i18n).ts._channel.nameOnly },
  					],
                  "onUpdate:modelValue": [($event: any) => (search()), ($event: any) => ((searchType).value = $event)],
                  modelValue: searchType.value
                }, null, 8 /* PROPS */, ["options", "modelValue"]),
                _createVNode(MkButton, {
                  large: "",
                  primary: "",
                  gradate: "",
                  rounded: "",
                  onClick: search
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              (channelPaginator.value)
                ? (_openBlock(), _createBlock(MkFoldableSection, { key: 0 }, {
                  header: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.searchResult), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(MkChannelList, {
                      key: key.value,
                      paginator: channelPaginator.value
                    }, null, 8 /* PROPS */, ["paginator"])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ]))
            : _createCommentVNode("v-if", true),
          (tab.value === 'featured')
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createVNode(MkPagination, { paginator: _unref(featuredPaginator) }, {
                default: _withCtx(({items}) => [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.root)
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (channel) => {
                      return (_openBlock(), _createBlock(MkChannelPreview, {
                        key: channel.id,
                        channel: channel
                      }, null, 8 /* PROPS */, ["channel"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["paginator"])
            ]))
            : (tab.value === 'favorites')
              ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                _createVNode(MkPagination, { paginator: _unref(favoritesPaginator) }, {
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.root)
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (channel) => {
                        return (_openBlock(), _createBlock(MkChannelPreview, {
                          key: channel.id,
                          channel: channel
                        }, null, 8 /* PROPS */, ["channel"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'following')
              ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
                _createVNode(MkPagination, { paginator: _unref(followingPaginator) }, {
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.root)
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (channel) => {
                        return (_openBlock(), _createBlock(MkChannelPreview, {
                          key: channel.id,
                          channel: channel
                        }, null, 8 /* PROPS */, ["channel"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'owned')
              ? (_openBlock(), _createElementBlock("div", {
                key: 3,
                class: "_gaps"
              }, [
                _createVNode(MkButton, {
                  link: "",
                  primary: "",
                  rounded: "",
                  to: "/channels/new"
                }, {
                  default: _withCtx(() => [
                    _hoisted_2,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.createNew), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkPagination, { paginator: _unref(ownedPaginator) }, {
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.root)
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (channel) => {
                        return (_openBlock(), _createBlock(MkChannelPreview, {
                          key: channel.id,
                          channel: channel
                        }, null, 8 /* PROPS */, ["channel"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "swipable", "tab"]))
}
}

})
