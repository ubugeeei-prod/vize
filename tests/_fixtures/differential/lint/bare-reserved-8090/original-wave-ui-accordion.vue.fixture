<template lang="pug">
div
  .w-divider.my12
  title-link.title1(h2) API

  component-api.mt0(:items="props" :descriptions="propsDescs" title="Props")

  title-link.title1.mt12(h2 slug="w-accordion-item-api") &lt;w-accordion-item&gt; API
  component-api.mt0(:items="accordionItemProps" :descriptions="accordionItemPropsDescs" title="Props")

  component-api(:items="slots" title="w-accordion Slots")
  component-api.mt12(:items="slotsAccordionItem" title="w-accordion-item Slots")

  component-api(:items="events" title="Events")

  component-api(:items="methods" title="w-accordion-item Methods")
</template>

<script>
import WAccordion from '@/wave-ui/components/w-accordion/index.vue'
import WAccordionItem from '@/wave-ui/components/w-accordion/item.vue'

const accordionItemPropsDescs = {
  title: 'The title of the accordion item.',
  content: 'The content of the accordion item.',
  expanded: 'Whether this item starts expanded.',
  disabled: 'Disables interaction with this item.',
  noRipple: 'When <code>true</code>, disables the pointer ripple for this item title only. When omitted, it follows the parent accordion then <code>$waveui.config.ripple</code>.'
}

const propsDescs = {
  modelValue: '<strong class="error"><code>value</code> in Vue 2.</strong><br>Provide an array of booleans to dictate the state (expanded and collapsed) of all the accordion items. This value gets updated by the accordion when using a v-model.',
  color: 'Applies a text color on each accordion item. Accepts all the color names of the color palette, status colors, or custom colors (learn more about the colors in the <a href="colors">colors</a> knowledge base page).<br>Providing a color hex, rgb(a) or hsl(a) will not work.',
  bgColor: 'Applies a background color on each accordion item. Accepts all the color names of the color palette, status colors, or custom colors (learn more about the colors in the <a href="colors">colors</a> knowledge base page).<br>Providing a color hex, rgb(a) or hsl(a) will not work.',
  items: 'Required when not using the <strong class="code">w-accordion-item</strong> component, but redundant otherwise.<br>Expecting an array of objects. Each object being an accordion item, it should include a <code>title</code> and <code>content</code> attributes.<br><div class="w-tag white grey-dark4--bg size--xs w-tag--round bd2">DEPRECATED</div>: you can also provide an integer number (call it <em class="code">x</em>), to loop through and create <em class="code">x</em> items in the accordion.<br>You can (not deprecated) use the individual slots <code>item-title.x</code> &amp; <code>item-content.x</code> to define each item title and content.',
  itemTitleKey: 'Specifies the name of the property in each item object where to find the item\'s title string.',
  itemContentKey: 'Specifies the name of the property in each item object where to find the item\'s content string.',
  itemColorKey: 'Specifies the name of the property in each item object where to find the item\'s desired color string. This allows a different color for each accordion item.',
  itemClass: 'Applies a custom CSS class (or space separated classes, array of classes or object of classes) on every item container (which includes both the title and the content of the item).',
  titleClass: 'Applies a custom CSS class (or space separated classes, array of classes or object of classes) on every title container.',
  contentClass: 'Applies a custom CSS class (or space separated classes, array of classes or object of classes) on every content container.',
  expandedClass: 'Applies a custom CSS class (or space separated classes, array of classes or object of classes) on every expanded item container.',
  expandIcon: 'Specifies the name of the icon to display in the toggle button. If set to false or empty string, the button will be removed.',
  expandIconRight: 'When set to true, the expand button will be placed at the right end of the item\'s title.',
  expandIconRotate90: 'With this option, the expand icon will initially point to the right, then point down on expand with a clockwise rotation of 90 degrees.',
  expandIconProps: 'An object of props to pass down to the <strong class="code">w-icon</strong> component for more control on the expand icon.<br>Example of use: <code>&lt;w-accordion :items="items" :expand-icon-props="{ flipY: true }"&gt;&lt;/w-accordion&gt;</code>',
  expandSingle: 'Specifies the accordion behavior, whether only one item can be expanded at a time or multiple. When set to true, expanding another item than the one already expanded is still possible, but it will collapse the other expanded item.',
  collapseIcon: 'Provide a different icon than the expand icon, to show when the accordion item is expanded. By default and when empty, there is no collapse icon: the expand icon rotates to show a closing ability.',
  shadow: 'Applies a shadow to the whole accordion container.',
  duration: 'Specify the duration in millisecond of the expand/collapse animation.',
  keepInDom: '<span class="grey text-italic">Will deactivate the <code>keep-alive</code> prop.</span> When set to <code>true</code>, every panel content stays in the DOM when collapsed (hidden with <code>v-show</code>), unless the item is removed from the accordion.<br>Note that the <code>mounted</code> hook will be run once per panel, as soon as it is appended to the DOM.<br>The default behavior is to keep panel contents alive when <code>keep-alive</code> is enabled, but unmount collapsed panels from the DOM.',
  keepAlive: 'When set to <code>true</code> and by default, each panel content is kept alive. Which means that the state of the components inside a panel will be maintained when collapsing and expanding it, and the <code>mounted</code> hook will only be run the first time it is opened.<br>When explicitly set to <code>false</code>, the panel state will be reset upon each reopening, and the <code>mounted</code> hook will be run again.<br>The default behavior is to keep panel contents alive, but unmount collapsed panels from the DOM (unless <code>keep-in-dom</code> is used).<br>You can read more about the keep-alive behavior in the <a href="https://vuejs.org/guide/built-ins/keep-alive.html" target="_blank">Vue official documentation for keep-alive <i class="w-icon mdi mdi-open-in-new"></i></a>.',
  noRipple: 'When <code>true</code>, disables the pointer ripple on item titles for all items. By default, ripple follows <code>$waveui.config.ripple</code>. Override per item with <code>no-ripple</code> on <code>w-accordion-item</code>.'
}

