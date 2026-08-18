<script setup lang="ts">
// Three `ComponentCustomProperties` augmentations reach this template: one from
// an installed package (`$t`), one from the generated `.nuxt` types (`$shout`),
// and one from the app's own `types/` directory (`$local`). All three are
// declared, so this page must type-check clean.
const { locale } = useI18n()
</script>

<template>
  <p>{{ $t('nav.home') }}</p>
  <p>{{ $shout(locale) }}</p>
  <p>{{ $local('sidebar') }}</p>
</template>
