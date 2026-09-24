<!-- Image avatar with alt text beside an initials fallback exposed as a named image. -->
<script setup lang="ts">
import { Avatar } from "../avatar.ts";
</script>

<template>
  <ul>
    <li>
      <Avatar
        src="https://example.com/avatars/ada-lovelace.png"
        alt="Ada Lovelace"
        name="Ada Lovelace"
        fallback="AL"
        status="online"
        loading="lazy"
      />
      Ada Lovelace (online)
    </li>
    <li>
      <Avatar
        role="img"
        aria-label="Grace Hopper"
        name="Grace Hopper"
        fallback="GH"
        status="away"
      />
      Grace Hopper (away)
    </li>
  </ul>
</template>
