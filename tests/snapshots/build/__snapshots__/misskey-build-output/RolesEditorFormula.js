import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-menu-2" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { computed, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import type { GetMkSelectValueTypesFromDef, MkSelectItem } from '@/components/MkSelect.vue'
import { genId } from '@/utility/id.js'
import MkInput from '@/components/MkInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkButton from '@/components/MkButton.vue'
import MkDraggable from '@/components/MkDraggable.vue'
import { i18n } from '@/i18n.js'
import { deepClone } from '@/utility/clone.js'
import { rolesCache } from '@/cache.js'

type KeyOfUnion<T> = T extends T ? keyof T : never;
type DistributiveOmit<T, K extends KeyOfUnion<T>> = T extends T
	? Omit<T, K>
	: never;

export default /*@__PURE__*/_defineComponent({
  __name: 'RolesEditorFormula',
  props: {
    modelValue: { type: null, required: true },
    draggable: { type: Boolean, required: false },
    dragStartCallback: { type: Function, required: false }
  },
  emits: ["update:modelValue", "condFormula", "remove"],
  async setup(__props: any, { emit: __emit }) {

let __temp: any, __restore: any

const emit = __emit
const props = __props
const v = ref(deepClone(props.modelValue));
const roles = await rolesCache.fetch();
watch(() => props.modelValue, () => {
	if (JSON.stringify(props.modelValue) === JSON.stringify(v.value)) return;
	v.value = deepClone(props.modelValue);
}, { deep: true });
watch(v, () => {
	emit('update:modelValue', v.value);
}, { deep: true });
const typeDef = [
	{ label: i18n.ts._role._condition.isLocal, value: 'isLocal' },
	{ label: i18n.ts._role._condition.isRemote, value: 'isRemote' },
	{ label: i18n.ts._role._condition.isSuspended, value: 'isSuspended' },
	{ label: i18n.ts._role._condition.isLocked, value: 'isLocked' },
	{ label: i18n.ts._role._condition.isBot, value: 'isBot' },
	{ label: i18n.ts._role._condition.isCat, value: 'isCat' },
	{ label: i18n.ts._role._condition.isExplorable, value: 'isExplorable' },
	{ label: i18n.ts._role._condition.roleAssignedTo, value: 'roleAssignedTo' },
	{ label: i18n.ts._role._condition.createdLessThan, value: 'createdLessThan' },
	{ label: i18n.ts._role._condition.createdMoreThan, value: 'createdMoreThan' },
	{ label: i18n.ts._role._condition.followersLessThanOrEq, value: 'followersLessThanOrEq' },
	{ label: i18n.ts._role._condition.followersMoreThanOrEq, value: 'followersMoreThanOrEq' },
	{ label: i18n.ts._role._condition.followingLessThanOrEq, value: 'followingLessThanOrEq' },
	{ label: i18n.ts._role._condition.followingMoreThanOrEq, value: 'followingMoreThanOrEq' },
	{ label: i18n.ts._role._condition.notesLessThanOrEq, value: 'notesLessThanOrEq' },
	{ label: i18n.ts._role._condition.notesMoreThanOrEq, value: 'notesMoreThanOrEq' },
	{ label: i18n.ts._role._condition.and, value: 'and' },
	{ label: i18n.ts._role._condition.or, value: 'or' },
	{ label: i18n.ts._role._condition.not, value: 'not' },
] as const satisfies MkSelectItem[];
const typeModelForMkSelect = computed<GetMkSelectValueTypesFromDef<typeof typeDef>>({
	get: () => v.value.type,
	set: (t) => {
		let newValue: DistributiveOmit<Misskey.entities.Role['condFormula'], 'id'>;
		switch (t) {
			case 'and': newValue = { type: 'and', values: [] }; break;
			case 'or': newValue = { type: 'or', values: [] }; break;
			case 'not': newValue = { type: 'not', value: { id: genId(), type: 'isRemote' } }; break;
			case 'roleAssignedTo': newValue = { type: 'roleAssignedTo', roleId: '' }; break;
			case 'createdLessThan': newValue = { type: 'createdLessThan', sec: 86400 }; break;
			case 'createdMoreThan': newValue = { type: 'createdMoreThan', sec: 86400 }; break;
			case 'followersLessThanOrEq': newValue = { type: 'followersLessThanOrEq', value: 10 }; break;
			case 'followersMoreThanOrEq': newValue = { type: 'followersMoreThanOrEq', value: 10 }; break;
			case 'followingLessThanOrEq': newValue = { type: 'followingLessThanOrEq', value: 10 }; break;
			case 'followingMoreThanOrEq': newValue = { type: 'followingMoreThanOrEq', value: 10 }; break;
			case 'notesLessThanOrEq': newValue = { type: 'notesLessThanOrEq', value: 10 }; break;
			case 'notesMoreThanOrEq': newValue = { type: 'notesMoreThanOrEq', value: 10 }; break;
			default: newValue = { type: t }; break;
		}
		v.value = { id: v.value.id, ...newValue };
	},
});
const assignedToDef = computed(() => roles.filter(r => r.target === 'manual').map(r => ({ label: r.name, value: r.id })) satisfies MkSelectItem[]);
function addChildValue() {
	if (v.value.type !== 'and' && v.value.type !== 'or') return;
	v.value.values.push({ id: genId(), type: 'isRemote' });
}
function childValuesItemUpdated(item: Misskey.entities.Role['condFormula']) {
	if (v.value.type !== 'and' && v.value.type !== 'or') return;
	const i = v.value.values.findIndex(_item => _item.id === item.id);
	v.value.values[i] = item;
}
function removeChildItem(itemId: string) {
	if (v.value.type !== 'and' && v.value.type !== 'or') return;
	v.value.values = v.value.values.filter(_item => _item.id !== itemId);
}
function removeSelf() {
	emit('remove');
}

return (_ctx: any,_cache: any) => {
  const _component_RolesEditorFormula = _resolveComponent("RolesEditorFormula")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.header)
      }, [ _createVNode(MkSelect, {
          items: _unref(typeDef),
          class: _normalizeClass(_ctx.$style.typeSelect),
          modelValue: typeModelForMkSelect.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((typeModelForMkSelect).value = $event))
        }, null, 8 /* PROPS */, ["items", "modelValue"]), (__props.draggable) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            class: _normalizeClass(["_button", _ctx.$style.dragHandle]),
            draggable: true,
            onDragstart: _cache[1] || (_cache[1] = _withModifiers(($event: any) => (__props.dragStartCallback), ["stop"]))
          }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), (__props.draggable) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            class: _normalizeClass(["_button", _ctx.$style.remove]),
            onClick: removeSelf
          }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]), (v.value.type === 'and' || v.value.type === 'or') ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps"
        }, [ _createVNode(MkDraggable, {
            direction: "vertical",
            withGaps: "",
            canNest: "",
            manualDragStart: "",
            group: "roleFormula",
            modelValue: v.value.values,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((v.value.values) = $event))
          }, {
            default: _withCtx(({ item, dragStart }) => [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.item)
              }, [
                _createVNode(_component_RolesEditorFormula, {
                  modelValue: item,
                  dragStartCallback: dragStart,
                  draggable: "",
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = updated => childValuesItemUpdated(updated)),
                  onRemove: _cache[4] || (_cache[4] = ($event: any) => (removeChildItem(_ctx.item.id)))
                }, null, 8 /* PROPS */, ["modelValue", "dragStartCallback"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkButton, {
            rounded: "",
            style: "margin: 0 auto;",
            onClick: addChildValue
          }, {
            default: _withCtx(() => [
              _hoisted_3,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ])) : (v.value.type === 'not') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: _normalizeClass(_ctx.$style.item)
          }, [ _createVNode(_component_RolesEditorFormula, {
              modelValue: v.value,
              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((v.value) = $event))
            }, null, 8 /* PROPS */, ["modelValue"]) ])) : (v.value.type === 'createdLessThan' || v.value.type === 'createdMoreThan') ? (_openBlock(), _createBlock(MkInput, {
            key: 2,
            type: "number",
            modelValue: v.value.sec,
            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((v.value.sec) = $event))
          }, {
            suffix: _withCtx(() => [
              _createTextVNode("sec")
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])) : (v.value.type === 'followersLessThanOrEq' || v.value.type === 'followersMoreThanOrEq' || v.value.type === 'followingLessThanOrEq' || v.value.type === 'followingMoreThanOrEq' || v.value.type === 'notesLessThanOrEq' || v.value.type === 'notesMoreThanOrEq') ? (_openBlock(), _createBlock(MkInput, {
            key: 3,
            type: "number",
            modelValue: v.value,
            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((v.value) = $event))
          }, null, 8 /* PROPS */, ["modelValue"])) : (v.value.type === 'roleAssignedTo') ? (_openBlock(), _createBlock(MkSelect, {
            key: 4,
            items: assignedToDef.value,
            modelValue: v.value.roleId,
            "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((v.value.roleId) = $event))
          }, null, 8 /* PROPS */, ["items", "modelValue"])) : _createCommentVNode("v-if", true) ]))
}
}

})
