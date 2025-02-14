<script lang="ts">
	import { onMount } from 'svelte';
	import { stepState } from '$states/converter.svelte';
	import { IconCircleMinus, IconCirclePlus, IconExclamationCircle } from '@tabler/icons-svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';

	let positives: string[] = $state([]);
	let negatives: string[] = $state([]);
	let suggestions: string[] = $state([]);

	onMount(async () => {
		// Disable the next button, enable the previous button
		stepState.disableNext = true;
		stepState.disablePrev = false;

		const result = await wrapper(commands.convAnalyze());
		if (result !== null && result.payload !== null) {
			positives = result.payload.positive;
			negatives = result.payload.negative;
			suggestions = result.payload.suggest;
		}
	});
</script>

{#snippet item(message: string, icon: any)}
	<li class="list row">
		{@render icon()}
		<span class="list-col-grow">
			{message}
		</span>
	</li>
{/snippet}

<ul class="list">
	{#each negatives as negative}
		{@render item(negative, IconCircleMinus)}
	{/each}
	{#each positives as positive}
		{@render item(positive, IconCirclePlus)}
	{/each}
	{#each suggestions as suggestion}
		{@render item(suggestion, IconExclamationCircle)}
	{/each}
</ul>
