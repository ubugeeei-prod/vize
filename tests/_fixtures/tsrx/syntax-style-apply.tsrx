const t = <style>.a { color: red; }</style>;
export function A() @{
	<>
		<style apply={t} />
		<p class={t.a}>x</p>
	</>
}
