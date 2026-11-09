export function Broken({ ready }: { ready: boolean }) @{
	<div>
		@if (ready) {
			return null;
		}
	</div>
}
