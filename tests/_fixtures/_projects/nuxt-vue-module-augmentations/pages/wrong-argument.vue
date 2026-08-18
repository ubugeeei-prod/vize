<template>
  <!-- INTENTIONAL type errors. Each global resolves to its real declared
       signature, so a wrong-typed call is a real TS2345. If any of these stops
       being reported, an `any` or `unknown` stand-in has come back. -->
  <p>{{ $shout(1) }}</p>
  <p>{{ $local(2) }}</p>
</template>
