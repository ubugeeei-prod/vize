<script setup lang="ts">const valid = 'ok';</script>
<template>
<!-- @vue-ignore -->{{ ignoredMissing }}
<!-- @vue-expect-error -->{{ expectedMissing }}
<!-- @vue-expect-error -->{{ valid }}
<p>{{ realMissing }}</p>
<!-- @vue-ignore --><p :id="valid">{{ childMissing }}</p>
</template>