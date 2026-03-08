import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { watch, ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import type { DeepPartial } from '@/utility/merge.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { deepMerge } from '@/utility/merge.js'
import { useMkSelect } from '@/composables/use-mkselect.js'

type PartialAllowedAntenna = Omit<Misskey.entities.Antenna, 'id' | 'createdAt' | 'updatedAt'> & {
	id?: string;
	createdAt?: string;
	updatedAt?: string;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAntennaEditor',
  props: {
    antenna: { type: null, required: false }
  },
  emits: ["created", "updated", "deleted"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const initialAntenna = deepMerge<PartialAllowedAntenna>(props.antenna ?? {}, {
	name: '',
	src: 'all',
	userListId: null,
	users: [],
	keywords: [],
	excludeKeywords: [],
	excludeBots: false,
	withReplies: false,
	caseSensitive: false,
	localOnly: false,
	withFile: false,
	excludeNotesInSensitiveChannel: false,
	isActive: true,
	hasUnreadNote: false,
	notify: false,
});
const {
	model: src,
	def: antennaSourcesSelectDef,
} = useMkSelect({
	items: [
		{ value: 'all', label: i18n.ts._antennaSources.all },
		//{ value: 'home', label: i18n.ts._antennaSources.homeTimeline },
		{ value: 'users', label: i18n.ts._antennaSources.users },
		//{ value: 'list', label: i18n.ts._antennaSources.userList },
		{ value: 'users_blacklist', label: i18n.ts._antennaSources.userBlacklist },
	],
	initialValue: initialAntenna.src,
});
const {
	model: userListId,
	def: userListsSelectDef,
} = useMkSelect({
	items: computed(() => {
		if (userLists.value == null) return [];
		return userLists.value.map(list => ({
			value: list.id,
			label: list.name,
		}));
	}),
	initialValue: initialAntenna.userListId,
});
const name = ref<string>(initialAntenna.name);
const users = ref<string>(initialAntenna.users.join('\n'));
const keywords = ref<string>(initialAntenna.keywords.map(x => x.join(' ')).join('\n'));
const excludeKeywords = ref<string>(initialAntenna.excludeKeywords.map(x => x.join(' ')).join('\n'));
const caseSensitive = ref<boolean>(initialAntenna.caseSensitive);
const localOnly = ref<boolean>(initialAntenna.localOnly);
const excludeBots = ref<boolean>(initialAntenna.excludeBots);
const withReplies = ref<boolean>(initialAntenna.withReplies);
const withFile = ref<boolean>(initialAntenna.withFile);
const excludeNotesInSensitiveChannel = ref<boolean>(initialAntenna.excludeNotesInSensitiveChannel);
const userLists = ref<Misskey.entities.UserList[] | null>(null);
watch(() => src.value, async () => {
	if (src.value === 'list' && userLists.value === null) {
		userLists.value = await misskeyApi('users/lists/list');
	}
});
async function saveAntenna() {
	const antennaData = {
		name: name.value,
		src: src.value,
		userListId: userListId.value,
		excludeBots: excludeBots.value,
		withReplies: withReplies.value,
		withFile: withFile.value,
		excludeNotesInSensitiveChannel: excludeNotesInSensitiveChannel.value,
		caseSensitive: caseSensitive.value,
		localOnly: localOnly.value,
		users: users.value.trim().split('\n').map(x => x.trim()),
		keywords: keywords.value.trim().split('\n').map(x => x.trim().split(' ')),
		excludeKeywords: excludeKeywords.value.trim().split('\n').map(x => x.trim().split(' ')),
	};
	if (initialAntenna.id == null) {
		const res = await os.apiWithDialog('antennas/create', antennaData);
		emit('created', res);
	} else {
		const res = await os.apiWithDialog('antennas/update', { ...antennaData, antennaId: initialAntenna.id });
		emit('updated', res);
	}
}
async function deleteAntenna() {
	if (initialAntenna.id == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: initialAntenna.name }),
	});
	if (canceled) return;
	await misskeyApi('antennas/delete', {
		antennaId: initialAntenna.id,
	});
	os.success();
	emit('deleted');
}
function addUser() {
	os.selectUser({ includeSelf: true }).then(user => {
		users.value = users.value.trim();
		users.value += '\n@' + Misskey.acct.toString(user);
		users.value = users.value.trim();
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 700px;"
    }, [ _createElementVNode("div", null, [ _createElementVNode("div", { class: "_gaps_m" }, [ _createVNode(MkInput, {
            modelValue: name.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((name).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSelect, {
            items: _unref(antennaSourcesSelectDef),
            modelValue: _unref(src),
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((src).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaSource), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["items", "modelValue"]), (_unref(src) === 'list') ? (_openBlock(), _createBlock(MkSelect, {
              key: 0,
              items: _unref(userListsSelectDef),
              modelValue: _unref(userListId),
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((userListId).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.userList), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["items", "modelValue"])) : (_unref(src) === 'users' || _unref(src) === 'users_blacklist') ? (_openBlock(), _createBlock(MkTextarea, {
                key: 1,
                modelValue: users.value,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((users).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.users), 1 /* TEXT */)
                ]),
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaUsersDescription), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _createElementVNode("button", {
                    class: "_textButton",
                    onClick: addUser
                  }, _toDisplayString(_unref(i18n).ts.addUser), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkSwitch, {
            modelValue: excludeBots.value,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((excludeBots).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaExcludeBots), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: withReplies.value,
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((withReplies).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.withReplies), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkTextarea, {
            modelValue: keywords.value,
            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((keywords).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaKeywords), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaKeywordsDescription), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkTextarea, {
            modelValue: excludeKeywords.value,
            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((excludeKeywords).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaExcludeKeywords), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.antennaKeywordsDescription), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: localOnly.value,
            "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((localOnly).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.localOnly), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: caseSensitive.value,
            "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((caseSensitive).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.caseSensitive), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: withFile.value,
            "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((withFile).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.withFileAntenna), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: excludeNotesInSensitiveChannel.value,
            "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((excludeNotesInSensitiveChannel).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.excludeNotesInSensitiveChannel), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.actions)
        }, [ _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, {
              inline: "",
              primary: "",
              onClick: _cache[12] || (_cache[12] = ($event: any) => (saveAntenna()))
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), (_unref(initialAntenna).id != null) ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                inline: "",
                danger: "",
                onClick: _cache[13] || (_cache[13] = ($event: any) => (deleteAntenna()))
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })) : _createCommentVNode("v-if", true) ]) ]) ]) ]))
}
}

})
