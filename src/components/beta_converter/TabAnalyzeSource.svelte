<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { bridge } from '$lib/functions';
	import { onMount } from 'svelte';
	import { ProgressRadial } from '@skeletonlabs/skeleton';
	import ZoomExclamation from '@tabler/icons-svelte/IconZoomExclamation.svelte';
	import { converterTab } from '$lib/stores';
	import { BundlerFlag, type TabAnalyzeResult } from '$components/beta_converter/types';

	export let sourceManga: Writable<string | null>;
	export let bundlerRecommendation: Writable<BundlerFlag>;
	let loading = true;
	let results: TabAnalyzeResult | undefined;

	onMount(async () => {
		if ($sourceManga === null) {
			$converterTab = 0;
			return;
		}

		const result = await bridge<TabAnalyzeResult>('pipe_analyze', { basePath: $sourceManga });
		loading = false;

		if (result) {
			results = result;
			$bundlerRecommendation = results.bundler;
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

			<!-- Responsive Container (recommended) -->
			<div class="table-container">
				<!-- Native Table Element -->
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
			<div class="w-full flex justify-evenly pt-4">
				<button
					on:click={() => converterTab.update(tab => tab-1)}
					class="btn variant-ghost-secondary btn-lg"
				>
					Go Back
				</button>
				<button
					disabled={results.negative.length > 0}
					on:click={() => converterTab.update(tab => tab+1)}
					class="btn variant-ghost-primary btn-lg"
				>
					Next Step
				</button>
			</div>
		{:else}
			<div class="h-full w-full flex flex-col items-center justify-center">
				<ZoomExclamation class="w-1/4 h-full text-error-500 animate-pulse" />
				<button
					on:click={() => converterTab.update(tab => tab-1)}
					class="btn variant-ghost-error btn-lg"
				>
					Go Back
				</button>
			</div>
		{/if}
	{/if}
</div>
