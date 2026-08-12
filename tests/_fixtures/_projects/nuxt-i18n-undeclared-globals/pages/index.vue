<script setup lang="ts">
// The composables resolve from the generated declarations, so the script is
// clean. `$t` in the template is not declared anywhere, and an auto-imported
// `useI18n` is not a declaration of it: every authored `$t` must be reported.
const { locale, t } = useI18n()
const localePath = useLocalePath()
</script>

<template>
  <a :href="localePath('/')">{{ t('nav.home') }}</a>
  <p>{{ $t('nav.about') }}</p>
  <p :title="$t('nav.help')">{{ locale }}</p>
</template>
