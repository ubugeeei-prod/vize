import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList } from "vue"

import type { Component } from 'vue'
import type { NavButtonName } from '../../composables/settings'
import { NavButtonBookmark, NavButtonCompose, NavButtonExplore, NavButtonFavorite, NavButtonFederated, NavButtonHashtag, NavButtonHome, NavButtonList, NavButtonLocal, NavButtonMention, NavButtonMoreMenu, NavButtonNotification, NavButtonScheduledPosts, NavButtonSearch } from '#components'
import { STORAGE_KEY_BOTTOM_NAV_BUTTONS } from '~/constants'

interface NavButton {
  name: string
  component: Component
}

export default /*@__PURE__*/_defineComponent({
  __name: 'NavBottom',
  setup(__props) {

const navButtons: NavButton[] = [
  { name: 'home', component: NavButtonHome },
  { name: 'search', component: NavButtonSearch },
  { name: 'notification', component: NavButtonNotification },
  { name: 'mention', component: NavButtonMention },
  { name: 'favorite', component: NavButtonFavorite },
  { name: 'bookmark', component: NavButtonBookmark },
  { name: 'compose', component: NavButtonCompose },
  { name: 'scheduledPosts', component: NavButtonScheduledPosts },
  { name: 'explore', component: NavButtonExplore },
  { name: 'local', component: NavButtonLocal },
  { name: 'federated', component: NavButtonFederated },
  { name: 'list', component: NavButtonList },
  { name: 'hashtag', component: NavButtonHashtag },
  { name: 'moreMenu', component: NavButtonMoreMenu },
]
const defaultSelectedNavButtonNames: NavButtonName[] = currentUser.value
  ? ['home', 'search', 'notification', 'mention', 'moreMenu']
  : ['explore', 'local', 'federated', 'moreMenu']
const selectedNavButtonNames = useLocalStorage<NavButtonName[]>(STORAGE_KEY_BOTTOM_NAV_BUTTONS, defaultSelectedNavButtonNames)
const selectedNavButtons = computed(() => selectedNavButtonNames.value.map(name => navButtons.find(navButton => navButton.name === name)))
// only one icon can be lit up at the same time
const moreMenuVisible = ref(false)

return (_ctx: any,_cache: any) => {
  const _component_Component = _resolveComponent("Component")

  return (_openBlock(), _createElementBlock("nav", {
      "h-14": "",
      border: "t base",
      flex: "",
      "flex-row": "",
      "text-xl": "",
      "of-y-scroll": "",
      "scrollbar-hide": "",
      "overscroll-none": "",
      class: "after-content-empty after:(h-[calc(100%+0.5px)] w-0.1px pointer-events-none)"
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(selectedNavButtons.value, (navButton) => {
        return (_openBlock(), _createBlock(_resolveDynamicComponent(navButton.component), {
          key: navButton.name,
          is: navButton.component,
          "active-class": moreMenuVisible.value ? '' : 'text-primary'
        }, null, 8 /* PROPS */, ["is", "active-class"]))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
