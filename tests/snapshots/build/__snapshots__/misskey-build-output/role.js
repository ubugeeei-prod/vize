import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, watch, ref, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkUserList from '@/components/MkUserList.vue'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'role',
  props: {
    roleId: { type: String, required: true },
    initialTab: { type: String, required: false, default: 'users' }
  },
  setup(__props: any) {

const props = __props
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const tab = ref(props.initialTab);
const role = ref<Misskey.entities.Role | null>(null);
const error = ref<string | null>(null);
const visible = ref(false);
watch(() => props.roleId, () => {
	misskeyApi('roles/show', {
		roleId: props.roleId,
	}).then(res => {
		role.value = res;
		error.value = null;
		visible.value = res.isExplorable && res.isPublic;
	}).catch((err) => {
		if (err.code === 'NO_SUCH_ROLE') {
			error.value = i18n.ts.noRole;
		} else {
			error.value = i18n.ts.somethingHappened;
		}
	});
}, { immediate: true });
const usersPaginator = markRaw(new Paginator('roles/users', {
	limit: 30,
	computedParams: computed(() => ({
		roleId: props.roleId,
	})),
}));
const headerTabs = computed(() => [{
	key: 'users',
	icon: 'ti ti-users',
	title: i18n.ts.users,
}, {
	key: 'timeline',
	icon: 'ti ti-pencil',
	title: i18n.ts.timeline,
}]);
definePage(() => ({
	title: role.value ? role.value.name : (error.value ?? i18n.ts.role),
	icon: 'ti ti-badge',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        (error.value != null)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 1200px;"
          }, [
            _createVNode(_component_MkResult, {
              type: "error",
              text: error.value
            }, null, 8 /* PROPS */, ["text"])
          ]))
          : (tab.value === 'users')
            ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "_spacer",
              style: "--MI_SPACER-w: 1200px;"
            }, [
              _createElementVNode("div", { class: "_gaps_s" }, [
                (role.value)
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(role.value.description), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true),
                (visible.value)
                  ? (_openBlock(), _createBlock(MkUserList, {
                    key: 0,
                    paginator: _unref(usersPaginator),
                    extractor: (item) => item.user
                  }, null, 8 /* PROPS */, ["paginator", "extractor"]))
                  : (!visible.value)
                    ? (_openBlock(), _createBlock(_component_MkResult, {
                      key: 1,
                      type: "empty",
                      text: _unref(i18n).ts.nothing
                    }, null, 8 /* PROPS */, ["text"]))
                  : _createCommentVNode("v-if", true)
              ])
            ]))
          : (tab.value === 'timeline')
            ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "_spacer",
              style: "--MI_SPACER-w: 700px;"
            }, [
              (visible.value)
                ? (_openBlock(), _createBlock(MkStreamingNotesTimeline, {
                  key: 0,
                  ref: "timeline",
                  src: "role",
                  role: props.roleId
                }, null, 8 /* PROPS */, ["role"]))
                : (!visible.value)
                  ? (_openBlock(), _createBlock(_component_MkResult, {
                    key: 1,
                    type: "empty",
                    text: _unref(i18n).ts.nothing
                  }, null, 8 /* PROPS */, ["text"]))
                : _createCommentVNode("v-if", true)
            ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["tabs", "tab"]))
}
}

})
