import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { id: "interface-bn", "font-medium": "true" }
const _hoisted_2 = { id: "interface-bn-desc", "pb-2": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "block i-ri:delete-bin-line" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "block i-ri:repeat-line" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", "i-ri:save-2-fill": "true" })
import type { NavButtonName } from '~/composables/settings'
import { STORAGE_KEY_BOTTOM_NAV_BUTTONS } from '~/constants'

interface NavButton {
  name: NavButtonName
  label: string
  icon: string
}

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsBottomNav',
  setup(__props) {

const availableNavButtons: NavButton[] = [
  { name: 'home', label: 'nav.home', icon: 'i-ri:home-5-line' },
  { name: 'search', label: 'nav.search', icon: 'i-ri:search-line' },
  { name: 'notification', label: 'nav.notifications', icon: 'i-ri:notification-4-line' },
  { name: 'mention', label: 'nav.conversations', icon: 'i-ri:at-line' },
  { name: 'favorite', label: 'nav.favourites', icon: 'i-ri:heart-line' },
  { name: 'bookmark', label: 'nav.bookmarks', icon: 'i-ri:bookmark-line' },
  { name: 'compose', label: 'nav.compose', icon: 'i-ri:quill-pen-line' },
  { name: 'scheduledPosts', label: 'nav.scheduled_posts', icon: 'i-ri:calendar-schedule-line' },
  { name: 'explore', label: 'nav.explore', icon: 'i-ri:compass-3-line' },
  { name: 'local', label: 'nav.local', icon: 'i-ri:group-2-line' },
  { name: 'federated', label: 'nav.federated', icon: 'i-ri:earth-line' },
  { name: 'list', label: 'nav.lists', icon: 'i-ri:list-check' },
  { name: 'hashtag', label: 'nav.hashtags', icon: 'i-ri:hashtag' },
  { name: 'moreMenu', label: 'nav.more_menu', icon: 'i-ri:more-fill' },
] as const
const defaultSelectedNavButtonNames = computed<NavButtonName[]>(() =>
  currentUser.value
    ? ['home', 'search', 'notification', 'mention', 'moreMenu']
    : ['explore', 'local', 'federated', 'moreMenu'],
)
const navButtonNamesSetting = useLocalStorage<NavButtonName[]>(STORAGE_KEY_BOTTOM_NAV_BUTTONS, defaultSelectedNavButtonNames.value)
const selectedNavButtonNames = ref<NavButtonName[]>(navButtonNamesSetting.value)
const selectedNavButtons = computed<NavButton[]>(() =>
  selectedNavButtonNames.value.map(name =>
    availableNavButtons.find(navButton => navButton.name === name)!,
  ),
)
const canSave = computed(() =>
  selectedNavButtonNames.value.length > 0
  && selectedNavButtonNames.value.includes('moreMenu')
  && JSON.stringify(selectedNavButtonNames.value) !== JSON.stringify(navButtonNamesSetting.value),
)
function isAdded(name: NavButtonName) {
  return selectedNavButtonNames.value.includes(name)
}
function append(navButtonName: NavButtonName) {
  const maxButtonNumber = 5
  if (selectedNavButtonNames.value.length < maxButtonNumber)
    selectedNavButtonNames.value = [...selectedNavButtonNames.value, navButtonName]
}
function remove(navButtonName: NavButtonName) {
  selectedNavButtonNames.value = selectedNavButtonNames.value.filter(name => name !== navButtonName)
}
function clear() {
  selectedNavButtonNames.value = []
}
function reset() {
  selectedNavButtonNames.value = defaultSelectedNavButtonNames.value
}
function save() {
  navButtonNamesSetting.value = selectedNavButtonNames.value
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", { "space-y-2": "" }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('settings.interface.bottom_nav')), 1 /* TEXT */), _createElementVNode("form", {
        "aria-labelledby": "interface-bn",
        "aria-describedby": "interface-bn-desc",
        onSubmit: _withModifiers(save, ["prevent"])
      }, [ _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('settings.interface.bottom_nav_instructions')), 1 /* TEXT */), _createTextVNode("\n      " + "\n      "), _createElementVNode("div", {
          "aria-hidden": "true",
          flex: "~ gap4 wrap",
          "items-center": "",
          "select-settings": "",
          "h-14": ""
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(selectedNavButtons.value, (availableNavButton) => {
            return (_openBlock(), _createElementBlock("nav", {
              key: availableNavButton.name,
              flex: "~ 1",
              "items-center": "",
              "justify-center": "",
              "text-xl": "",
              "scrollbar-hide": "",
              "overscroll-none": ""
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(availableNavButton.icon)
              }, null, 2 /* CLASS */)
            ]))
          }), 128 /* KEYED_FRAGMENT */)) ]), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("div", {
          flex: "~ gap4 wrap",
          py4: ""
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(availableNavButtons), ({ name, label, icon }) => {
            return (_openBlock(), _createElementBlock("button", {
              key: name,
              "btn-text": "",
              flex: "~ gap-2",
              "items-center": "",
              p2: "",
              border: "~ base rounded",
              "bg-base": "",
              "ws-nowrap": "",
              class: _normalizeClass(isAdded(name) ? 'text-secondary hover:text-second bg-auto' : ''),
              type: "button",
              role: "switch",
              "aria-checked": isAdded(name),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (isAdded(_ctx.name) ? remove(_ctx.name) : append(_ctx.name)))
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(icon)
              }),
              _createTextVNode("\n          " + _toDisplayString(label ? _ctx.$t(label) : 'More menu'), 1 /* TEXT */)
            ], 8 /* PROPS */, ["aria-checked"]))
          }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("div", {
          flex: "~ col",
          "gap-y-4": "",
          "gap-x-2": "",
          "py-1": "",
          sm: "~ justify-end flex-row"
        }, [ _createElementVNode("button", {
            "btn-outline": "",
            "font-bold": "",
            py2: "",
            "full-w": "",
            "sm-wa": "",
            flex: "~ gap2 center",
            type: "button",
            disabled: selectedNavButtonNames.value.length === 0,
            class: _normalizeClass(selectedNavButtonNames.value.length === 0 ? 'border-none' : undefined),
            onClick: clear
          }, [ _hoisted_3, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('action.clear')), 1 /* TEXT */) ], 10 /* CLASS, PROPS */, ["disabled"]), _createElementVNode("button", {
            "btn-outline": "",
            "font-bold": "",
            py2: "",
            "full-w": "",
            "sm-wa": "",
            flex: "~ gap2 center",
            type: "reset",
            onClick: reset
          }, [ _hoisted_4, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('action.reset')), 1 /* TEXT */) ]), _createElementVNode("button", {
            "btn-solid": "",
            "font-bold": "",
            py2: "",
            "full-w": "",
            "sm-wa": "",
            flex: "~ gap2 center",
            disabled: !canSave.value
          }, [ _hoisted_5, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('action.save')), 1 /* TEXT */) ], 8 /* PROPS */, ["disabled"]) ]) ], 32 /* NEED_HYDRATION */) ]))
}
}

})
