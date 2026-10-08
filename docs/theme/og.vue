<script setup lang="ts">
const props = defineProps<{
  title: string;
  description?: string;
  siteName?: string;
  category?: string;
  localeName?: string;
  locale?: string;
  route?: string;
  isHome?: boolean;
  assetFingerprint?: string;
}>();
</script>

<template>
  <div
    class="og"
    :lang="props.locale"
    :class="{
      'is-home': props.isHome,
      'long-title': props.title.length > 38,
      'very-long-title': props.title.length > 70,
    }"
  >
    <div class="left">
      <div class="header">
        <img alt="" class="header-logo" height="28" src="/logo.svg" width="28" />
        <span class="site-name"> VIZE </span>
        <span v-if="!props.isHome" class="category"
          >{{ props.category }} · {{ props.localeName }}</span
        >
      </div>
      <div class="title-area">
        <h1 class="title">
          {{ props.title }}
        </h1>
        <p class="desc">
          {{ props.description || "High-Performance Vue.js Toolchain in Rust" }}
        </p>
      </div>
      <span class="url">{{ props.isHome ? "vizejs.dev" : `vizejs.dev${props.route}` }}</span>
    </div>
    <div class="right">
      <img alt="" class="logo" src="/logo.svg" />
      <span class="tagline"> High-Performance Vue.js Toolchain in Rust </span>
    </div>
  </div>
</template>

<style scoped>
.og {
  --og-paper: #e6e2d6;
  --og-ink: #121212;
  --og-muted-ink: #5a5750;
  --og-label-size: 14px;
  --og-title-size: 96px;
  --og-description-size: 21px;
  --og-caption-size: 10px;
  --og-category-size: 16px;
  --og-url-size: 12px;
  --og-content-layer: 1;

  width: 1200px;
  height: 630px;
  background: var(--og-paper);
  position: relative;
  overflow: hidden;
  font-family:
    Helvetica Neue,
    Helvetica,
    Arial,
    system-ui,
    sans-serif;
  color: var(--og-ink);
  display: flex;
}

.left {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding-block: 56px;
  padding-inline: 72px 0;
  position: relative;
  z-index: var(--og-content-layer);
}

.right {
  width: 420px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-logo {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.site-name {
  font-size: var(--og-label-size);
  font-weight: 600;
  letter-spacing: 0.2em;
  font-family:
    JetBrains Mono,
    ui-monospace,
    SFMono-Regular,
    SF Mono,
    Menlo,
    Consolas,
    monospace;
  color: var(--og-muted-ink);
}

.title-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.title {
  font-size: var(--og-title-size);
  font-weight: 700;
  letter-spacing: -0.06em;
  line-height: 1.15;
  margin: 0;
  padding-block-end: 4px;
  max-width: 680px;
}

.desc {
  font-size: var(--og-description-size);
  font-weight: 500;
  color: var(--og-muted-ink);
  line-height: 1.35;
  letter-spacing: -0.02em;
  margin-block: 32px 0;
  margin-inline: 0;
  max-width: 480px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.url {
  font-size: var(--og-label-size);
  font-weight: 500;
  color: var(--og-muted-ink);
  font-family:
    JetBrains Mono,
    ui-monospace,
    SFMono-Regular,
    SF Mono,
    Menlo,
    Consolas,
    monospace;
  letter-spacing: 0.02em;
}

.logo {
  width: auto;
  height: 340px;
  opacity: 0.85;
  position: absolute;
  inset-inline-end: -16px;
  inset-block-start: 50%;
  transform: translateY(-50%);
}

.tagline {
  position: absolute;
  /* This element writes vertically: block-start is right, inline-end is bottom. */
  inset-block-start: 40px;
  inset-inline-end: 56px;
  font-family:
    JetBrains Mono,
    ui-monospace,
    SFMono-Regular,
    SF Mono,
    Menlo,
    Consolas,
    monospace;
  font-size: var(--og-caption-size);
  font-weight: 400;
  letter-spacing: 0.02em;
  line-height: 1.6;
  color: var(--og-muted-ink);
  writing-mode: vertical-rl;
  text-orientation: mixed;
  max-height: 240px;
  opacity: 0.5;
}

.og:not(.is-home) {
  --og-title-size: 72px;
  font-family:
    "Helvetica Neue", Helvetica, Arial, "Noto Sans CJK JP", "Hiragino Kaku Gothic ProN",
    "Yu Gothic", system-ui, sans-serif;

  .title {
    max-width: 640px;
    overflow-wrap: anywhere;
    letter-spacing: -0.04em;
  }

  .url {
    max-width: 650px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--og-url-size);
  }

  .right {
    width: 360px;
  }

  .logo {
    height: 280px;
    opacity: 0.7;
  }
}

.og:not(.is-home).long-title {
  --og-title-size: 56px;
}

.og:not(.is-home).very-long-title {
  --og-title-size: 44px;
}

.category {
  font-size: var(--og-category-size);
  color: var(--og-muted-ink);
  margin-inline-start: 16px;
}
</style>
