export const navigationRendererFixtures = [
  {
    filename: "BreadcrumbConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "./breadcrumb.ts";
</script>

<template>
  <Breadcrumb label="Docs path">
    <BreadcrumbList>
      <BreadcrumbItem>
        <BreadcrumbLink href="/">Home</BreadcrumbLink>
        <BreadcrumbSeparator>/</BreadcrumbSeparator>
      </BreadcrumbItem>
      <BreadcrumbItem current>
        <BreadcrumbLink current="page">Docs</BreadcrumbLink>
      </BreadcrumbItem>
    </BreadcrumbList>
  </Breadcrumb>
</template>
`,
  },
  {
    filename: "TabsConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "./tabs.ts";
</script>

<template>
  <TabsRoot default-value="overview">
    <TabsList aria-label="Sections">
      <TabsTrigger value="overview">Overview</TabsTrigger>
      <TabsTrigger value="activity">Activity</TabsTrigger>
    </TabsList>
    <TabsContent value="overview">Overview panel</TabsContent>
    <TabsContent value="activity">Activity panel</TabsContent>
  </TabsRoot>
</template>
`,
  },
  {
    filename: "StepperConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { StepperContent, StepperItem, StepperList, StepperRoot, StepperTrigger } from "./stepper.ts";
</script>

<template>
  <StepperRoot default-value="shipping">
    <StepperList aria-label="Checkout">
      <StepperItem completed value="shipping">
        <StepperTrigger>Shipping</StepperTrigger>
      </StepperItem>
      <StepperItem value="billing">
        <StepperTrigger>Billing</StepperTrigger>
      </StepperItem>
    </StepperList>
    <StepperContent value="shipping">Shipping panel</StepperContent>
    <StepperContent value="billing">Billing panel</StepperContent>
  </StepperRoot>
</template>
`,
  },
] as const;
