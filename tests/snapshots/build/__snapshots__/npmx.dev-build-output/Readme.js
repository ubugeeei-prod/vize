import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Readme',
  props: {
    html: { type: String, required: true }
  },
  setup(__props: any) {

const { copy } = useClipboard()
// Combined click handler for:
// 1. Intercepting npmjs.com links to route internally
// 2. Copy button functionality for code blocks
function handleClick(event: MouseEvent) {
  const target = event.target as HTMLElement | undefined
  if (!target) return
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button) {
    return
  }
  // Handle copy button clicks
  const copyTarget = target.closest('[data-copy]')
  if (copyTarget) {
    const wrapper = copyTarget.closest('.readme-code-block')
    if (!wrapper) return
    const pre = wrapper.querySelector('pre')
    if (!pre?.textContent) return
    copy(pre.textContent)
    const icon = copyTarget.querySelector('span')
    if (!icon) return
    const originalIcon = 'i-lucide:copy'
    const successIcon = 'i-lucide:check'
    icon.classList.remove(originalIcon)
    icon.classList.add(successIcon)
    setTimeout(() => {
      icon.classList.remove(successIcon)
      icon.classList.add(originalIcon)
    }, 2000)
    return
  }
  // Handle npmjs.com link clicks - route internally
  const anchor = target.closest('a')
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href) return
  // Handle relative anchor links
  if (href.startsWith('#') || href.startsWith('/')) {
    event.preventDefault()
    navigateTo(href)
    return
  }
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("article", {
      class: "readme prose prose-invert max-w-[70ch] lg:max-w-none px-1",
      dir: "auto",
      innerHTML: __props.html,
      style: {
        '--i18n-note': '\'' + _ctx.$t('package.readme.callout.note') + '\'',
        '--i18n-tip': '\'' + _ctx.$t('package.readme.callout.tip') + '\'',
        '--i18n-important': '\'' + _ctx.$t('package.readme.callout.important') + '\'',
        '--i18n-warning': '\'' + _ctx.$t('package.readme.callout.warning') + '\'',
        '--i18n-caution': '\'' + _ctx.$t('package.readme.callout.caution') + '\'',
      },
      onClick: handleClick
    }, null, 8 /* PROPS */, ["innerHTML"]))
}
}

})
