import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-fill": "true" })
const _hoisted_2 = { "text-xl": "true", "font-700": "true", mb3: "true" }
const _hoisted_3 = { "font-700": "true", "my-2": "true", "text-lg": "true" }
const _hoisted_4 = { "mr-2": "true", "break-words": "true", "overflow-hidden": "true", "leading-4": "true", "h-full": "true", "inline-block": "true", "align-middle": "true" }
const _hoisted_5 = { class: "px2 md:px1.5 lg:px2 lg:px2 py0 lg:py-0.5", rounded: "true", "bg-code": "true", border: "px $c-border-code", "shadow-sm": "true", my1: "true", "font-mono": "true", "font-600": "true" }

interface ShortcutDef {
  keys: string[]
  isSequence: boolean
}
interface ShortcutItem {
  description: string
  shortcut: ShortcutDef
}
interface ShortcutItemGroup {
  name: string
  items: ShortcutItem[]
}

export default /*@__PURE__*/_defineComponent({
  __name: 'MagickeysKeyboardShortcuts',
  emits: ["close"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const { t } = useI18n()
/* TODOs:
 * - I18n
 */
const isMac = useIsMac()
const modifierKeyName = computed(() => isMac.value ? '⌘' : 'Ctrl')
const shortcutItemGroups = computed<ShortcutItemGroup[]>(() => [
  {
    name: t('magic_keys.groups.navigation.title'),
    items: [
      {
        description: t('magic_keys.groups.navigation.shortcut_help'),
        shortcut: { keys: ['?'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.navigation.next_status'),
        shortcut: { keys: ['j'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.navigation.previous_status'),
        shortcut: { keys: ['k'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_search'),
        shortcut: { keys: ['/'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_home'),
        shortcut: { keys: ['g', 'h'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_notifications'),
        shortcut: { keys: ['g', 'n'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_conversations'),
        shortcut: { keys: ['g', 'c'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_favourites'),
        shortcut: { keys: ['g', 'f'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_bookmarks'),
        shortcut: { keys: ['g', 'b'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_explore'),
        shortcut: { keys: ['g', 'e'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_local'),
        shortcut: { keys: ['g', 'l'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_federated'),
        shortcut: { keys: ['g', 't'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_lists'),
        shortcut: { keys: ['g', 'i'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_settings'),
        shortcut: { keys: ['g', 's'], isSequence: true },
      },
      {
        description: t('magic_keys.groups.navigation.go_to_profile'),
        shortcut: { keys: ['g', 'p'], isSequence: true },
      },
    ],
  },
  {
    name: t('magic_keys.groups.actions.title'),
    items: [
      {
        description: t('magic_keys.groups.actions.search'),
        shortcut: { keys: [modifierKeyName.value, 'k'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.command_mode'),
        shortcut: { keys: [modifierKeyName.value, '/'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.compose'),
        shortcut: { keys: ['c'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.show_new_items'),
        shortcut: { keys: ['.'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.favourite'),
        shortcut: { keys: ['f'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.boost'),
        shortcut: { keys: ['b'], isSequence: false },
      },
      {
        description: t('magic_keys.groups.actions.quote'),
        shortcut: { keys: ['q'], isSequence: false },
      },
    ],
  },
  {
    name: t('magic_keys.groups.media.title'),
    items: [],
  },
])

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "px-3": "",
      "sm:px-5": "",
      "py-2": "",
      "sm:py-4": "",
      "max-w-220": "",
      relative: "",
      "max-h-screen": ""
    }, [ _createElementVNode("button", {
        "btn-action-icon": "",
        absolute: "",
        "top-1": "",
        "sm:top-2": "",
        "right-1": "",
        "sm:right-2": "",
        m1: "",
        "aria-label": _ctx.$t('modals.aria_label_close'),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('close')))
      }, [ _hoisted_1 ], 8 /* PROPS */, ["aria-label"]), _createElementVNode("h2", _hoisted_2, _toDisplayString(_ctx.$t('magic_keys.dialog_header')), 1 /* TEXT */), _createElementVNode("div", {
        mb2: "",
        grid: "",
        "grid-cols-1": "",
        "md:grid-cols-3": "",
        "gap-y-": "",
        "md:gap-x-6": "",
        "lg:gap-x-8": ""
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(shortcutItemGroups.value, (group) => {
          return (_openBlock(), _createElementBlock("div", { key: group.name }, [
            _createElementVNode("h3", _hoisted_3, _toDisplayString(group.name), 1 /* TEXT */),
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(group.items, (item) => {
              return (_openBlock(), _createElementBlock("div", {
                key: item.description,
                flex: "",
                "my-1": "",
                "lg:my-2": "",
                "justify-between": "",
                "place-items-center": "",
                "max-w-full": "",
                "text-base": ""
              }, [
                _createElementVNode("div", _hoisted_4, _toDisplayString(item.description), 1 /* TEXT */),
                _createElementVNode("div", null, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(item.shortcut.keys, (key, idx) => {
                    return (_openBlock(), _createElementBlock(_Fragment, { key: idx }, [
                      (idx !== 0)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          mx1: "",
                          "text-sm": "",
                          op80: ""
                        }, _toDisplayString(item.shortcut.isSequence ? _ctx.$t('magic_keys.sequence_then') : '+'), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      _createElementVNode("code", _hoisted_5, _toDisplayString(key), 1 /* TEXT */)
                    ], 64 /* STABLE_FRAGMENT */))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]))
            }), 128 /* KEYED_FRAGMENT */))
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
