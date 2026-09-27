import type { Slots } from './Single.vue';
type Payload = Parameters<NonNullable<Slots['single']>>[0];
export const different: Payload = { str: 'different', num: 2 };
export const wrong: Payload = { str: 'different', num: 'wrong' };