const slots = {
  'item-title': {
    description: 'Provide a custom title for every item. Applies to all the items, but can be overridden by the <code>item-title.x</code> slot.',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  },
  'item-title.x': {
    description: '<em class="code">x</em> is an integer starting at <span class="code">1</span>.<br>Provide a custom title for a single item: the item at the index <em class="code">x</em>.',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  },
  'item-content': {
    description: 'Provide a custom content for every item. Applies to all the items, but can be overridden by the <code>item-content.x</code> slot.',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  },
  'item-content.x': {
    description: '<em class="code">x</em> is an integer starting at <span class="code">1</span>.<br>Provide a custom content for a single item: the item at the index <em class="code">x</em>.',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  }
}

const slotsAccordionItem = {
  title: {
    description: 'Provide a custom title for the item(s) (you could also use the title prop instead).',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  },
  content: {
    description: 'Provide a custom content for the item(s) (you could also use the content prop instead).',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1 to be consistent with the <code>item.x</code> slot.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  },
  default: {
    description: 'on the w-accordion-item component only. Provide a custom content for the item(s) (you could also use the content prop or the content slot instead).',
    params: {
      item: 'The current item object.',
      index: 'The item index in the array of items. Starts at 1.',
      expanded: 'A boolean representing the expanded state of the accordion item.'
    }
  }
}

const events = {
  input: {
    description: 'Emitted each time the state of the accordion changes (when an item is expanded or collapsed).<br>Updates the v-model value in Vue 2.x only.',
    params: {
      '[Array]': 'An array of booleans representing the expanded state of each accordion item.'
    }
  },
  'update:modelValue': {
    description: 'Emitted each time the state of the accordion changes (when an item is expanded or collapsed).<br>Updates the v-model value in Vue 3 only.',
    params: {
      '[Array]': 'An array of booleans representing the expanded state of each accordion item.'
    }
  },
  focus: {
    description: 'Emitted on each accordion item title focus.',
    params: {
      '[Object]': 'The focused accordion item object.'
    }
  }
}

const methods = {
  focus: {
    description: 'Focuses the accordion item title. Also runs when <code>v-focus</code> is used. No-op when the item is disabled.'
  }
}

export default {
  data: () => ({
    propsDescs,
    accordionItemPropsDescs,
    slots,
    slotsAccordionItem,
    methods
  }),

  computed: {
    // Reads all the props and events directly from the component, so that as soon as a new prop or event
    // is added it will appear even if no description is yet provided.
    props () {
      return WAccordion.props
    },

    accordionItemProps () {
      return { noRipple: { type: Boolean }, ...WAccordionItem.props }
    },
    events () {
      return WAccordion.emits.reduce((obj, label) => {
        obj[label] = events[label] || {}
        return obj
      }, {})
    }
  }
}
</script>
