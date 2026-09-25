<!-- Documentation breadcrumb trail whose last link marks the current page. -->
<script setup lang="ts">
import {
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbRoot,
  BreadcrumbSeparator,
} from "../breadcrumb.ts";
</script>

<template>
  <BreadcrumbRoot label="Documentation path">
    <BreadcrumbList>
      <BreadcrumbItem>
        <BreadcrumbLink href="/">Home</BreadcrumbLink>
        <BreadcrumbSeparator>/</BreadcrumbSeparator>
      </BreadcrumbItem>
      <BreadcrumbItem>
        <BreadcrumbLink href="/docs">Docs</BreadcrumbLink>
        <BreadcrumbSeparator>/</BreadcrumbSeparator>
      </BreadcrumbItem>
      <BreadcrumbItem>
        <BreadcrumbLink href="/docs/components">Components</BreadcrumbLink>
        <BreadcrumbSeparator>/</BreadcrumbSeparator>
      </BreadcrumbItem>
      <BreadcrumbItem current>
        <BreadcrumbLink href="/docs/components/breadcrumb" current="page"
          >Breadcrumb</BreadcrumbLink
        >
      </BreadcrumbItem>
    </BreadcrumbList>
  </BreadcrumbRoot>
</template>
