import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { computed, watch, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import MkWindow from '@/components/MkWindow.vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkFolder from '@/components/MkFolder.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import MkSwitch from '@/components/MkSwitch.vue'
import MkRolePreview from '@/components/MkRolePreview.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'avatar-decoration-edit-dialog',
  props: {
    avatarDecoration: { type: null, required: false }
  },
  emits: ["done", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const windowEl = useTemplateRef('windowEl');
const url = ref<string>(props.avatarDecoration ? props.avatarDecoration.url : '');
const name = ref<string>(props.avatarDecoration ? props.avatarDecoration.name : '');
const description = ref<string>(props.avatarDecoration ? props.avatarDecoration.description : '');
const roleIdsThatCanBeUsedThisDecoration = ref(props.avatarDecoration ? props.avatarDecoration.roleIdsThatCanBeUsedThisDecoration : []);
const rolesThatCanBeUsedThisDecoration = ref<Misskey.entities.Role[]>([]);
watch(roleIdsThatCanBeUsedThisDecoration, async () => {
	rolesThatCanBeUsedThisDecoration.value = (await Promise.all(roleIdsThatCanBeUsedThisDecoration.value.map((id) => misskeyApi('admin/roles/show', { roleId: id }).catch(() => null)))).filter(x => x != null);
}, { immediate: true });
async function addRole() {
	const roles = await misskeyApi('admin/roles/list');
	const currentRoleIds = rolesThatCanBeUsedThisDecoration.value.map(x => x.id);
	const { canceled, result: roleId } = await os.select({
		items: roles.filter(r => r.isPublic).filter(r => !currentRoleIds.includes(r.id)).map(r => ({ label: r.name, value: r.id })),
	});
	if (canceled || roleId == null) return;
	rolesThatCanBeUsedThisDecoration.value.push(roles.find(r => r.id === roleId)!);
}
async function removeRole(role: Misskey.entities.Role, ev: PointerEvent) {
	rolesThatCanBeUsedThisDecoration.value = rolesThatCanBeUsedThisDecoration.value.filter(x => x.id !== role.id);
}
async function done() {
	const params = {
		url: url.value,
		name: name.value,
		description: description.value,
		roleIdsThatCanBeUsedThisDecoration: rolesThatCanBeUsedThisDecoration.value.map(x => x.id),
	};
	if (props.avatarDecoration) {
		await os.apiWithDialog('admin/avatar-decorations/update', {
			id: props.avatarDecoration.id,
			...params,
		});
		emit('done', {
			updated: {
				id: props.avatarDecoration.id,
				...params,
			},
		});
		windowEl.value?.close();
	} else {
		const created = await os.apiWithDialog('admin/avatar-decorations/create', params);
		emit('done', {
			created: created,
		});
		windowEl.value?.close();
	}
}
async function del() {
	if (props.avatarDecoration == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: name.value }),
	});
	if (canceled) return;
	misskeyApi('admin/avatar-decorations/delete', {
		id: props.avatarDecoration.id,
	}).then(() => {
		emit('done', {
			deleted: true,
		});
		windowEl.value?.close();
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")

  return (_openBlock(), _createBlock(MkWindow, {
      ref_key: "windowEl", ref: windowEl,
      initialWidth: 400,
      initialHeight: 500,
      canResize: true,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (_unref(windowEl)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (__props.avatarDecoration) ? {
          name: "header",
          fn: _withCtx(() => [
            _createTextVNode(_toDisplayString(__props.avatarDecoration.name), 1 /* TEXT */)
          ]),
          key: "0"
        } : {
        name: "header",
        fn: _withCtx(() => [
          _createTextVNode("New decoration")
        ]),
        key: "1"
      } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["initialWidth", "initialHeight", "canResize"]))
}
}

})
