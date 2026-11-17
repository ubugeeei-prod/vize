function Badge(props: { text: string }) @{
	<b class="badge">{props.text}</b>
}

export default function App({ title, count }: { title: string; count: number }) @{
	const heading = title.toUpperCase();
	<section class="app">
		<h1>{heading}</h1>
		<Badge text={title} />
		<p>{count}</p>
	</section>
}
