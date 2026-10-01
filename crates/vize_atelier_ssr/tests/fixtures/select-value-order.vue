<template><select v-model="model"><option :value="bound" value="static">bound first</option><option value="static" :value="bound">static first</option></select></template>
