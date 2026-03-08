import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "absolute inset-0 bg-bg/80 backdrop-blur-md" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "pb-0.5" }, "npmx")
const _hoisted_3 = { "aria-hidden": "true", class: "scale-35 transform-origin-br font-mono tracking-wide text-accent absolute bottom-0.5 -inset-ie-1" }
const _hoisted_4 = { class: "text-xs px-1.5 py-0.5 rounded badge-green font-sans font-medium ms-2" }
import { LinkBase } from '#components'
import type { NavigationConfig, NavigationConfigWithGroups } from '~/types'
import { isEditableElement } from '~/utils/input'
import { NPMX_DOCS_SITE } from '#shared/utils/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'AppHeader',
  props: {
    showLogo: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const keyboardShortcuts = useKeyboardShortcuts()
const { isConnected, npmUser } = useConnector()
const desktopLinks = computed<NavigationConfig>(() => [
  {
    name: 'Compare',
    label: $t('nav.compare'),
    to: { name: 'compare' },
    keyshortcut: 'c',
    type: 'link',
    external: false,
    iconClass: 'i-lucide:git-compare',
  },
  {
    name: 'Settings',
    label: $t('nav.settings'),
    to: { name: 'settings' },
    keyshortcut: ',',
    type: 'link',
    external: false,
    iconClass: 'i-lucide:settings',
  },
])
const mobileLinks = computed<NavigationConfigWithGroups>(() => [
  {
    name: 'Desktop Links',
    type: 'group',
    items: [...desktopLinks.value],
  },
  {
    type: 'separator',
  },
  {
    name: 'About & Policies',
    type: 'group',
    items: [
      {
        name: 'About',
        label: $t('footer.about'),
        to: { name: 'about' },
        type: 'link',
        external: false,
        iconClass: 'i-lucide:info',
      },
      {
        name: 'Privacy Policy',
        label: $t('privacy_policy.title'),
        to: { name: 'privacy' },
        type: 'link',
        external: false,
        iconClass: 'i-lucide:shield-check',
      },
      {
        name: 'Accessibility',
        label: $t('a11y.title'),
        to: { name: 'accessibility' },
        type: 'link',
        external: false,
        iconClass: 'i-custom:a11y',
      },
    ],
  },
  {
    type: 'separator',
  },
  {
    name: 'External Links',
    type: 'group',
    label: $t('nav.links'),
    items: [
      {
        name: 'Docs',
        label: $t('footer.docs'),
        href: NPMX_DOCS_SITE,
        target: '_blank',
        type: 'link',
        external: true,
        iconClass: 'i-lucide:file-text',
      },
      {
        name: 'Source',
        label: $t('footer.source'),
        href: 'https://repo.npmx.dev',
        target: '_blank',
        type: 'link',
        external: true,
        iconClass: 'i-simple-icons:github',
      },
      {
        name: 'Social',
        label: $t('footer.social'),
        href: 'https://social.npmx.dev',
        target: '_blank',
        type: 'link',
        external: true,
        iconClass: 'i-simple-icons:bluesky',
      },
      {
        name: 'Chat',
        label: $t('footer.chat'),
        href: 'https://chat.npmx.dev',
        target: '_blank',
        type: 'link',
        external: true,
        iconClass: 'i-lucide:message-circle',
      },
    ],
  },
])
const showFullSearch = shallowRef(false)
const showMobileMenu = shallowRef(false)
const { env, prNumber } = useAppConfig().buildInfo
// On mobile, clicking logo+search button expands search
const route = useRoute()
const isMobile = useIsMobile()
const isSearchExpandedManually = shallowRef(false)
const searchBoxRef = useTemplateRef('searchBoxRef')
// On search page, always show search expanded on mobile
const isOnHomePage = computed(() => route.name === 'index')
const isOnSearchPage = computed(() => route.name === 'search')
const isSearchExpanded = computed(() => isOnSearchPage.value || isSearchExpandedManually.value)
function expandMobileSearch() {
  isSearchExpandedManually.value = true
  nextTick(() => {
    searchBoxRef.value?.focus()
  })
}
watch(
  isOnSearchPage,
  visible => {
    if (!visible) return
    searchBoxRef.value?.focus()
    nextTick(() => {
      searchBoxRef.value?.focus()
    })
  },
  { flush: 'sync' },
)
function handleSearchBlur() {
  showFullSearch.value = false
  // Collapse expanded search on mobile after blur (with delay for click handling)
  // But don't collapse if we're on the search page
  if (isMobile.value && !isOnSearchPage.value) {
    setTimeout(() => {
      isSearchExpandedManually.value = false
    }, 150)
  }
}
function handleSearchFocus() {
  showFullSearch.value = true
}
onKeyStroke(
  e => {
    if (!keyboardShortcuts.value || isEditableElement(e.target)) {
      return
    }
    for (const link of desktopLinks.value) {
      if (link.to && link.keyshortcut && isKeyWithoutModifiers(e, link.keyshortcut)) {
        e.preventDefault()
        navigateTo(link.to)
        break
      }
    }
  },
  { dedupe: true },
)

return (_ctx: any,_cache: any) => {
  const _component_AppLogo = _resolveComponent("AppLogo")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_HeaderSearchBox = _resolveComponent("HeaderSearchBox")
  const _component_HeaderPackagesDropdown = _resolveComponent("HeaderPackagesDropdown")
  const _component_HeaderOrgsDropdown = _resolveComponent("HeaderOrgsDropdown")
  const _component_HeaderAccountMenu = _resolveComponent("HeaderAccountMenu")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_HeaderMobileMenu = _resolveComponent("HeaderMobileMenu")

  return (_openBlock(), _createElementBlock("header", { class: "sticky top-0 z-50 border-b border-border" }, [ _hoisted_1, _createElementVNode("nav", {
        "aria-label": _ctx.$t('nav.main_navigation'),
        class: "relative container min-h-14 flex items-center gap-2 z-1 justify-end"
      }, [ (!isSearchExpanded.value && !isOnHomePage.value) ? (_openBlock(), _createBlock(_component_NuxtLink, {
            key: 0,
            to: "/",
            "aria-label": _ctx.$t('header.home'),
            class: "sm:hidden flex-shrink-0 font-mono text-lg font-medium text-fg hover:text-fg transition-colors duration-200 focus-ring"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_AppLogo, { class: "w-8 h-8 rounded-lg" })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["aria-label"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), (__props.showLogo) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "hidden sm:flex flex-shrink-0 items-center"
          }, [ _createVNode(_component_NuxtLink, {
              to: { name: 'index' },
              "aria-label": _ctx.$t('header.home'),
              dir: "ltr",
              class: "relative inline-flex items-center gap-1 header-logo font-mono text-lg font-medium text-fg hover:text-fg/90 transition-colors duration-200 rounded"
            }, {
              default: _withCtx(() => [
                _createVNode(_component_AppLogo, { class: "w-7 h-7 rounded-lg" }),
                _hoisted_2,
                _createElementVNode("span", _hoisted_3, _toDisplayString(_unref(env) === 'release' ? 'alpha' : _unref(env)), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to", "aria-label"]), (_unref(prNumber)) ? (_openBlock(), _createBlock(_component_NuxtLink, {
                key: 0,
                to: `https://github.com/npmx-dev/npmx.dev/pull/${_unref(prNumber)}`,
                "aria-label": `Open GitHub pull request ${_unref(prNumber)}`
              }, {
                default: _withCtx(() => [
                  _createElementVNode("span", _hoisted_4, "\n            PR #" + _toDisplayString(_unref(prNumber)), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "aria-label"])) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("span", {
            key: 1,
            class: "hidden sm:block w-1"
          })), _createTextVNode("\n      " + "\n      " + "\n\n      " + "\n      "), _createElementVNode("div", {
          class: _normalizeClass(["flex-1 flex items-center md:gap-6", {
            'hidden sm:flex': !isSearchExpanded.value,
            'justify-end': isOnHomePage.value,
            'justify-center': !isOnHomePage.value,
          }])
        }, [ _createVNode(_component_HeaderSearchBox, {
            ref_key: "searchBoxRef", ref: searchBoxRef,
            inputClass: isSearchExpanded.value ? 'w-full' : '',
            class: _normalizeClass({ 'max-w-md': !isSearchExpanded.value }),
            onFocus: handleSearchFocus,
            onBlur: handleSearchBlur
          }, null, 10 /* CLASS, PROPS */, ["inputClass"]), (!isSearchExpanded.value && _unref(isConnected) && _unref(npmUser)) ? (_openBlock(), _createElementBlock("ul", {
              key: 0,
              class: _normalizeClass(["hidden sm:flex items-center gap-4 sm:gap-6 list-none m-0 p-0", { hidden: showFullSearch.value }])
            }, [ (_unref(isConnected) && _unref(npmUser)) ? (_openBlock(), _createElementBlock("li", {
                  key: 0,
                  class: "flex items-center"
                }, [ _createVNode(_component_HeaderPackagesDropdown, { username: _unref(npmUser) }, null, 8 /* PROPS */, ["username"]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n          "), _createTextVNode("\n          "), (_unref(isConnected) && _unref(npmUser)) ? (_openBlock(), _createElementBlock("li", {
                  key: 0,
                  class: "flex items-center"
                }, [ _createVNode(_component_HeaderOrgsDropdown, { username: _unref(npmUser) }, null, 8 /* PROPS */, ["username"]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("div", { class: "hidden sm:flex flex-shrink-0" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(desktopLinks.value, (link) => {
            return (_openBlock(), _createBlock(LinkBase, {
              key: link.name,
              class: "border-none",
              variant: "button-secondary",
              to: link.to,
              "aria-keyshortcuts": link.keyshortcut
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(link.label), 1 /* TEXT */)
              ]),
              _: 2 /* DYNAMIC */
            }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to", "aria-keyshortcuts"]))
          }), 128 /* KEYED_FRAGMENT */)), _createVNode(_component_HeaderAccountMenu) ]), _createTextVNode("\n\n      " + "\n      "), (!isSearchExpanded.value && !isOnHomePage.value) ? (_openBlock(), _createBlock(_component_ButtonBase, {
            key: 0,
            type: "button",
            class: "sm:hidden ms-auto",
            "aria-label": _ctx.$t('nav.tap_to_search'),
            "aria-expanded": showMobileMenu.value,
            onClick: expandMobileSearch,
            classicon: "i-lucide:search"
          }, null, 8 /* PROPS */, ["aria-label", "aria-expanded"])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), _createVNode(_component_ButtonBase, {
          type: "button",
          class: "sm:hidden",
          "aria-label": _ctx.$t('nav.open_menu'),
          "aria-expanded": showMobileMenu.value,
          onClick: _cache[0] || (_cache[0] = ($event: any) => (showMobileMenu.value = !showMobileMenu.value)),
          classicon: "i-lucide:menu"
        }, null, 8 /* PROPS */, ["aria-label", "aria-expanded"]) ], 8 /* PROPS */, ["aria-label"]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_component_HeaderMobileMenu, {
        links: mobileLinks.value,
        open: showMobileMenu.value,
        "onUpdate:open": _cache[1] || (_cache[1] = ($event: any) => ((showMobileMenu).value = $event))
      }, null, 8 /* PROPS */, ["links", "open"]) ]))
}
}

})
