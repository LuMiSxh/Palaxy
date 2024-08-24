<script lang="ts">
	import { bridge } from "$lib/functions"
	import { onMount } from "svelte"
	import { type CommandAnalyze } from "$components/converter/types"
	import {
		bundleRecommendation,
		disableBack,
		disableNext,
		loading,
	} from "$components/converter/stores"

	let results: CommandAnalyze | undefined

	// Initial state
	$loading = true
	$disableBack = true
	$disableNext = true

	onMount(async () => {
		const result = await bridge<CommandAnalyze>("analyze")
		$loading = false
		$disableBack = false

		if (result) {
			results = result
			$bundleRecommendation = results.flag
			$disableNext = results.negative.length > 0
		}
	})
</script>

<div
	class="flex w-5/6 flex-col items-center justify-center p-3"
	style="height: calc(100vh - 14rem)"
>
	{#if results}
		<div class="flex w-full flex-grow flex-col justify-center gap-3 overflow-y-auto">
			{#each results.negative as sentence}
				<div class="card variant-glass-error flex items-center p-2">
					<p class="text-start">{sentence}</p>
				</div>
			{/each}
			{#if results.negative.length === 0}
				{#each results.positive as sentence}
					<div class="card variant-glass-success flex items-center p-2">
						<p class="text-start">{sentence}</p>
					</div>
				{/each}
			{/if}
			{#each results.suggest as sentence}
				<div class="card variant-glass-secondary flex items-center p-2">
					<p class="text-start">{sentence}</p>
				</div>
			{/each}
		</div>
	{/if}
</div>
