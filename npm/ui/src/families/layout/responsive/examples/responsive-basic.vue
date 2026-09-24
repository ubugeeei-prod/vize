<!-- Navigation that switches between a compact and a full layout by breakpoint, rendered from an SSR width. -->
<script setup lang="ts">
import { ResponsiveShow, ResponsiveSwitch } from "../responsive.ts";
</script>

<template>
  <header>
    <ResponsiveSwitch as="nav" :ssr-width="1024" aria-label="Primary">
      <template #base>
        <a href="/menu">Menu</a>
      </template>
      <template #md>
        <a href="/">Home</a>
        <a href="/docs">Docs</a>
        <a href="/blog">Blog</a>
      </template>
    </ResponsiveSwitch>
    <ResponsiveShow above="lg" :ssr-width="1024">
      <a href="/signup">Create account</a>
    </ResponsiveShow>
  </header>
</template>
