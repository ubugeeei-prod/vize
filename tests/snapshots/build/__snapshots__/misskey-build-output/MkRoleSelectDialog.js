import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { computed, ref, toRefs, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkRolePreview from '@/components/MkRolePreview.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import * as os from '@/os.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkLoading from '@/components/global/MkLoading.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRoleSelectDialog',
  props: {
    initialRoleIds: { type: Array, required: false, default: undefined },
    infoMessage: { type: String, required: false, default: undefined },
    title: { type: String, required: false, default: undefined },
    publicOnly: { type: Boolean, required: true, default: true }
  },
  emits: ["done", "close", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { initialRoleIds, infoMessage, title, publicOnly } = toRefs(props);
const windowEl = useTemplateRef('windowEl');
const roles = ref<Misskey.entities.Role[]>([]);
const selectedRoleIds = ref<string[]>(initialRoleIds.value ?? []);
const fetching = ref(false);
const selectedRoles = computed(() => {
	const r = roles.value.filter(role => selectedRoleIds.value.includes(role.id));
	r.sort((a, b) => {
		if (a.displayOrder !== b.displayOrder) {
			return b.displayOrder - a.displayOrder;
		}

		return a.id.localeCompare(b.id);
	});
	return r;
});
async function fetchRoles() {
	fetching.value = true;
	const result = await misskeyApi('admin/roles/list', {});
	roles.value = result.filter(it => publicOnly.value ? it.isPublic : true);
	fetching.value = false;
}
async function addRole() {
	const items = roles.value
		.filter(r => r.isPublic)
		.filter(r => !selectedRoleIds.value.includes(r.id))
		.map(r => ({ label: r.name, value: r.id }));
	const { canceled, result: roleId } = await os.select({ items });
	if (canceled || roleId == null) return;
	selectedRoleIds.value.push(roleId);
}
async function removeRole(roleId: string) {
	selectedRoleIds.value = selectedRoleIds.value.filter(x => x !== roleId);
}
function onOkClicked() {
	emit('done', selectedRoles.value);
	windowEl.value?.close();
}
function onCancelClicked() {
	emit('close');
	windowEl.value?.close();
}
function onCloseModalWindow() {
	emit('close');
	windowEl.value?.close();
}
fetchRoles();

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "windowEl", ref: windowEl,
      withOkButton: false,
      okButtonDisabled: false,
      width: 400,
      height: 500,
      onClose: onCloseModalWindow,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(title)), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(["_gaps", _ctx.$style.root])
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.header)
              }, [
                _createVNode(MkButton, {
                  rounded: "",
                  onClick: addRole
                }, {
                  default: _withCtx(() => [
                    _hoisted_1,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              (selectedRoles.value.length > 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(["_gaps", _ctx.$style.roleItemArea])
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(selectedRoles.value, (role) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: role.id,
                      class: _normalizeClass(_ctx.$style.roleItem)
                    }, [
                      _createVNode(MkRolePreview, {
                        class: _normalizeClass(_ctx.$style.role),
                        role: role,
                        forModeration: true,
                        detailed: false,
                        style: "pointer-events: none;"
                      }, null, 8 /* PROPS */, ["role", "forModeration", "detailed"]),
                      _createElementVNode("button", {
                        class: _normalizeClass(["_button", _ctx.$style.roleUnAssign]),
                        onClick: _cache[1] || (_cache[1] = ($event: any) => (removeRole(role.id)))
                      }, [
                        _hoisted_2
                      ])
                    ]))
                  }), 128 /* KEYED_FRAGMENT */))
                ]))
                : (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: _normalizeClass(_ctx.$style.roleItemArea),
                  style: "text-align: center"
                }, _toDisplayString(_unref(i18n).ts._roleSelectDialog.notSelected), 1 /* TEXT */)),
              (_unref(infoMessage))
                ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(infoMessage)), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.buttons)
              }, [
                _createVNode(MkButton, {
                  primary: "",
                  onClick: onOkClicked
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.ok), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkButton, { onClick: onCancelClicked }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["withOkButton", "okButtonDisabled", "width", "height"]))
}
}

})
