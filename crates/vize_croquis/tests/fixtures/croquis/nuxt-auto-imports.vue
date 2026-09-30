<script setup lang="ts">
// The reproduction from the issue: Nuxt and Nuxt-module auto-imports used with
// no import statement, plus the `$t` template global. Every one of them is
// declared by the generated `.nuxt` types, so this page must type-check clean.
const { locale } = useI18n()
const localePath = useLocalePath()
const page = await queryCollection('content').path('/about').first()
</script>

<template>
  <a :href="localePath('/')">{{ $t('nav.home') }}</a>
  <p>{{ $shout(locale) }}</p>
  <pre>{{ page?.title }}</pre>
</template>
