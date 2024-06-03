<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { converterTab } from '$lib/stores';
	import { BundlerFlag, type TabVolumeResult } from '$components/beta_converter/types';
	import { onMount } from 'svelte';
	import { bridge } from '$lib/functions';
	import { ProgressRadial } from '@skeletonlabs/skeleton';

	export let bundler: Writable<BundlerFlag>;
	export let sourceManga: Writable<string | null>;

	let loading = true;
	let results: TabVolumeResult | undefined;

	onMount(async () => {
		if ($sourceManga === null) {
			$converterTab = 0;
			return;
		}

		const result = await bridge<TabVolumeResult>(
			'pipe_volume',
			{ basePath: $sourceManga, bundlerFlag: $bundler }
		);
		loading = false;

		if (result) results = result;
	});
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	{#if loading}
		<div class="h-full w-full flex items-center justify-center animate-pulse">
			<ProgressRadial value={undefined} strokeLinecap="round" meter="stroke-secondary-500" />
		</div>
	{:else}
		{#if $bundler === BundlerFlag.MANUAL}
			<div>
				Manual
				{JSON.stringify(results)}
			</div>
		{:else if $bundler === BundlerFlag.IMAGE}
			<div>
				Image
				{JSON.stringify(results)}
			</div>
		{:else if $bundler === BundlerFlag.NAME}
			<div>
				Naming
				{JSON.stringify(results)}
			</div>
		{/if}
		<!-- Step selector -->
		<div class="w-full flex justify-evenly pt-4">
			<button
				on:click={() => converterTab.update(tab => tab-1)}
				disabled={false}
				class="btn variant-ghost-secondary btn-lg"
			>
				Go Back
			</button>
			<button
				disabled={true}
				on:click={() => converterTab.update(tab => tab+1)}
				class="btn variant-ghost-primary btn-lg"
			>
				Next Step
			</button>
		</div>
	{/if}
</div>