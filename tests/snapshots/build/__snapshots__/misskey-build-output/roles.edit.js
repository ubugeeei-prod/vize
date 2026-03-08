import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { computed, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XEditor from './roles.editor.vue'
import { genId } from '@/utility/id.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import { rolesCache } from '@/cache.js'
import { useRouter } from '@/router.js'

type RoleLike = Pick<Misskey.entities.Role, 'name' | 'description' | 'isAdministrator' | 'isModerator' | 'color' | 'iconUrl' | 'target' | 'isPublic' | 'isExplorable' | 'asBadge' | 'canEditMembersByModerator' | 'displayOrder' | 'preserveAssignmentOnMoveAccount'> & {
	condFormula: any;
	policies: any;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'roles.edit',
  props: {
    id: { type: String, required: false }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const props = __props
const router = useRouter();
const role = ref<Misskey.entities.Role | null>(null);
const data = ref<RoleLike | null>(null);
if (props.id) {
	role.value = await misskeyApi('admin/roles/show', {
		roleId: props.id,
	});
	data.value = role.value;
} else {
	data.value = {
		name: 'New Role',
		description: '',
		isAdministrator: false,
		isModerator: false,
		color: null,
		iconUrl: null,
		target: 'manual',
		condFormula: { id: genId(), type: 'isRemote' },
		isPublic: false,
		isExplorable: false,
		asBadge: false,
		canEditMembersByModerator: false,
		displayOrder: 0,
		preserveAssignmentOnMoveAccount: false,
		policies: {},
	};
}
async function save() {
	if (data.value === null) return;
	rolesCache.delete();
	if (role.value) {
		os.apiWithDialog('admin/roles/update', {
			roleId: role.value.id,
			...data.value,
		});
		router.push('/admin/roles/:id', {
			params: {
				id: role.value.id,
			},
		});
	} else {
		const created = await os.apiWithDialog('admin/roles/create', {
			...data.value,
		});
		router.push('/admin/roles/:id', {
			params: {
				id: created.id,
			},
		});
	}
}
const headerTabs = computed(() => []);
definePage(() => ({
	title: role.value ? `${i18n.ts._role.edit}: ${role.value.name}` : i18n.ts._role.new,
	icon: 'ti ti-badge',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, { tabs: headerTabs.value }, {
      footer: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footer)
        }, [
          _createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
          }, [
            _createVNode(MkButton, {
              primary: "",
              rounded: "",
              onClick: save
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          (data.value)
            ? (_openBlock(), _createBlock(XEditor, {
              key: 0,
              modelValue: data.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((data).value = $event))
            }, null, 8 /* PROPS */, ["modelValue"]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["tabs"]))
}
}

})
