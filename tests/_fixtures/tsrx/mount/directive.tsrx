'use client';

import { ref } from 'vue';

export function Client({ label }: { label: string }) @{
	const count = ref(0);

	<button onClick={() => count.value++}>{label}: {count.value}</button>
}
