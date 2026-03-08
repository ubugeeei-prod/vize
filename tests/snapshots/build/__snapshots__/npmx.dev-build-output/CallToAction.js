import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = { class: "text-lg text-fg uppercase tracking-wider mb-6" }
const _hoisted_2 = { class: "font-medium text-fg" }
const _hoisted_3 = { class: "text-sm text-fg-muted leading-relaxed" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right rtl-flip w-3 h-3", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'CallToAction',
  setup(__props) {

const socialLinks = computed(() => [
  {
    id: 'github',
    href: 'https://repo.npmx.dev',
    icon: 'i-simple-icons:github',
    titleKey: $t('about.get_involved.contribute.title'),
    descriptionKey: $t('about.get_involved.contribute.description'),
    ctaKey: $t('about.get_involved.contribute.cta'),
  },
  {
    id: 'discord',
    href: 'https://chat.npmx.dev',
    icon: 'i-lucide:message-circle',
    titleKey: $t('about.get_involved.community.title'),
    descriptionKey: $t('about.get_involved.community.description'),
    ctaKey: $t('about.get_involved.community.cta'),
  },
  {
    id: 'bluesky',
    href: 'https://social.npmx.dev',
    icon: 'i-simple-icons:bluesky',
    titleKey: $t('about.get_involved.follow.title'),
    descriptionKey: $t('about.get_involved.follow.description'),
    ctaKey: $t('about.get_involved.follow.cta'),
  },
])
function handleCardClick(event: MouseEvent) {
  if ((event.target as HTMLElement).closest(':any-link')) return
  if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return
  const selection = window.getSelection()
  if (selection && selection.type === 'Range') return
  const card = event.currentTarget as HTMLElement
  const link = card.querySelector('a')
  link?.click()
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('about.get_involved.title')), 1 /* TEXT */), _createElementVNode("div", { class: "grid gap-4 sm:grid-cols-3 sm:items-stretch sm:grid-rows-[auto,1fr,auto]" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(socialLinks.value, (link) => {
          return (_openBlock(), _createElementBlock("div", {
            key: link.id,
            onClick: handleCardClick,
            class: "cursor-pointer group relative grid gap-3 p-4 rounded-lg bg-bg-subtle hover:bg-bg-elevated border border-border hover:border-border-hover transition-all duration-200 sm:grid-rows-subgrid sm:row-span-3 focus-within:ring-2 focus-within:ring-accent/50"
          }, [
            _createElementVNode("h3", { class: "flex gap-2" }, [
              _createElementVNode("span", {
                class: _normalizeClass(["shrink-0 mt-1 w-5 h-5 text-fg", link.icon]),
                "aria-hidden": "true"
              }, null, 2 /* CLASS */),
              _createElementVNode("span", _hoisted_2, _toDisplayString(link.titleKey), 1 /* TEXT */)
            ]),
            _createElementVNode("p", _hoisted_3, _toDisplayString(link.descriptionKey), 1 /* TEXT */),
            _createElementVNode("a", {
              href: link.href,
              target: "_blank",
              rel: "noopener noreferrer",
              class: "text-sm text-fg-muted group-hover:text-fg inline-flex items-center gap-1 mt-auto focus-visible:outline-none"
            }, [
              _createTextVNode(_toDisplayString(link.ctaKey) + "\n          ", 1 /* TEXT */),
              _hoisted_4
            ], 8 /* PROPS */, ["href"])
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
