<!-- Uncontrolled tabs with a labelled tab list and matching panels. -->
<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "../tabs.ts";
</script>

<template>
  <TabsRoot default-value="overview">
    <TabsList aria-label="Product sections">
      <TabsTrigger value="overview">Overview</TabsTrigger>
      <TabsTrigger value="specs">Specifications</TabsTrigger>
      <TabsTrigger value="reviews" disabled>Reviews</TabsTrigger>
    </TabsList>
    <TabsContent value="overview">A lightweight jacket for everyday wear.</TabsContent>
    <TabsContent value="specs">Water resistant, 320 g, machine washable.</TabsContent>
    <TabsContent value="reviews">Reviews are coming soon.</TabsContent>
  </TabsRoot>
</template>
