import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { computed, markRaw, ref, shallowRef } from 'vue'
import type { IPaginator } from '@/utility/paginator.js'
import MkFlashPreview from '@/components/MkFlashPreview.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'flash-index',
  setup(__props) {

const router = useRouter();
const tab = ref('featured');
const searchQuery = ref('');
const searchPaginator = shallowRef<Paginator<'flash/search'> | null>(null);
const searchKey = ref(0);
const featuredFlashsPaginator = markRaw(new Paginator('flash/featured', {
	limit: 5,
	offsetMode: true,
}));
const myFlashsPaginator = markRaw(new Paginator('flash/my', {
	limit: 5,
}));
const likedFlashsPaginator = markRaw(new Paginator('flash/my-likes', {
	limit: 5,
	canSearch: true,
	searchParamName: 'search',
}));
function create() {
	router.push('/play/new');
}
function search() {
	if (searchQuery.value.trim() === '') {
		return;
	}
	searchPaginator.value = markRaw(new Paginator('flash/search', {
		params: {
			query: searchQuery.value,
		},
	}));
	searchKey.value++;
}
const headerActions = computed(() => [{
	icon: 'ti ti-plus',
	text: i18n.ts.create,
	handler: create,
}]);
const headerTabs = computed(() => [{
	key: 'search',
	title: i18n.ts.search,
	icon: 'ti ti-search',
}, {
	key: 'featured',
	title: i18n.ts._play.featured,
	icon: 'ti ti-flare',
}, {
	key: 'my',
	title: i18n.ts._play.my,
	icon: 'ti ti-edit',
}, {
	key: 'liked',
	title: i18n.ts._play.liked,
	icon: 'ti ti-heart',
}]);
definePage(() => ({
	title: 'Play',
	icon: 'ti ti-player-play',
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
          style: "--MI_SPACER-w: 700px;"
        }, [
          (tab.value === 'search')
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createElementVNode("div", { class: "_gaps" }, [
                _createVNode(MkInput, {
                  large: true,
                  type: "search",
                  modelValue: searchQuery.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((searchQuery).value = $event))
                }, {
                  prefix: _withCtx(() => [
                    _hoisted_1
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["large", "modelValue"]),
                _createVNode(MkButton, {
                  large: "",
                  primary: "",
                  gradate: "",
                  rounded: "",
                  style: "margin: 0 auto;",
                  onClick: search
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                (searchPaginator.value)
                  ? (_openBlock(), _createBlock(MkPagination, {
                    key: searchKey.value,
                    paginator: searchPaginator.value
                  }, {
                    default: _withCtx(({items}) => [
                      _createElementVNode("div", { class: "_gaps_s" }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (flash) => {
                          return (_openBlock(), _createBlock(MkFlashPreview, {
                            key: flash.id,
                            flash: flash
                          }, null, 8 /* PROPS */, ["flash"]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["paginator"]))
                  : _createCommentVNode("v-if", true)
              ])
            ]))
            : (tab.value === 'featured')
              ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                _createVNode(MkPagination, { paginator: _unref(featuredFlashsPaginator) }, {
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (flash) => {
                        return (_openBlock(), _createBlock(MkFlashPreview, {
                          key: flash.id,
                          flash: flash
                        }, null, 8 /* PROPS */, ["flash"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'my')
              ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
                _createElementVNode("div", { class: "_gaps" }, [
                  _createVNode(MkButton, {
                    gradate: "",
                    rounded: "",
                    style: "margin: 0 auto;",
                    onClick: _cache[2] || (_cache[2] = ($event: any) => (create()))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_2
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkPagination, { paginator: _unref(myFlashsPaginator) }, {
                    default: _withCtx(({items}) => [
                      _createElementVNode("div", { class: "_gaps_s" }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (flash) => {
                          return (_openBlock(), _createBlock(MkFlashPreview, {
                            key: flash.id,
                            flash: flash
                          }, null, 8 /* PROPS */, ["flash"]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["paginator"])
                ])
              ]))
            : (tab.value === 'liked')
              ? (_openBlock(), _createElementBlock("div", { key: 3 }, [
                _createVNode(MkPagination, {
                  paginator: _unref(likedFlashsPaginator),
                  withControl: ""
                }, {
                  default: _withCtx(({items}) => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (like) => {
                        return (_openBlock(), _createBlock(MkFlashPreview, {
                          key: like.flash.id,
                          flash: like.flash
                        }, null, 8 /* PROPS */, ["flash"]))
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
