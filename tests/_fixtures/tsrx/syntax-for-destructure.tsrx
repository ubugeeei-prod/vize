export function A({ o }: { o: Record<string, number> }) @{
	<ul>
		@for (const [k, v] of Object.entries(o); key k) {
			<li>{k}={v}</li>
		}
	</ul>
}
