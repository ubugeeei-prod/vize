import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane-arrival" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane-arrival" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane-departure" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plane-departure" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import FormInfo from '@/components/MkInfo.vue'
import MkInput from '@/components/MkInput.vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkUserInfo from '@/components/MkUserInfo.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i.js'
import { unisonReload } from '@/utility/unison-reload.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'migration',
  setup(__props) {

const $i = ensureSignin();
const moveToAccount = ref('');
const movedTo = ref<Misskey.entities.UserDetailed>();
const accountAliases = ref(['']);
async function init() {
	if ($i.movedTo) {
		movedTo.value = await misskeyApi('users/show', { userId: $i.movedTo });
	} else {
		moveToAccount.value = '';
	}
	if ($i.alsoKnownAs && $i.alsoKnownAs.length > 0) {
		const alsoKnownAs = await misskeyApi('users/show', { userIds: $i.alsoKnownAs });
		accountAliases.value = (alsoKnownAs && alsoKnownAs.length > 0) ? alsoKnownAs.map(user => `@${Misskey.acct.toString(user)}`) : [''];
	} else {
		accountAliases.value = [''];
	}
}
async function move(): Promise<void> {
	const account = moveToAccount.value;
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.tsx._accountMigration.migrationConfirm({ account }),
	});
	if (confirm.canceled) return;
	await os.apiWithDialog('i/move', {
		moveToAccount: account,
	});
	unisonReload();
}
function add(): void {
	accountAliases.value.push('');
}
async function save(): Promise<void> {
	const alsoKnownAs = accountAliases.value.map(alias => alias.trim()).filter(alias => alias !== '');
	const i = await os.apiWithDialog('i/update', {
		alsoKnownAs,
	});
	$i.alsoKnownAs = i.alsoKnownAs;
	init();
}
init();

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkFolder, { defaultOpen: true }, {
        icon: _withCtx(() => [
          _hoisted_1
        ]),
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveFrom), 1 /* TEXT */)
        ]),
        caption: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveFromSub), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(FormInfo, null, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveFromDescription), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createElementVNode("div", null, [
              _createVNode(MkButton, {
                disabled: accountAliases.value.length >= 10,
                inline: "",
                style: "margin-right: 8px;",
                onClick: add
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"]),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createElementVNode("div", { class: "_gaps" }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(accountAliases.value, (_, i) => {
                return (_openBlock(), _createBlock(MkInput, { modelValue: accountAliases.value[i], "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((accountAliases.value[i]) = $event)) }, {
                  prefix: _withCtx(() => [
                    _hoisted_4
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).tsx._accountMigration.moveFromLabel({ n: i + 1 })), 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["modelValue"]))
              }), 256 /* UNKEYED_FRAGMENT */))
            ])
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]), _createVNode(MkFolder, { defaultOpen: !!_unref($i).movedTo }, {
        icon: _withCtx(() => [
          _hoisted_5
        ]),
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveTo), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(FormInfo, null, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveAccountDescription), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            (_unref($i) && !_unref($i).movedTo)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                _createVNode(FormInfo, null, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveAccountHowTo), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormInfo, { warn: "" }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveCannotBeUndone), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkInput, {
                  modelValue: moveToAccount.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((moveToAccount).value = $event))
                }, {
                  prefix: _withCtx(() => [
                    _hoisted_6
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.moveToLabel), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkButton, {
                  inline: "",
                  danger: "",
                  disabled: !moveToAccount.value,
                  onClick: move
                }, {
                  default: _withCtx(() => [
                    _hoisted_7,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.startMigration), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["disabled"])
              ], 64 /* STABLE_FRAGMENT */))
              : (_unref($i))
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  _createVNode(FormInfo, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.postMigrationNote), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(FormInfo, { warn: "" }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._accountMigration.movedAndCannotBeUndone), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._accountMigration.movedTo), 1 /* TEXT */),
                  (movedTo.value)
                    ? (_openBlock(), _createBlock(MkUserInfo, {
                      key: 0,
                      user: movedTo.value,
                      class: "_panel _shadow"
                    }, null, 8 /* PROPS */, ["user"]))
                    : _createCommentVNode("v-if", true)
                ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]) ]))
}
}

})
