<template>
  <div v-bind:class="{on:isOn}" v-on:click.prevent.stop="save"></div>
</template>
