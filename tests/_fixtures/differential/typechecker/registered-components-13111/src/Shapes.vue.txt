<script lang="ts">
import { defineAsyncComponent, defineComponent } from "vue";
import { Menu } from "menu-library";
import Child from "./Child.vue";
import TsButton from "./TsButton";
import * as ui from "./ui";

export default defineComponent({
  components: {
    AliasedTs: TsButton,
    AliasedMenu: Menu,
    MemberButton: ui.Button,
    MemberCard: ui.Card,
    "string-key": Child,
    Lazy: defineAsyncComponent(() => import("./Child.vue")),
  },
});
</script>

<template>
  <AliasedTs size="sm" />
  <aliased-ts size="sm" />
  <AliasedMenu as="nav" />
  <MemberButton size="md" />
  <member-card label="card" />
  <string-key label="string" />
  <Lazy label="lazy" />
</template>
