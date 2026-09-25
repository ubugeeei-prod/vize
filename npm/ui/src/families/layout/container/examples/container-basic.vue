<!-- Centered page container that limits the reading width of its content. -->
<script setup lang="ts">
import { Container } from "../container.ts";
</script>

<template>
  <Container as="main" size="lg" :padding-inline="24">
    <h1>Team handbook</h1>
    <p>Everything new teammates need to know during their first week.</p>
  </Container>
</template>
