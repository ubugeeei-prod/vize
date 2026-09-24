<!-- Profile preview that appears when a user link is hovered or focused. -->
<script setup lang="ts">
import { HoverCard, HoverCardArrow, HoverCardContent, HoverCardTrigger } from "../hover-card.ts";
</script>

<template>
  <p>
    Reviewed by
    <HoverCard :open-delay="300" :close-delay="150">
      <HoverCardTrigger href="/users/ada">@ada</HoverCardTrigger>
      <HoverCardContent placement="bottom">
        <strong>Ada Lovelace</strong>
        <span>Staff engineer on the compiler team.</span>
        <HoverCardArrow />
      </HoverCardContent>
    </HoverCard>
    on Monday.
  </p>
</template>
