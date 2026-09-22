<template>
  <Widget :value="read()" class="box" id="root" :next="write()" />
  <Widget title="{{ first() }}" id="root" />
</template>
