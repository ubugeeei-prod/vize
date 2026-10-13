<script>
import { defineComponent } from 'vue'
import Widget from '~/components/common/widget.vue'
import IpfsImageViewer from '~/components/ipfs/ipfs-image-viewer.vue'
import { format } from '~/mixins/format'
import TokenLogo from './token-logo.vue'

/**
 * Displays a token, icon and its value.
 * Optionally can show additional detail string after the value in parentheses.
 */
export default defineComponent({
  name: 'token-value',
  mixins: [format],
  components: {
    IpfsImageViewer,
    TokenLogo,
    Widget
  },

  props: {
    /**
     * Token title, rendered without text transformation
     */
    label: String,
    /**
     * Token tooltip
     */
    tooltip: String,
    /**
     * Token value. Large numbers are abbreviated with full value in tooltip
     */
    value: { type: [Number], required: true },
    /**
     * Icon path, from src/assets/icons folder
     */
    icon: String,
    /**
     * Additional detail rendered following token value if present
     */
    detail: String,
    /**
     * Amount to multiply the token values by
     */
    multiplier: {
      type: Number,
      default: 1
    },
    /**
     * Flag to know if this token is a coefficient type to change the view
     */
    coefficient: {
      type: Boolean,
      default: false
    },
    /**
     * Flag to know if this token is a coefficient type to change the view
     */
    coefficientPercentage: {
      type: Number || String
    },
    /**
     * utility | cash | voice
     * Determines the icon that will be shown if no icon is provided
     */
    type: {
      type: String,
      default: 'utility'
    },
    /**
     * IPFS CID of the logo
     */
    daoLogo: {
      type: String,
      default: undefined
    }
  },

  methods: {
    imageUrl(icon) {
      return require('~/assets/icons/' + icon)
    }
  }
})
</script>

<template lang="pug">
.row.items-center.justify-start.q-px-sm
  .col
    .row.q-mb-sm
      .col
        .text-body2.text-bold {{label === 'Cash Token (HUSD)' ? $t('proposal-creation.cashToken') : label}}
    .row.items-center
      token-logo(:customIcon="icon" :daoLogo="daoLogo" :type="type")
      .col
        widget(:style="{padding: '12px 15px', 'border-radius': '15px'}")
          .text-left.inline-block
            span(v-if="!coefficient") {{value}}
            span.text-bold.q-mx-sm(:class="coefficientPercentage && coefficientPercentage >= 0 ? 'text-positive' : 'text-negative'" v-else-if="coefficient && (coefficientPercentage !== undefined || coefficientPercentage !== null)") x
              | {{coefficientPercentage}}
            q-tooltip(:content-style="{'font-size': '1em'}" anchor="top right" self="top right" v-if="tooltip") {{tooltip}}
            div {{ detail }}
          .text-caption.text-left.inline-block.q-ml-sm.text-italic(v-if="detail") {{'(' + detail + ')'}}
</template>
<style scoped lang="stylus">
.token-overlay
  color white
  font-family: 'Source Sans Pro', sans-serif
  font-size: 145%
  font-style: italic
  font-weight: bold
  background-color: var(--q-color-primary)
  width: 100%
  height: 100%
  display: flex
  align-items: center
  justify-content: center
  border-radius: 50%
</style>
