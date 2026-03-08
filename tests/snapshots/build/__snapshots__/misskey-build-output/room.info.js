import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("hr")
import { computed, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { ensureSignin } from '@/i.js'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'room.info',
  props: {
    room: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const $i = ensureSignin();
const isOwner = computed(() => {
	return props.room.ownerId === $i.id;
});
const name_ = ref(props.room.name);
const description_ = ref(props.room.description);
function save() {
	os.apiWithDialog('chat/rooms/update', {
		roomId: props.room.id,
		name: name_.value,
		description: description_.value,
	});
}
async function del() {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.deleteAreYouSure({ x: name_.value }),
	});
	if (canceled) return;
	await os.apiWithDialog('chat/rooms/delete', {
		roomId: props.room.id,
	});
	router.push('/chat');
}
const isMuted = ref(props.room.isMuted ?? false);
watch(isMuted, async () => {
	await os.apiWithDialog('chat/rooms/mute', {
		roomId: props.room.id,
		mute: isMuted.value,
	});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkInput, {
        disabled: !isOwner.value,
        modelValue: name_.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((name_).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["disabled", "modelValue"]), _createVNode(MkTextarea, {
        disabled: !isOwner.value,
        modelValue: description_.value,
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((description_).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["disabled", "modelValue"]), (isOwner.value) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          onClick: save
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _hoisted_1, (isOwner.value || (_unref($i).isAdmin || _unref($i).isModerator)) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          danger: "",
          onClick: del
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.deleteRoom), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), (!isOwner.value) ? (_openBlock(), _createBlock(MkSwitch, {
          key: 0,
          modelValue: isMuted.value,
          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((isMuted).value = $event))
        }, {
          label: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.muteThisRoom), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"])) : _createCommentVNode("v-if", true) ]))
}
}

})
