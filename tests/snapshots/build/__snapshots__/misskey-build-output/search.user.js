import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import { markRaw, ref, shallowRef, toRef } from 'vue'
import type { Endpoints } from 'misskey-js'
import MkUserList from '@/components/MkUserList.vue'
import MkInput from '@/components/MkInput.vue'
import MkRadios from '@/components/MkRadios.vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import * as os from '@/os.js'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'search.user',
  props: {
    query: { type: String, required: false, default: '' },
    origin: { type: null, required: false, default: 'combined' }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const key = ref(0);
const paginator = shallowRef<Paginator<'users/search'> | null>(null);
const searchQuery = ref(toRef(props, 'query').value);
const searchOrigin = ref(toRef(props, 'origin').value);
async function search() {
	const query = searchQuery.value.toString().trim();
	// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
	if (query == null || query === '') return;
	//#region AP lookup
	if (query.startsWith('https://') && !query.includes(' ')) {
		const confirm = await os.confirm({
			type: 'info',
			text: i18n.ts.lookupConfirm,
		});
		if (!confirm.canceled) {
			const promise = misskeyApi('ap/show', {
				uri: query,
			});
			os.promiseDialog(promise, null, null, i18n.ts.fetchingAsApObject);
			const res = await promise;
			if (res.type === 'User') {
				router.push('/@:acct/:page?', {
					params: {
						acct: `${res.object.username}@${res.object.host}`,
					},
				});
			// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
			} else if (res.type === 'Note') {
				router.push('/notes/:noteId/:initialTab?', {
					params: {
						noteId: res.object.id,
					},
				});
			}
			return;
		}
	}
	//#endregion
	if (query.length > 1 && !query.includes(' ')) {
		if (query.startsWith('@')) {
			const confirm = await os.confirm({
				type: 'info',
				text: i18n.ts.lookupConfirm,
			});
			if (!confirm.canceled) {
				router.pushByPath(`/${query}`);
				return;
			}
		}
		if (query.startsWith('#')) {
			const confirm = await os.confirm({
				type: 'info',
				text: i18n.ts.openTagPageConfirm,
			});
			if (!confirm.canceled) {
				router.push('/user-tags/:tag', {
					params: {
						tag: query.substring(1),
					},
				});
				return;
			}
		}
	}
	paginator.value = markRaw(new Paginator('users/search', {
		limit: 10,
		offsetMode: true,
		params: {
			query: query,
			origin: instance.federation === 'none' ? 'local' : searchOrigin.value,
		},
	}));
	key.value++;
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", { class: "_gaps" }, [ _createVNode(MkInput, {
          large: true,
          autofocus: true,
          type: "search",
          onEnter: _withModifiers(search, ["prevent"]),
          modelValue: searchQuery.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((searchQuery).value = $event))
        }, {
          prefix: _withCtx(() => [
            _hoisted_1
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["large", "autofocus", "modelValue"]), (_unref(instance).federation !== 'none') ? (_openBlock(), _createBlock(MkRadios, {
            key: 0,
            options: [
  				{ value: 'combined', label: _unref(i18n).ts.all },
  				{ value: 'local', label: _unref(i18n).ts.local },
  				{ value: 'remote', label: _unref(i18n).ts.remote },
  			],
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => (search())),
            modelValue: searchOrigin.value
          }, null, 8 /* PROPS */, ["options", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkButton, {
          large: "",
          primary: "",
          gradate: "",
          rounded: "",
          onClick: search
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]), (paginator.value) ? (_openBlock(), _createBlock(MkFoldableSection, { key: 0 }, {
          header: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.searchResult), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createVNode(MkUserList, {
              key: `searchUsers:${key.value}`,
              paginator: paginator.value
            }, null, 8 /* PROPS */, ["paginator"])
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
