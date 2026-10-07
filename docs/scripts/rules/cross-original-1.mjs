// Complete project contexts retained from the original reference.
export const original1 = {
  "duplicate-id": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./CheckoutForm.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "CheckoutForm.vue":
        '<script setup lang="ts">\nimport BillingAddress from "./BillingAddress.vue";\nimport ShippingAddress from "./ShippingAddress.vue";\n</script>\n\n<template>\n  <ShippingAddress />\n  <BillingAddress />\n</template>',
      "ShippingAddress.vue":
        '<template>\n  <label for="postal-code">Shipping postal code</label>\n  <input id="postal-code" />\n</template>',
      "BillingAddress.vue":
        '<template>\n  <label for="postal-code">Billing postal code</label>\n  <input id="postal-code" />\n</template>',
    },
    good: {
      "CheckoutForm.vue":
        '<script setup lang="ts">\nimport BillingAddress from "./BillingAddress.vue";\nimport ShippingAddress from "./ShippingAddress.vue";\n</script>\n\n<template>\n  <ShippingAddress />\n  <BillingAddress />\n</template>',
      "ShippingAddress.vue":
        '<script setup lang="ts">\nimport { useId } from "vue";\n\nconst postalCodeId = useId();\n</script>\n\n<template>\n  <label :for="postalCodeId">Shipping postal code</label>\n  <input :id="postalCodeId" />\n</template>',
      "BillingAddress.vue":
        '<script setup lang="ts">\nimport { useId } from "vue";\n\nconst postalCodeId = useId();\n</script>\n\n<template>\n  <label :for="postalCodeId">Billing postal code</label>\n  <input :id="postalCodeId" />\n</template>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "non-unique-id": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./ResultsList.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "ResultsList.vue":
        '<template>\n  <article v-for="result in results" :key="result.id">\n    <h2 id="result-title">{{ result.title }}</h2>\n  </article>\n</template>',
    },
    good: {
      "ResultsList.vue":
        '<template>\n  <article v-for="result in results" :key="result.id">\n    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>\n  </article>\n</template>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "spread-breaks-reactivity": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./UserPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada", role: "admin" });\n</script>\n\n<template>\n  <UserSummary :user="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nconst props = defineProps<{ user: { name: string; role: string } }>();\nconst copiedUser = { ...props.user };\n</script>',
    },
    good: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada", role: "admin" });\n</script>\n\n<template>\n  <UserSummary :user="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nimport { toRef } from "vue";\n\nconst props = defineProps<{ user: { name: string; role: string } }>();\nconst user = toRef(props, "user");\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "reassignment-breaks-reactivity": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./UserPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :user="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nimport { toRef } from "vue";\n\nconst props = defineProps<{ user: { name: string } }>();\nlet user = toRef(props, "user");\n\nuser = props.user;\n</script>',
    },
    good: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :user="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nimport { toRef } from "vue";\n\nconst props = defineProps<{ user: { name: string } }>();\nconst user = toRef(props, "user");\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "value-extraction-breaks-reactivity": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./UserPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :item="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nconst { item } = defineProps<{ item: { name: string } }>();\nconst itemSnapshot = item;\n</script>',
    },
    good: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :item="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nimport { computed } from "vue";\n\nconst { item } = defineProps<{ item: { name: string } }>();\nconst itemView = computed(() => item);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
};
