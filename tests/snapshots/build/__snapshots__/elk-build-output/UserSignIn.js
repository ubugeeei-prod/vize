import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = { "text-3xl": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary-light": "true", me1: "true" }, "https://")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:lightbulb-line": "true", "me-1": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
import Fuse from 'fuse.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'UserSignIn',
  setup(__props) {

const input = ref<HTMLInputElement | undefined>()
const knownServers = ref<string[]>([])
const autocompleteIndex = ref(0)
const autocompleteShow = ref(false)
const { busy, error, displayError, server, oauth } = useSignIn(input)
const fuse = shallowRef(new Fuse([] as string[]))
const filteredServers = computed(() => {
  if (!server.value)
    return []

  const results = fuse.value.search(server.value, { limit: 6 }).map(result => result.item)
  if (results[0] === server.value)
    return []

  return results
})
function isValidUrl(str: string) {
  try {
    // eslint-disable-next-line no-new
    new URL(str)
    return true
  }
  catch {
    return false
  }
}
async function handleInput() {
  const input = server.value.trim()
  if (input.startsWith('https://'))
    server.value = input.replace('https://', '')
  if (input.length)
    displayError.value = false
  if (
    isValidUrl(`https://${input}`)
    && input.match(/^[a-z0-9-]+(\.[a-z0-9-]+)+(:\d+)?$/i)
    // Do not hide the autocomplete if a result has an exact substring match on the input
    && !filteredServers.value.some(s => s.includes(input))
  ) {
    autocompleteShow.value = false
  }
  else {
    autocompleteShow.value = true
  }
}
function toSelector(server: string) {
  return server.replace(/[^\w-]/g, '-')
}
function move(delta: number) {
  if (filteredServers.value.length === 0) {
    autocompleteIndex.value = 0
    return
  }
  autocompleteIndex.value = ((autocompleteIndex.value + delta) + filteredServers.value.length) % filteredServers.value.length
  document.querySelector(`#${toSelector(filteredServers.value[autocompleteIndex.value])}`)?.scrollIntoView(false)
}
function onEnter(e: KeyboardEvent) {
  if (autocompleteShow.value === true && filteredServers.value[autocompleteIndex.value]) {
    server.value = filteredServers.value[autocompleteIndex.value]
    e.preventDefault()
    autocompleteShow.value = false
  }
}
function escapeAutocomplete(evt: KeyboardEvent) {
  if (!autocompleteShow.value)
    return
  autocompleteShow.value = false
  evt.stopPropagation()
}
function select(index: number) {
  server.value = filteredServers.value[index]
}
onMounted(async () => {
  input?.value?.focus()
  knownServers.value = await (globalThis.$fetch as any)('/api/list-servers')
  fuse.value = new Fuse(knownServers.value, { shouldSort: true })
})
onClickOutside(input, () => {
  autocompleteShow.value = false
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("form", {
      "text-center": "",
      "justify-center": "",
      "items-center": "",
      "max-w-150": "",
      py6: "",
      flex: "~ col gap-3",
      onSubmit: _cache[0] || (_cache[0] = _withModifiers((...args) => (oauth && oauth(...args)), ["prevent"]))
    }, [ _createElementVNode("div", {
        flex: "~ center",
        "items-end": "",
        mb2: "",
        "gap-x-2": ""
      }, [ _createElementVNode("img", {
          src: `/${''}logo.svg`,
          "w-12": "",
          "h-12": "",
          mxa: "",
          height: "48",
          width: "48",
          alt: _ctx.$t('app_logo'),
          class: "rtl-flip"
        }, null, 8 /* PROPS */, ["src", "alt"]), _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('action.sign_in')), 1 /* TEXT */) ]), _createElementVNode("div", null, _toDisplayString(_ctx.$t('user.server_address_label')), 1 /* TEXT */), _createElementVNode("div", {
        class: _normalizeClass(_unref(error) ? 'animate animate-shake-x animate-delay-100' : null)
      }, [ _createElementVNode("div", {
          dir: "ltr",
          flex: "",
          "bg-gray:10": "",
          px4: "",
          py2: "",
          mxa: "",
          rounded: "",
          border: "~ base",
          "items-center": "",
          "font-mono": "",
          "focus:outline-none": "",
          "focus:ring": "2 primary inset",
          relative: "",
          class: _normalizeClass(_unref(displayError) ? 'border-red-600 dark:border-red-400' : null)
        }, [ _hoisted_2, _withDirectives(_createElementVNode("input", {
            ref_key: "input", ref: input,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((server).value = $event)),
            autocapitalize: "off",
            inputmode: "url",
            "outline-none": "",
            "bg-transparent": "",
            "w-full": "",
            "max-w-50": "",
            spellcheck: "false",
            autocorrect: "off",
            autocomplete: "off",
            onInput: handleInput,
            onKeydown: [_withKeys(($event: any) => (move(1)), ["down"]), _withKeys(($event: any) => (move(-1)), ["up"]), _withKeys(onEnter, ["enter"]), _withKeys(_withModifiers(escapeAutocomplete, ["prevent"]), ["esc"])],
            onFocus: _cache[2] || (_cache[2] = ($event: any) => (autocompleteShow.value = true))
          }, null, 32 /* NEED_HYDRATION */), [ [_vModelText, _unref(server)] ]), (autocompleteShow.value && filteredServers.value.length) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              absolute: "",
              "left-6em": "",
              "right-0": "",
              top: "100%",
              "bg-base": "",
              rounded: "",
              border: "~ base",
              "z-10": "",
              shadow: "",
              "of-auto": "",
              "overflow-y-auto": "",
              class: "max-h-[8rem]"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredServers.value, (name, idx) => {
                return (_openBlock(), _createElementBlock("button", {
                  key: name,
                  id: toSelector(name),
                  value: name,
                  "px-2": "",
                  py1: "",
                  "font-mono": "",
                  "w-full": "",
                  "text-left": "",
                  class: _normalizeClass(autocompleteIndex.value === idx ? 'text-primary font-bold' : null),
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (select(idx)))
                }, _toDisplayString(name), 11 /* TEXT, CLASS, PROPS */, ["id", "value"]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), _createElementVNode("div", { "min-h-4": "" }, [ _createVNode(_Transition, {
            css: "",
            "enter-active-class": "animate animate-fade-in"
          }, {
            default: _withCtx(() => [
              (_unref(displayError))
                ? (_openBlock(), _createElementBlock("p", {
                  key: 0,
                  role: "alert",
                  "p-0": "",
                  "m-0": "",
                  "text-xs": "",
                  "text-red-600": "",
                  "dark:text-red-400": ""
                }, _toDisplayString(_ctx.$t('error.sign_in_error')), 1 /* TEXT */))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }) ]) ], 2 /* CLASS */), _createElementVNode("div", {
        "text-secondary": "",
        "text-sm": "",
        flex: ""
      }, [ _hoisted_3, _createElementVNode("span", null, [ _createVNode(_component_i18n_t, { keypath: "user.tip_no_account" }, {
            default: _withCtx(() => [
              _createVNode(_component_NuxtLink, {
                href: "https://joinmastodon.org/servers",
                target: "_blank",
                external: "",
                class: "text-primary",
                hover: "underline"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('user.tip_register_account')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }) ]) ]), _createElementVNode("button", {
        flex: "~ row",
        "gap-x-2": "",
        "items-center": "",
        "btn-solid": "",
        mt2: "",
        disabled: !_unref(server) || _unref(busy)
      }, [ (_unref(busy)) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            "aria-hidden": "true",
            block: "",
            animate: "",
            "animate-spin": "",
            "preserve-3d": "",
            class: "rtl-flip"
          }, [ _hoisted_4 ])) : (_openBlock(), _createElementBlock("span", {
            key: 1,
            "aria-hidden": "true",
            block: "",
            "i-ri:login-circle-line": "",
            class: "rtl-flip"
          })), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('action.sign_in')), 1 /* TEXT */) ], 8 /* PROPS */, ["disabled"]) ], 32 /* NEED_HYDRATION */))
}
}

})
