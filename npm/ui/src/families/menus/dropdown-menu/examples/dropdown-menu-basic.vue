<!-- Account menu button with grouped actions, a checkbox item, and a destructive item. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuItemIndicator,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../dropdown-menu.ts";

const open = ref(false);
const away = ref(false);
const lastAction = ref("none");
</script>

<template>
  <div>
    <DropdownMenu v-model:open="open">
      <DropdownMenuTrigger>Account</DropdownMenuTrigger>
      <DropdownMenuContent aria-label="Account">
        <DropdownMenuGroup>
          <DropdownMenuLabel>Signed in as ada@example.com</DropdownMenuLabel>
          <DropdownMenuItem @select="() => (lastAction = 'Profile')">Profile</DropdownMenuItem>
          <DropdownMenuItem @select="() => (lastAction = 'Billing')">Billing</DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem v-model="away">
          <DropdownMenuItemIndicator>✓</DropdownMenuItemIndicator>
          Appear away
        </DropdownMenuCheckboxItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem @select="() => (lastAction = 'Sign out')">Sign out</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
    <output>Last action: {{ lastAction }}; away: {{ away ? "yes" : "no" }}</output>
  </div>
</template>
