<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { BundlerFlag } from '$components/converter/types';
	import { tabDisableNext } from '$lib/stores';

	export let bundler: Writable<BundlerFlag | null>;
	export let sourceManga: Writable<string | null>;
	export let cpv: Writable<Array<number> | null>;

	// Initial state
	$tabDisableNext = false;
	$tabDisableNext = true;

</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	{$cpv}
</div>