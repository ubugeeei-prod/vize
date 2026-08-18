<template>
  <!-- Negative control. No augmentation anywhere declares `$missing`, so both
       tools must report it once per authored use rather than resolving it. -->
  <p>{{ $missing('a') }}</p>
  <p>{{ $missing('b') }}</p>
  <p :title="$missing('c')">x</p>
</template>
