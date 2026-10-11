<script setup lang="ts">
const { nth = 2 } = defineProps<{
  id?: string;

  /** @default 2 */
  nth?: 1 | 2 | 3;
}>();

defineSlots<{
  default: () => unknown;
}>();
</script>

<template>
  <a v-if="id" :href="`#${id}`" class="anchor">
    <component :is="`h${nth}`" :id="id" v-bind="$attrs">
      <slot />
    </component>
  </a>

  <component :is="`h${nth}`" v-else :id="id" v-bind="$attrs">
    <slot />
  </component>

  <hr />
</template>

<style scoped>
@import "~/assets/styles/custom-media-query.css";

h1,
h2,
h3 {
  color: var(--color-base);

  margin: 0;
  display: flex;
  align-items: center;
  column-gap: 0.625rem;

  &::before {
    content: "";
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
    background-color: var(--color-base);
  }
}

hr {
  border-top: 1px solid var(--color-divider);
  border-bottom: none;
  border-left: none;
  border-right: none;
}

h2 {
  font-size: 1.25rem;
  line-height: 1.7;

  @media (--mobile) {
    font-size: 1.125rem;
    line-height: 1.725;
  }
}

.anchor {
  text-decoration: none;
  @media (any-hover: hover) {
    &:hover {
      color: var(--color-accent-hover);

      h1,
      h2,
      h3,
      h4 {
        color: var(--color-accent-hover);
        &::before {
          background-color: var(--color-accent-hover);
        }
      }
    }
  }
}
</style>
