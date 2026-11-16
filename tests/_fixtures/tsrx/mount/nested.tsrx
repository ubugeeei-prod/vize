import { ref } from 'vue';

export function Toggle({ on, n }: { on: boolean; n: number }) @{
	<>
		<h1>{n}</h1>
		{on ? <em class="on">yes {n}</em> : <span>no</span>}
	</>
}

export function Modes({ mode, label }: { mode: string; label: string }) @{
	<>
		{mode === 'a' ? <Badge text={label} /> : mode === 'b' ? <b>{label}</b> : null}
		<hr />
	</>
}

function Badge({ text }: { text: string }) @{
	<i class="badge">{text}</i>
}

export function Guard({ count, label }: { count: number; label: string }) @{
	<>
		{count > 0 && <p data-label={label}>{label}</p>}
	</>
}

export function Disclosure({ title }: { title: string }) @{
	const open = ref(false);

	<>
		<button onClick={() => (open.value = !open.value)}>{title}</button>
		{open.value && title.length > 3 ? <section class="long">{title}</section> : open.value ? <section>short</section> : null}
	</>
}
