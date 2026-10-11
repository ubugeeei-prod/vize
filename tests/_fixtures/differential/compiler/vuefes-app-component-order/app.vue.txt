<script setup lang="ts">
import "~/assets/styles/main.css";
import { useAutoThemeChanger } from "./stores/animation";

import { useI18n, useHead, useSeoMeta, defineOgImage } from "#imports";
import { NuxtPage, NuxtRouteAnnouncer } from "#components";

const { t } = useI18n();

// NOTE: use getter to handle locale changes
const title = () => `${t("nuxtSiteConfig.name")} %separator %s`;
const description = () => t("nuxtSiteConfig.description");
const ogImage = "https://vuefes.jp/2025/og-image.png";

defineOgImage({ component: "root", url: ogImage });
useHead({ templateParams: { separator: "-" } });
useSeoMeta({
  titleTemplate: title,
  ogTitle: title,
  description,
  ogDescription: description,
});

useAutoThemeChanger();
</script>

<template>
  <NuxtRouteAnnouncer />

  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
