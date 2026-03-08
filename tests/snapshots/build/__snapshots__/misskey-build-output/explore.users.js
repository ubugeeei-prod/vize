import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bookmark ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chart-line ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hash ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hash ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chart-line ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message ti-fw", style: "margin-right: 0.5em;" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket ti-fw", style: "margin-right: 0.5em;" })
import { watch, ref, useTemplateRef, computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkUserList from '@/components/MkUserList.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import MkTab from '@/components/MkTab.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'explore.users',
  props: {
    tag: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const origin = ref<'local' | 'remote'>('local');
const tagsLocal = ref<Misskey.entities.Hashtag[]>([]);
const tagsRemote = ref<Misskey.entities.Hashtag[]>([]);
const tagUsersPaginator = props.tag != null ? markRaw(new Paginator('hashtags/users', {
	limit: 30,
	params: {
		tag: props.tag,
		origin: 'combined',
		sort: '+follower',
	},
})) : null;
const pinnedUsersPaginator = markRaw(new Paginator('pinned-users', {
	noPaging: true,
}));
const popularUsersPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		state: 'alive',
		origin: 'local',
		sort: '+follower',
	},
}));
const recentlyUpdatedUsersPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		origin: 'local',
		sort: '+updatedAt',
	},
}));
const recentlyRegisteredUsersPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		origin: 'local',
		state: 'alive',
		sort: '+createdAt',
	},
}));
const popularUsersFPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		state: 'alive',
		origin: 'remote',
		sort: '+follower',
	},
}));
const recentlyUpdatedUsersFPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		origin: 'combined',
		sort: '+updatedAt',
	},
}));
const recentlyRegisteredUsersFPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		origin: 'combined',
		sort: '+createdAt',
	},
}));
misskeyApi('hashtags/list', {
	sort: '+attachedLocalUsers',
	attachedToLocalUserOnly: true,
	limit: 30,
}).then(tags => {
	tagsLocal.value = tags;
});
misskeyApi('hashtags/list', {
	sort: '+attachedRemoteUsers',
	attachedToRemoteUserOnly: true,
	limit: 30,
}).then(tags => {
	tagsRemote.value = tags;
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 1200px;"
    }, [ (_unref(instance).federation !== 'none') ? (_openBlock(), _createBlock(MkTab, {
          key: 0,
          tabs: [
  			{ key: 'local', label: _unref(i18n).ts.local },
  			{ key: 'remote', label: _unref(i18n).ts.remote },
  		],
          style: "margin-bottom: var(--MI-margin);",
          modelValue: origin.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((origin).value = $event))
        }, null, 8 /* PROPS */, ["tabs", "modelValue"])) : _createCommentVNode("v-if", true), (origin.value === 'local') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (__props.tag == null) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(MkFoldableSection, {
                class: "_margin",
                persistKey: "explore-pinned-users"
              }, {
                header: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.pinnedUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(pinnedUsersPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkFoldableSection, {
                class: "_margin",
                persistKey: "explore-popular-users"
              }, {
                header: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.popularUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(popularUsersPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkFoldableSection, {
                class: "_margin",
                persistKey: "explore-recently-updated-users"
              }, {
                header: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.recentlyUpdatedUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(recentlyUpdatedUsersPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkFoldableSection, {
                class: "_margin",
                persistKey: "explore-recently-registered-users"
              }, {
                header: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.recentlyRegisteredUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(recentlyRegisteredUsersPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createVNode(MkFoldableSection, {
            foldable: true,
            expanded: false,
            class: "_margin"
          }, {
            header: _withCtx(() => [
              _hoisted_5,
              _createTextVNode(_toDisplayString(_unref(i18n).ts.popularTags), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", null, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tagsLocal.value, (tag) => {
                  return (_openBlock(), _createBlock(_component_MkA, {
                    key: 'local:' + tag.tag,
                    to: `/user-tags/${tag.tag}`,
                    style: "margin-right: 16px; font-weight: bold;"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(tag.tag), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                }), 128 /* KEYED_FRAGMENT */)),
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tagsRemote.value, (tag) => {
                  return (_openBlock(), _createBlock(_component_MkA, {
                    key: 'remote:' + tag.tag,
                    to: `/user-tags/${tag.tag}`,
                    style: "margin-right: 16px;"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(tag.tag), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["foldable", "expanded"]), (_unref(tagUsersPaginator) != null) ? (_openBlock(), _createBlock(MkFoldableSection, {
              key: `${__props.tag}`,
              class: "_margin"
            }, {
              header: _withCtx(() => [
                _hoisted_6,
                _createTextVNode(_toDisplayString(__props.tag), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(MkUserList, { paginator: _unref(tagUsersPaginator) }, null, 8 /* PROPS */, ["paginator"])
              ]),
              _: 1 /* STABLE */
            })) : _createCommentVNode("v-if", true), (__props.tag == null) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(MkFoldableSection, { class: "_margin" }, {
                header: _withCtx(() => [
                  _hoisted_7,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.popularUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(popularUsersFPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkFoldableSection, { class: "_margin" }, {
                header: _withCtx(() => [
                  _hoisted_8,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.recentlyUpdatedUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(recentlyUpdatedUsersFPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkFoldableSection, { class: "_margin" }, {
                header: _withCtx(() => [
                  _hoisted_9,
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.recentlyDiscoveredUsers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkUserList, { paginator: _unref(recentlyRegisteredUsersFPaginator) }, null, 8 /* PROPS */, ["paginator"])
                ]),
                _: 1 /* STABLE */
              }) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ])) ]))
}
}

})
