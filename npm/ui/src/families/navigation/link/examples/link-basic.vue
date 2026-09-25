<!-- Account navigation links marking the current page and a disabled destination. -->
<script setup lang="ts">
import { Link } from "../link.ts";
</script>

<template>
  <nav aria-label="Account">
    <ul>
      <li><Link href="/account" aria-current="page">Overview</Link></li>
      <li><Link href="/account/billing">Billing</Link></li>
      <li><Link href="/account/team" disabled>Team (upgrade required)</Link></li>
      <li>
        <Link href="https://status.example.com" target="_blank" rel="noopener noreferrer"
          >Service status (opens in a new tab)</Link
        >
      </li>
    </ul>
  </nav>
</template>
