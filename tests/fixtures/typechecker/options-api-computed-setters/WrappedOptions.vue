<template>
  <div>{{ ratio }} {{ fooBar }} {{ plain }} {{ absent }} {{ literal }} {{ arrow }}</div>
</template>
<script lang="ts">
const sizing = {
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
    },
}

export default {
    mixins: ([sizing] as const),
    props: (['foo-bar', 'plain'] as const),
    computed: {
        fooBar: {
            get(): string {
                return 'f'
            },
            set(_value: string) {},
        },
        plain: {
            get(): string {
                return 'p'
            },
            set(_value: string) {},
        },
        absent: {
            get(): string {
                return 'a'
            },
            set: undefined,
        },
        literal: {
            get(): string {
                return 'l'
            },
            set: null,
        },
        arrow: {
            get: () => 'r',
            set: (_value: string) => {},
        },
    },
}
</script>
