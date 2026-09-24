<!-- QR code SVG that encodes a URL with an explicit accessible name. -->
<script setup lang="ts">
import { QrCode } from "../qr-code.ts";
</script>

<template>
  <figure>
    <QrCode value="https://vizejs.dev" label="QR code linking to vizejs.dev" error-correction="Q" />
    <figcaption>Scan to open the Vize documentation.</figcaption>
  </figure>
</template>
