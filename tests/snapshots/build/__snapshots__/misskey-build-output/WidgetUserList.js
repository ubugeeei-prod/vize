import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useInterval } from '@@/js/use-interval.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'userList';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetUserList',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	listId: {
		type: 'string',
		default: null as string | null,
		hidden: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure, save } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const list = ref<Misskey.entities.UserList | null>(null);
const users = ref<Misskey.entities.UserDetailed[]>([]);
const fetching = ref(true);
async function chooseList() {
	const lists = await misskeyApi('users/lists/list');
	const { canceled, result: listId } = await os.select({
		title: i18n.ts.selectList,
		items: lists.map(x => ({
			value: x.id, label: x.name,
		})),
		default: widgetProps.listId,
	});
	if (canceled || listId == null) return;
	const list = lists.find(x => x.id === listId)!;
	widgetProps.listId = list.id;
	save();
	fetch();
}
const fetch = () => {
	if (widgetProps.listId == null) {
		fetching.value = false;
		return;
	}

	misskeyApi('users/lists/show', {
		listId: widgetProps.listId,
	}).then(_list => {
		list.value = _list;
		misskeyApi('users/show', {
			userIds: list.value.userIds ?? [],
		}).then(_users => {
			users.value = _users;
			fetching.value = false;
		});
	});
};
useInterval(fetch, 1000 * 60, {
	immediate: true,
	afterMounted: true,
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkAvatar = _resolveComponent("MkAvatar")

  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      class: "mkw-userList"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(list.value ? list.value.name : _unref(i18n).ts._widgets.userList), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(configure)()))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          (_unref(widgetProps).listId == null)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "init"
            }, [
              _createVNode(MkButton, {
                primary: "",
                onClick: chooseList
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets._userList.chooseList), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : (fetching.value)
              ? (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "users"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(users.value, (user) => {
                return (_openBlock(), _createElementBlock("span", {
                  key: user.id,
                  class: "user"
                }, [
                  _createVNode(_component_MkAvatar, {
                    user: user,
                    class: "avatar",
                    indicator: "",
                    link: "",
                    preview: ""
                  }, null, 8 /* PROPS */, ["user"])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
