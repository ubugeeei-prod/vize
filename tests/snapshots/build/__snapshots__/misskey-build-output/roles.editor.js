import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-shield-lock" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-license" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
import { watch, ref, computed } from 'vue'
import { throttle } from 'throttle-debounce'
import * as Misskey from 'misskey-js'
import RolesEditorFormula from './RolesEditorFormula.vue'
import type { MkSelectItem, GetMkSelectValueTypesFromDef } from '@/components/MkSelect.vue'
import MkInput from '@/components/MkInput.vue'
import MkColorInput from '@/components/MkColorInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkRange from '@/components/MkRange.vue'
import FormSlot from '@/components/form/slot.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { deepClone } from '@/utility/clone.js'

type RoleLike = Pick<Misskey.entities.Role, 'name' | 'description' | 'isAdministrator' | 'isModerator' | 'color' | 'iconUrl' | 'target' | 'isPublic' | 'isExplorable' | 'asBadge' | 'canEditMembersByModerator' | 'displayOrder' | 'preserveAssignmentOnMoveAccount'> & {
	id?: Misskey.entities.Role['id'] | null;
	condFormula: any;
	policies: any;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'roles.editor',
  props: {
    modelValue: { type: Object, required: true },
    readonly: { type: Boolean, required: false }
  },
  emits: ["update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const role = ref(deepClone(props.modelValue));
// fill missing policy
for (const ROLE_POLICY of Misskey.rolePolicies) {
	if (role.value.policies[ROLE_POLICY] == null) {
		role.value.policies[ROLE_POLICY] = {
			useDefault: true,
			priority: 0,
			value: instance.policies[ROLE_POLICY],
		};
	}
}
function updateAvatarDecorationLimit(value: string | number) {
	const numValue = Number(value);
	const limited = Math.min(16, Math.max(0, numValue));
	role.value.policies.avatarDecorationLimit.value = limited;
}
const rolePermissionDef = [
	{ label: i18n.ts.normalUser, value: 'normal' },
	{ label: i18n.ts.moderator, value: 'moderator' },
	{ label: i18n.ts.administrator, value: 'administrator' },
] as const satisfies MkSelectItem[];
const rolePermission = computed<GetMkSelectValueTypesFromDef<typeof rolePermissionDef>>({
	get: () => role.value.isAdministrator ? 'administrator' : role.value.isModerator ? 'moderator' : 'normal',
	set: (val) => {
		role.value.isAdministrator = (val === 'administrator');
		role.value.isModerator = (val === 'moderator');
	},
});
const q = ref('');
function getPriorityIcon(option: { priority: number }): string {
	if (option.priority === 2) return 'ti ti-arrows-up';
	if (option.priority === 1) return 'ti ti-arrow-narrow-up';
	return 'ti ti-point';
}
function matchQuery(keywords: string[]): boolean {
	if (q.value.trim().length === 0) return true;
	return keywords.some(keyword => keyword.toLowerCase().includes(q.value.toLowerCase()));
}
const save = throttle(100, () => {
	const data = {
		name: role.value.name,
		description: role.value.description,
		color: role.value.color === '' ? null : role.value.color,
		iconUrl: role.value.iconUrl === '' ? null : role.value.iconUrl,
		displayOrder: role.value.displayOrder,
		target: role.value.target,
		condFormula: role.value.condFormula,
		isAdministrator: role.value.isAdministrator,
		isModerator: role.value.isModerator,
		isPublic: role.value.isPublic,
		isExplorable: role.value.isExplorable,
		asBadge: role.value.asBadge,
		canEditMembersByModerator: role.value.canEditMembersByModerator,
		preserveAssignmentOnMoveAccount: role.value.preserveAssignmentOnMoveAccount,
		policies: role.value.policies,
	};

	emit('update:modelValue', data);
});
watch(role, save, { deep: true });

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (__props.readonly && role.value.id != null) ? (_openBlock(), _createBlock(MkInput, {
          key: 0,
          modelValue: role.value.id,
          readonly: true
        }, {
          label: _withCtx(() => [
            _createTextVNode("ID")
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue", "readonly"])) : _createCommentVNode("v-if", true), _createVNode(MkInput, {
        readonly: __props.readonly,
        modelValue: role.value.name,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((role.value.name) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.name), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkTextarea, {
        readonly: __props.readonly,
        modelValue: role.value.description,
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((role.value.description) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.description), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkColorInput, {
        modelValue: role.value.color,
        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((role.value.color) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.color), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
        type: "url",
        modelValue: role.value.iconUrl,
        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((role.value.iconUrl) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.iconUrl), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
        type: "number",
        modelValue: role.value.displayOrder,
        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((role.value.displayOrder) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.displayOrder), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.descriptionOfDisplayOrder), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSelect, {
        items: _unref(rolePermissionDef),
        readonly: __props.readonly,
        modelValue: rolePermission.value,
        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((rolePermission).value = $event))
      }, {
        label: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.permission), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createElementVNode("div", { innerHTML: _unref(i18n).ts._role.descriptionOfPermission.replaceAll('\n', '<br>') }, null, 8 /* PROPS */, ["innerHTML"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["items", "readonly", "modelValue"]), _createVNode(MkSelect, {
        items: [{ label: _unref(i18n).ts._role.manual, value: 'manual' }, { label: _unref(i18n).ts._role.conditional, value: 'conditional' }],
        readonly: __props.readonly,
        modelValue: role.value.target,
        "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((role.value.target) = $event))
      }, {
        label: _withCtx(() => [
          _hoisted_2,
          _createTextVNode(" "),
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.assignTarget), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createElementVNode("div", { innerHTML: _unref(i18n).ts._role.descriptionOfAssignTarget.replaceAll('\n', '<br>') }, null, 8 /* PROPS */, ["innerHTML"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["items", "readonly", "modelValue"]), (role.value.target === 'conditional') ? (_openBlock(), _createBlock(MkFolder, {
          key: 0,
          defaultOpen: ""
        }, {
          label: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._role.condition), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_gaps" }, [
              _createVNode(RolesEditorFormula, {
                modelValue: role.value.condFormula,
                "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((role.value.condFormula) = $event))
              }, null, 8 /* PROPS */, ["modelValue"])
            ])
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createVNode(MkSwitch, {
        readonly: __props.readonly,
        modelValue: role.value.preserveAssignmentOnMoveAccount,
        "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((role.value.preserveAssignmentOnMoveAccount) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.preserveAssignmentOnMoveAccount), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.preserveAssignmentOnMoveAccount_description), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkSwitch, {
        readonly: __props.readonly,
        modelValue: role.value.canEditMembersByModerator,
        "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((role.value.canEditMembersByModerator) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.canEditMembersByModerator), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.descriptionOfCanEditMembersByModerator), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkSwitch, {
        readonly: __props.readonly,
        modelValue: role.value.isPublic,
        "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((role.value.isPublic) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.isPublic), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.descriptionOfIsPublic), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkSwitch, {
        readonly: __props.readonly,
        modelValue: role.value.asBadge,
        "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((role.value.asBadge) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.asBadge), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.descriptionOfAsBadge), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(MkSwitch, {
        readonly: __props.readonly,
        modelValue: role.value.isExplorable,
        "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((role.value.isExplorable) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.isExplorable), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.descriptionOfIsExplorable), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["readonly", "modelValue"]), _createVNode(FormSlot, null, {
        label: _withCtx(() => [
          _hoisted_3,
          _createTextVNode(" "),
          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.policies), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(MkInput, {
              type: "search",
              modelValue: q.value,
              "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((q).value = $event))
            }, {
              prefix: _withCtx(() => [
                _hoisted_4
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            (matchQuery([_unref(i18n).ts._role._options.rateLimitFactor, 'rateLimitFactor']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.rateLimitFactor), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.rateLimitFactor.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(`${Math.floor(role.value.policies.rateLimitFactor.value * 100)}%`), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.rateLimitFactor))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.rateLimitFactor.useDefault,
                      "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((role.value.policies.rateLimitFactor.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      modelValue: role.value.policies.rateLimitFactor.value * 100,
                      min: 0,
                      max: 400,
                      step: 10,
                      textConverter: (v) => `${v}%`,
                      "onUpdate:modelValue": _cache[15] || (_cache[15] = v => role.value.policies.rateLimitFactor.value = (v / 100))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.rateLimitFactor), 1 /* TEXT */)
                      ]),
                      caption: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.descriptionOfRateLimitFactor), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue", "min", "max", "step", "textConverter"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.rateLimitFactor.priority,
                      "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event: any) => ((role.value.policies.rateLimitFactor.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.gtlAvailable, 'gtlAvailable']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.gtlAvailable), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.gtlAvailable.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.gtlAvailable.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.gtlAvailable))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.gtlAvailable.useDefault,
                      "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event: any) => ((role.value.policies.gtlAvailable.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.gtlAvailable.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.gtlAvailable.value,
                      "onUpdate:modelValue": _cache[18] || (_cache[18] = ($event: any) => ((role.value.policies.gtlAvailable.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.gtlAvailable.priority,
                      "onUpdate:modelValue": _cache[19] || (_cache[19] = ($event: any) => ((role.value.policies.gtlAvailable.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.ltlAvailable, 'ltlAvailable']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.ltlAvailable), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.ltlAvailable.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.ltlAvailable.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.ltlAvailable))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.ltlAvailable.useDefault,
                      "onUpdate:modelValue": _cache[20] || (_cache[20] = ($event: any) => ((role.value.policies.ltlAvailable.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.ltlAvailable.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.ltlAvailable.value,
                      "onUpdate:modelValue": _cache[21] || (_cache[21] = ($event: any) => ((role.value.policies.ltlAvailable.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.ltlAvailable.priority,
                      "onUpdate:modelValue": _cache[22] || (_cache[22] = ($event: any) => ((role.value.policies.ltlAvailable.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canPublicNote, 'canPublicNote']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canPublicNote), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canPublicNote.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canPublicNote.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canPublicNote))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canPublicNote.useDefault,
                      "onUpdate:modelValue": _cache[23] || (_cache[23] = ($event: any) => ((role.value.policies.canPublicNote.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canPublicNote.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canPublicNote.value,
                      "onUpdate:modelValue": _cache[24] || (_cache[24] = ($event: any) => ((role.value.policies.canPublicNote.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canPublicNote.priority,
                      "onUpdate:modelValue": _cache[25] || (_cache[25] = ($event: any) => ((role.value.policies.canPublicNote.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.chatAvailability, 'chatAvailability']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.chatAvailability), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.chatAvailability.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.chatAvailability.value === 'available' ? _unref(i18n).ts.yes : role.value.policies.chatAvailability.value === 'readonly' ? _unref(i18n).ts.readonly : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.chatAvailability))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.chatAvailability.useDefault,
                      "onUpdate:modelValue": _cache[26] || (_cache[26] = ($event: any) => ((role.value.policies.chatAvailability.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSelect, {
                      items: [
  							{ label: _unref(i18n).ts.enabled, value: 'available' },
  							{ label: _unref(i18n).ts.readonly, value: 'readonly' },
  							{ label: _unref(i18n).ts.disabled, value: 'unavailable' },
  						],
                      disabled: role.value.policies.chatAvailability.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.chatAvailability.value,
                      "onUpdate:modelValue": _cache[27] || (_cache[27] = ($event: any) => ((role.value.policies.chatAvailability.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["items", "disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.chatAvailability.priority,
                      "onUpdate:modelValue": _cache[28] || (_cache[28] = ($event: any) => ((role.value.policies.chatAvailability.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.mentionMax, 'mentionLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.mentionMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.mentionLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.mentionLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.mentionLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.mentionLimit.useDefault,
                      "onUpdate:modelValue": _cache[29] || (_cache[29] = ($event: any) => ((role.value.policies.mentionLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.mentionLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.mentionLimit.value,
                      "onUpdate:modelValue": _cache[30] || (_cache[30] = ($event: any) => ((role.value.policies.mentionLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.mentionLimit.priority,
                      "onUpdate:modelValue": _cache[31] || (_cache[31] = ($event: any) => ((role.value.policies.mentionLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canInvite, 'canInvite']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canInvite), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canInvite.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canInvite.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canInvite))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canInvite.useDefault,
                      "onUpdate:modelValue": _cache[32] || (_cache[32] = ($event: any) => ((role.value.policies.canInvite.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canInvite.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canInvite.value,
                      "onUpdate:modelValue": _cache[33] || (_cache[33] = ($event: any) => ((role.value.policies.canInvite.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canInvite.priority,
                      "onUpdate:modelValue": _cache[34] || (_cache[34] = ($event: any) => ((role.value.policies.canInvite.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.inviteLimit, 'inviteLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.inviteLimit), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.inviteLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.inviteLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.inviteLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteLimit.useDefault,
                      "onUpdate:modelValue": _cache[35] || (_cache[35] = ($event: any) => ((role.value.policies.inviteLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.inviteLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteLimit.value,
                      "onUpdate:modelValue": _cache[36] || (_cache[36] = ($event: any) => ((role.value.policies.inviteLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.inviteLimit.priority,
                      "onUpdate:modelValue": _cache[37] || (_cache[37] = ($event: any) => ((role.value.policies.inviteLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.inviteLimitCycle, 'inviteLimitCycle']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.inviteLimitCycle), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.inviteLimitCycle.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.inviteLimitCycle.value + _unref(i18n).ts._time.minute), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.inviteLimitCycle))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteLimitCycle.useDefault,
                      "onUpdate:modelValue": _cache[38] || (_cache[38] = ($event: any) => ((role.value.policies.inviteLimitCycle.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.inviteLimitCycle.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteLimitCycle.value,
                      "onUpdate:modelValue": _cache[39] || (_cache[39] = ($event: any) => ((role.value.policies.inviteLimitCycle.value) = $event))
                    }, {
                      suffix: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._time.minute), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.inviteLimitCycle.priority,
                      "onUpdate:modelValue": _cache[40] || (_cache[40] = ($event: any) => ((role.value.policies.inviteLimitCycle.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.inviteExpirationTime, 'inviteExpirationTime']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.inviteExpirationTime), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.inviteExpirationTime.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.inviteExpirationTime.value + _unref(i18n).ts._time.minute), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.inviteExpirationTime))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteExpirationTime.useDefault,
                      "onUpdate:modelValue": _cache[41] || (_cache[41] = ($event: any) => ((role.value.policies.inviteExpirationTime.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.inviteExpirationTime.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.inviteExpirationTime.value,
                      "onUpdate:modelValue": _cache[42] || (_cache[42] = ($event: any) => ((role.value.policies.inviteExpirationTime.value) = $event))
                    }, {
                      suffix: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._time.minute), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.inviteExpirationTime.priority,
                      "onUpdate:modelValue": _cache[43] || (_cache[43] = ($event: any) => ((role.value.policies.inviteExpirationTime.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canManageCustomEmojis, 'canManageCustomEmojis']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canManageCustomEmojis), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canManageCustomEmojis.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canManageCustomEmojis.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canManageCustomEmojis))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canManageCustomEmojis.useDefault,
                      "onUpdate:modelValue": _cache[44] || (_cache[44] = ($event: any) => ((role.value.policies.canManageCustomEmojis.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canManageCustomEmojis.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canManageCustomEmojis.value,
                      "onUpdate:modelValue": _cache[45] || (_cache[45] = ($event: any) => ((role.value.policies.canManageCustomEmojis.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canManageCustomEmojis.priority,
                      "onUpdate:modelValue": _cache[46] || (_cache[46] = ($event: any) => ((role.value.policies.canManageCustomEmojis.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canManageAvatarDecorations, 'canManageAvatarDecorations']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canManageAvatarDecorations), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canManageAvatarDecorations.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canManageAvatarDecorations.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canManageAvatarDecorations))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canManageAvatarDecorations.useDefault,
                      "onUpdate:modelValue": _cache[47] || (_cache[47] = ($event: any) => ((role.value.policies.canManageAvatarDecorations.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canManageAvatarDecorations.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canManageAvatarDecorations.value,
                      "onUpdate:modelValue": _cache[48] || (_cache[48] = ($event: any) => ((role.value.policies.canManageAvatarDecorations.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canManageAvatarDecorations.priority,
                      "onUpdate:modelValue": _cache[49] || (_cache[49] = ($event: any) => ((role.value.policies.canManageAvatarDecorations.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canSearchNotes, 'canSearchNotes']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canSearchNotes), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canSearchNotes.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canSearchNotes.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canSearchNotes))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canSearchNotes.useDefault,
                      "onUpdate:modelValue": _cache[50] || (_cache[50] = ($event: any) => ((role.value.policies.canSearchNotes.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canSearchNotes.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canSearchNotes.value,
                      "onUpdate:modelValue": _cache[51] || (_cache[51] = ($event: any) => ((role.value.policies.canSearchNotes.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canSearchNotes.priority,
                      "onUpdate:modelValue": _cache[52] || (_cache[52] = ($event: any) => ((role.value.policies.canSearchNotes.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canSearchUsers, 'canSearchUsers']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canSearchUsers), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canSearchUsers.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canSearchUsers.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canSearchUsers))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canSearchUsers.useDefault,
                      "onUpdate:modelValue": _cache[53] || (_cache[53] = ($event: any) => ((role.value.policies.canSearchUsers.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canSearchUsers.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canSearchUsers.value,
                      "onUpdate:modelValue": _cache[54] || (_cache[54] = ($event: any) => ((role.value.policies.canSearchUsers.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canSearchUsers.priority,
                      "onUpdate:modelValue": _cache[55] || (_cache[55] = ($event: any) => ((role.value.policies.canSearchUsers.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canUseTranslator, 'canUseTranslator']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canUseTranslator), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canUseTranslator.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canUseTranslator.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canUseTranslator))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canUseTranslator.useDefault,
                      "onUpdate:modelValue": _cache[56] || (_cache[56] = ($event: any) => ((role.value.policies.canUseTranslator.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canUseTranslator.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canUseTranslator.value,
                      "onUpdate:modelValue": _cache[57] || (_cache[57] = ($event: any) => ((role.value.policies.canUseTranslator.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canUseTranslator.priority,
                      "onUpdate:modelValue": _cache[58] || (_cache[58] = ($event: any) => ((role.value.policies.canUseTranslator.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.driveCapacity, 'driveCapacityMb']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.driveCapacity), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.driveCapacityMb.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.driveCapacityMb.value + 'MB'), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.driveCapacityMb))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.driveCapacityMb.useDefault,
                      "onUpdate:modelValue": _cache[59] || (_cache[59] = ($event: any) => ((role.value.policies.driveCapacityMb.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.driveCapacityMb.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.driveCapacityMb.value,
                      "onUpdate:modelValue": _cache[60] || (_cache[60] = ($event: any) => ((role.value.policies.driveCapacityMb.value) = $event))
                    }, {
                      suffix: _withCtx(() => [
                        _createTextVNode("MB")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.driveCapacityMb.priority,
                      "onUpdate:modelValue": _cache[61] || (_cache[61] = ($event: any) => ((role.value.policies.driveCapacityMb.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.maxFileSize, 'maxFileSizeMb']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.maxFileSize), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.maxFileSizeMb.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.maxFileSizeMb.value + 'MB'), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.maxFileSizeMb))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.maxFileSizeMb.useDefault,
                      "onUpdate:modelValue": _cache[62] || (_cache[62] = ($event: any) => ((role.value.policies.maxFileSizeMb.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.maxFileSizeMb.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.maxFileSizeMb.value,
                      "onUpdate:modelValue": _cache[63] || (_cache[63] = ($event: any) => ((role.value.policies.maxFileSizeMb.value) = $event))
                    }, {
                      suffix: _withCtx(() => [
                        _createTextVNode("MB")
                      ]),
                      caption: _withCtx(() => [
                        _createElementVNode("div", null, [
                          _hoisted_5,
                          _createTextVNode(" " + _toDisplayString(_unref(i18n).ts._role._options.maxFileSize_caption), 1 /* TEXT */)
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.maxFileSizeMb.priority,
                      "onUpdate:modelValue": _cache[64] || (_cache[64] = ($event: any) => ((role.value.policies.maxFileSizeMb.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.uploadableFileTypes, 'uploadableFileTypes']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.uploadableFileTypes), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.uploadableFileTypes.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, "...")),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.uploadableFileTypes))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.uploadableFileTypes.useDefault,
                      "onUpdate:modelValue": _cache[65] || (_cache[65] = ($event: any) => ((role.value.policies.uploadableFileTypes.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkTextarea, {
                      modelValue: role.value.policies.uploadableFileTypes.value.join('\n'),
                      disabled: role.value.policies.uploadableFileTypes.useDefault,
                      readonly: __props.readonly,
                      "onUpdate:modelValue": _cache[66] || (_cache[66] = ($event: any) => (role.value.policies.uploadableFileTypes.value = $event.split('\n')))
                    }, {
                      caption: _withCtx(() => [
                        _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._role._options.uploadableFileTypes_caption), 1 /* TEXT */),
                        _createElementVNode("div", null, [
                          _hoisted_6,
                          _createTextVNode(" " + _toDisplayString(_unref(i18n).tsx._role._options.uploadableFileTypes_caption2({ x: 'application/octet-stream' })), 1 /* TEXT */)
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue", "disabled", "readonly"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.uploadableFileTypes.priority,
                      "onUpdate:modelValue": _cache[67] || (_cache[67] = ($event: any) => ((role.value.policies.uploadableFileTypes.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.alwaysMarkNsfw, 'alwaysMarkNsfw']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.alwaysMarkNsfw), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.alwaysMarkNsfw.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.alwaysMarkNsfw.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.alwaysMarkNsfw))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.alwaysMarkNsfw.useDefault,
                      "onUpdate:modelValue": _cache[68] || (_cache[68] = ($event: any) => ((role.value.policies.alwaysMarkNsfw.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.alwaysMarkNsfw.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.alwaysMarkNsfw.value,
                      "onUpdate:modelValue": _cache[69] || (_cache[69] = ($event: any) => ((role.value.policies.alwaysMarkNsfw.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.alwaysMarkNsfw.priority,
                      "onUpdate:modelValue": _cache[70] || (_cache[70] = ($event: any) => ((role.value.policies.alwaysMarkNsfw.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canUpdateBioMedia, 'canUpdateBioMedia']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canUpdateBioMedia), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canUpdateBioMedia.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canUpdateBioMedia.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canUpdateBioMedia))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canUpdateBioMedia.useDefault,
                      "onUpdate:modelValue": _cache[71] || (_cache[71] = ($event: any) => ((role.value.policies.canUpdateBioMedia.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canUpdateBioMedia.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canUpdateBioMedia.value,
                      "onUpdate:modelValue": _cache[72] || (_cache[72] = ($event: any) => ((role.value.policies.canUpdateBioMedia.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canUpdateBioMedia.priority,
                      "onUpdate:modelValue": _cache[73] || (_cache[73] = ($event: any) => ((role.value.policies.canUpdateBioMedia.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.pinMax, 'pinLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.pinMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.pinLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.pinLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.pinLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.pinLimit.useDefault,
                      "onUpdate:modelValue": _cache[74] || (_cache[74] = ($event: any) => ((role.value.policies.pinLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.pinLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.pinLimit.value,
                      "onUpdate:modelValue": _cache[75] || (_cache[75] = ($event: any) => ((role.value.policies.pinLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.pinLimit.priority,
                      "onUpdate:modelValue": _cache[76] || (_cache[76] = ($event: any) => ((role.value.policies.pinLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.antennaMax, 'antennaLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.antennaMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.antennaLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.antennaLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.antennaLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.antennaLimit.useDefault,
                      "onUpdate:modelValue": _cache[77] || (_cache[77] = ($event: any) => ((role.value.policies.antennaLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.antennaLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.antennaLimit.value,
                      "onUpdate:modelValue": _cache[78] || (_cache[78] = ($event: any) => ((role.value.policies.antennaLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.antennaLimit.priority,
                      "onUpdate:modelValue": _cache[79] || (_cache[79] = ($event: any) => ((role.value.policies.antennaLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.wordMuteMax, 'wordMuteLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.wordMuteMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.wordMuteLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.wordMuteLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.wordMuteLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.wordMuteLimit.useDefault,
                      "onUpdate:modelValue": _cache[80] || (_cache[80] = ($event: any) => ((role.value.policies.wordMuteLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.wordMuteLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.wordMuteLimit.value,
                      "onUpdate:modelValue": _cache[81] || (_cache[81] = ($event: any) => ((role.value.policies.wordMuteLimit.value) = $event))
                    }, {
                      suffix: _withCtx(() => [
                        _createTextVNode("chars")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.wordMuteLimit.priority,
                      "onUpdate:modelValue": _cache[82] || (_cache[82] = ($event: any) => ((role.value.policies.wordMuteLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.webhookMax, 'webhookLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.webhookMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.webhookLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.webhookLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.webhookLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.webhookLimit.useDefault,
                      "onUpdate:modelValue": _cache[83] || (_cache[83] = ($event: any) => ((role.value.policies.webhookLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.webhookLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.webhookLimit.value,
                      "onUpdate:modelValue": _cache[84] || (_cache[84] = ($event: any) => ((role.value.policies.webhookLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.webhookLimit.priority,
                      "onUpdate:modelValue": _cache[85] || (_cache[85] = ($event: any) => ((role.value.policies.webhookLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.clipMax, 'clipLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.clipMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.clipLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.clipLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.clipLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.clipLimit.useDefault,
                      "onUpdate:modelValue": _cache[86] || (_cache[86] = ($event: any) => ((role.value.policies.clipLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.clipLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.clipLimit.value,
                      "onUpdate:modelValue": _cache[87] || (_cache[87] = ($event: any) => ((role.value.policies.clipLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.clipLimit.priority,
                      "onUpdate:modelValue": _cache[88] || (_cache[88] = ($event: any) => ((role.value.policies.clipLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.noteEachClipsMax, 'noteEachClipsLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.noteEachClipsMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.noteEachClipsLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.noteEachClipsLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.noteEachClipsLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.noteEachClipsLimit.useDefault,
                      "onUpdate:modelValue": _cache[89] || (_cache[89] = ($event: any) => ((role.value.policies.noteEachClipsLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.noteEachClipsLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.noteEachClipsLimit.value,
                      "onUpdate:modelValue": _cache[90] || (_cache[90] = ($event: any) => ((role.value.policies.noteEachClipsLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.noteEachClipsLimit.priority,
                      "onUpdate:modelValue": _cache[91] || (_cache[91] = ($event: any) => ((role.value.policies.noteEachClipsLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.userListMax, 'userListLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.userListMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.userListLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.userListLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.userListLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.userListLimit.useDefault,
                      "onUpdate:modelValue": _cache[92] || (_cache[92] = ($event: any) => ((role.value.policies.userListLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.userListLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.userListLimit.value,
                      "onUpdate:modelValue": _cache[93] || (_cache[93] = ($event: any) => ((role.value.policies.userListLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.userListLimit.priority,
                      "onUpdate:modelValue": _cache[94] || (_cache[94] = ($event: any) => ((role.value.policies.userListLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.userEachUserListsMax, 'userEachUserListsLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.userEachUserListsMax), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.userEachUserListsLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.userEachUserListsLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.userEachUserListsLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.userEachUserListsLimit.useDefault,
                      "onUpdate:modelValue": _cache[95] || (_cache[95] = ($event: any) => ((role.value.policies.userEachUserListsLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.userEachUserListsLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.userEachUserListsLimit.value,
                      "onUpdate:modelValue": _cache[96] || (_cache[96] = ($event: any) => ((role.value.policies.userEachUserListsLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.userEachUserListsLimit.priority,
                      "onUpdate:modelValue": _cache[97] || (_cache[97] = ($event: any) => ((role.value.policies.userEachUserListsLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canHideAds, 'canHideAds']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canHideAds), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canHideAds.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canHideAds.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canHideAds))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canHideAds.useDefault,
                      "onUpdate:modelValue": _cache[98] || (_cache[98] = ($event: any) => ((role.value.policies.canHideAds.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canHideAds.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canHideAds.value,
                      "onUpdate:modelValue": _cache[99] || (_cache[99] = ($event: any) => ((role.value.policies.canHideAds.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canHideAds.priority,
                      "onUpdate:modelValue": _cache[100] || (_cache[100] = ($event: any) => ((role.value.policies.canHideAds.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.avatarDecorationLimit, 'avatarDecorationLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.avatarDecorationLimit), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.avatarDecorationLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.avatarDecorationLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.avatarDecorationLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.avatarDecorationLimit.useDefault,
                      "onUpdate:modelValue": _cache[101] || (_cache[101] = ($event: any) => ((role.value.policies.avatarDecorationLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      type: "number",
                      min: 0,
                      max: 16,
                      "onUpdate:modelValue": [updateAvatarDecorationLimit, ($event: any) => ((role.value.policies.avatarDecorationLimit.value) = $event)],
                      modelValue: role.value.policies.avatarDecorationLimit.value
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.avatarDecorationLimit), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.avatarDecorationLimit.priority,
                      "onUpdate:modelValue": _cache[102] || (_cache[102] = ($event: any) => ((role.value.policies.avatarDecorationLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canImportAntennas, 'canImportAntennas']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canImportAntennas), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canImportAntennas.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canImportAntennas.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canImportAntennas))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportAntennas.useDefault,
                      "onUpdate:modelValue": _cache[103] || (_cache[103] = ($event: any) => ((role.value.policies.canImportAntennas.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canImportAntennas.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportAntennas.value,
                      "onUpdate:modelValue": _cache[104] || (_cache[104] = ($event: any) => ((role.value.policies.canImportAntennas.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canImportAntennas.priority,
                      "onUpdate:modelValue": _cache[105] || (_cache[105] = ($event: any) => ((role.value.policies.canImportAntennas.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canImportBlocking, 'canImportBlocking']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canImportBlocking), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canImportBlocking.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canImportBlocking.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canImportBlocking))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportBlocking.useDefault,
                      "onUpdate:modelValue": _cache[106] || (_cache[106] = ($event: any) => ((role.value.policies.canImportBlocking.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canImportBlocking.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportBlocking.value,
                      "onUpdate:modelValue": _cache[107] || (_cache[107] = ($event: any) => ((role.value.policies.canImportBlocking.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canImportBlocking.priority,
                      "onUpdate:modelValue": _cache[108] || (_cache[108] = ($event: any) => ((role.value.policies.canImportBlocking.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canImportFollowing, 'canImportFollowing']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canImportFollowing), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canImportFollowing.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canImportFollowing.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canImportFollowing))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportFollowing.useDefault,
                      "onUpdate:modelValue": _cache[109] || (_cache[109] = ($event: any) => ((role.value.policies.canImportFollowing.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canImportFollowing.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportFollowing.value,
                      "onUpdate:modelValue": _cache[110] || (_cache[110] = ($event: any) => ((role.value.policies.canImportFollowing.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canImportFollowing.priority,
                      "onUpdate:modelValue": _cache[111] || (_cache[111] = ($event: any) => ((role.value.policies.canImportFollowing.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canImportMuting, 'canImportMuting']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canImportMuting), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canImportMuting.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canImportMuting.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canImportMuting))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportMuting.useDefault,
                      "onUpdate:modelValue": _cache[112] || (_cache[112] = ($event: any) => ((role.value.policies.canImportMuting.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canImportMuting.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportMuting.value,
                      "onUpdate:modelValue": _cache[113] || (_cache[113] = ($event: any) => ((role.value.policies.canImportMuting.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canImportMuting.priority,
                      "onUpdate:modelValue": _cache[114] || (_cache[114] = ($event: any) => ((role.value.policies.canImportMuting.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.canImportUserLists, 'canImportUserLists']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.canImportUserLists), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.canImportUserLists.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.canImportUserLists.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.canImportUserLists))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportUserLists.useDefault,
                      "onUpdate:modelValue": _cache[115] || (_cache[115] = ($event: any) => ((role.value.policies.canImportUserLists.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.canImportUserLists.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.canImportUserLists.value,
                      "onUpdate:modelValue": _cache[116] || (_cache[116] = ($event: any) => ((role.value.policies.canImportUserLists.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.canImportUserLists.priority,
                      "onUpdate:modelValue": _cache[117] || (_cache[117] = ($event: any) => ((role.value.policies.canImportUserLists.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.noteDraftLimit, 'noteDraftLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.noteDraftLimit), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.noteDraftLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.noteDraftLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.noteDraftLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.noteDraftLimit.useDefault,
                      "onUpdate:modelValue": _cache[118] || (_cache[118] = ($event: any) => ((role.value.policies.noteDraftLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.noteDraftLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.noteDraftLimit.value,
                      "onUpdate:modelValue": _cache[119] || (_cache[119] = ($event: any) => ((role.value.policies.noteDraftLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.noteDraftLimit.priority,
                      "onUpdate:modelValue": _cache[120] || (_cache[120] = ($event: any) => ((role.value.policies.noteDraftLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.scheduledNoteLimit, 'scheduledNoteLimit']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.scheduledNoteLimit), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.scheduledNoteLimit.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.scheduledNoteLimit.value), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.scheduledNoteLimit))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.scheduledNoteLimit.useDefault,
                      "onUpdate:modelValue": _cache[121] || (_cache[121] = ($event: any) => ((role.value.policies.scheduledNoteLimit.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkInput, {
                      disabled: role.value.policies.scheduledNoteLimit.useDefault,
                      type: "number",
                      readonly: __props.readonly,
                      modelValue: role.value.policies.scheduledNoteLimit.value,
                      "onUpdate:modelValue": _cache[122] || (_cache[122] = ($event: any) => ((role.value.policies.scheduledNoteLimit.value) = $event))
                    }, null, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.scheduledNoteLimit.priority,
                      "onUpdate:modelValue": _cache[123] || (_cache[123] = ($event: any) => ((role.value.policies.scheduledNoteLimit.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (matchQuery([_unref(i18n).ts._role._options.watermarkAvailable, 'watermarkAvailable']))
              ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role._options.watermarkAvailable), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  (role.value.policies.watermarkAvailable.useDefault)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.useDefaultLabel)
                    }, _toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */))
                    : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(role.value.policies.watermarkAvailable.value ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.priorityIndicator)
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(getPriorityIcon(role.value.policies.watermarkAvailable))
                    }, null, 2 /* CLASS */)
                  ])
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      readonly: __props.readonly,
                      modelValue: role.value.policies.watermarkAvailable.useDefault,
                      "onUpdate:modelValue": _cache[124] || (_cache[124] = ($event: any) => ((role.value.policies.watermarkAvailable.useDefault) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.useBaseValue), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["readonly", "modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: role.value.policies.watermarkAvailable.useDefault,
                      readonly: __props.readonly,
                      modelValue: role.value.policies.watermarkAvailable.value,
                      "onUpdate:modelValue": _cache[125] || (_cache[125] = ($event: any) => ((role.value.policies.watermarkAvailable.value) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "readonly", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0,
                      max: 2,
                      step: 1,
                      easing: "",
                      textConverter: (v) => v === 0 ? _unref(i18n).ts._role._priority.low : v === 1 ? _unref(i18n).ts._role._priority.middle : v === 2 ? _unref(i18n).ts._role._priority.high : '',
                      modelValue: role.value.policies.watermarkAvailable.priority,
                      "onUpdate:modelValue": _cache[126] || (_cache[126] = ($event: any) => ((role.value.policies.watermarkAvailable.priority) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._role.priority), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
