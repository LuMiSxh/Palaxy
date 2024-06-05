<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { bridge } from '$lib/functions';
	import { onMount } from 'svelte';
	import { ProgressRadial } from '@skeletonlabs/skeleton';
	import { converterTab, tabDisableBack, tabDisableNext } from '$lib/stores';
	import { BundlerFlag, type TabAnalyzeResult } from '$components/converter/types';

	export let sourceManga: Writable<string | null>;
	export let bundlerRecommendation: Writable<BundlerFlag>;
	let loading = true;
	let results: TabAnalyzeResult | undefined;

	// Initial state
	$tabDisableBack = false;
	$tabDisableNext = true;

	onMount(async () => {
		if ($sourceManga === null) {
			$converterTab = 0;
			return;
		}

		const result = await bridge<TabAnalyzeResult>('flow_analyze', { basePath: $sourceManga });
		loading = false;

		if (result) {
			results = result;
			$bundlerRecommendation = results.bundler;
			$tabDisableNext = results.negative.length > 0;
		}
	});

</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	{#if loading}
		<div class="h-full w-full flex items-center justify-center animate-pulse">
			<ProgressRadial value={undefined} strokeLinecap="round" meter="stroke-secondary-500" />
		</div>
	{:else}
		{#if results}
			<div class="table-container w-3/4">
				<table class="table table-hover">
					<thead>
					<tr>
						<th class="table-header">Information</th>
					</tr>
					</thead>
					<tbody>
					{#each results.negative as sentence}
						<tr class="!variant-glass-error">
							<td class="!whitespace-pre-wrap">{sentence}</td>
						</tr>
					{/each}
					{#if results.negative.length === 0}
						{#each results.positive as sentence}
							<tr class="!variant-glass-success">
								<td class="!whitespace-pre-wrap">{sentence}</td>
							</tr>
						{/each}
						{#each results.suggest as sentence}
							<tr class="!variant-glass-secondary">
								<td class="!whitespace-pre-wrap">{sentence}</td>
							</tr>
						{/each}
					{/if}
					</tbody>
				</table>
			</div>
		{/if}
	{/if}
</div>
