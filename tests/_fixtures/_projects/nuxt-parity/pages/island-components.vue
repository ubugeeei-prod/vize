<template>
  <ServerProps title="ok" />
  <ServerProps :title="123" />
  <ServerProps />
  <LazyServerProps title="ok" :hydrate-when="true" />
  <LazyServerProps :title="123" />
  <LazyServerProps />
</template>
