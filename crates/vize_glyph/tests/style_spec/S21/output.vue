<template>
  <Widget :value="read()" id="root" class="box" :next="write()" />
  <Widget title="{{ first() }}" id="root" />
</template>
