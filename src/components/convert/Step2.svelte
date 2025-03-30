<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { t } from 'svelte-i18n-lingui';
	import convState, { stepState } from '$states/converter.svelte';
	import { IconCircleMinus, IconCirclePlus, IconExclamationCircle } from '@tabler/icons-svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';

	let positives: string[] = $state([]);
	let negatives: string[] = $state([]);
	let warnings: string[] = $state([]);
	let isLoading: boolean = $state(true);
	let resultsContainer: HTMLElement | null = $state(null);
	let cleanupKeyHint: (() => void) | null = null;

	onMount(async () => {
		// Disable the next button, enable the previous button
		stepState.disableNext = true;
		stepState.disablePrev = false;

		const result = await wrapper(commands.convAnalyze());
		isLoading = false;

		if (result !== null && result.payload !== null) {
			convState.bundleRecommendation = result.payload.flag;
			positives = result.payload.positive;
			negatives = result.payload.negative;
			warnings = result.payload.warning;

			// When there are no negatives, enable the next button
			if (negatives.length === 0) {
				stepState.disableNext = false;
			}
		}

		// Focus on the results container after ui render pass
		await tick()
		resultsContainer?.focus()

		// Show the key hint for navigating
		cleanupKeyHint = keyHint.smartAdd([
			['arrowdown', $t`Scroll down`],
			['arrowup', $t`Scroll up`]
		]);
	});

	onDestroy(() => {
		if (cleanupKeyHint) {
			cleanupKeyHint();
		}
	});
</script>

<div class="flex h-full w-full flex-col p-4" style="max-height: calc(100vh - 8rem)">
	{#if isLoading}
		<div class="flex h-full items-center justify-center">
			<LoadingSpinner text={$t`Analyzing source material...`} />
		</div>
	{:else}
		<div class="card">
			<div class="card-header">
				<h3 class="font-semibold">
					{$t`Analysis Results`}
				</h3>
			</div>
			<div class="card-body">
				<div
					class="results-container !focus:outline-none"
					bind:this={resultsContainer}
					tabindex="0"
					role="tab"
					style="max-height: calc(100vh - 14rem)"
				>
					{#if negatives.length > 0 || warnings.length > 0 || positives.length > 0}
						<div class="space-y-6">
							{#if negatives.length > 0}
								<div>
									<h3 class="text-error mb-3 flex items-center text-lg font-semibold">
										<IconCircleMinus size={20} class="mr-2" />
										{$t`Issues Found`}
									</h3>
									<div class="space-y-2">
										{#each negatives as negative, i (i)}
											<div
												class="list-row items-center rounded bg-error/10 px-4 py-3 border-l-3 border-error"
												style="--index: {i}"
											>
												<span class="list-col-grow text-md">{negative}</span>
											</div>
										{/each}
									</div>
								</div>
							{/if}

							{#if warnings.length > 0}
								<div>
									<h3 class="text-warning mb-3 flex items-center text-lg font-semibold">
										<IconExclamationCircle size={20} class="mr-2" />
										{$t`Warnings`}
									</h3>
									<div class="space-y-2">
										{#each warnings as warning, i (i)}
											<div
												class="list-row items-center rounded bg-warning/10 px-4 py-3 border-l-3 border-warning"
												style="--index: {i + negatives.length}"
											>
												<span class="list-col-grow text-md">{warning}</span>
											</div>
										{/each}
									</div>
								</div>
							{/if}

							{#if positives.length > 0}
								<div>
									<h3 class="text-success mb-3 flex items-center text-lg font-semibold">
										<IconCirclePlus size={20} class="mr-2" />
										{$t`Good Points`}
									</h3>
									<div class="space-y-2">
										{#each positives as positive, i (i)}
											<div
												class="list-row items-center rounded bg-success/10 px-4 py-3 border-l-3 border-success"
												style="--index: {i + negatives.length + warnings.length}"
											>
												<span class="list-col-grow text-md">{positive}</span>
											</div>
										{/each}
									</div>
								</div>
							{/if}
						</div>
					{:else}
						<div class="alert alert-info">
							<p>{$t`No analysis results found. There might be an issue with the source material.`}</p>
						</div>
					{/if}
				</div>

				{#if negatives.length > 0}
					<div class="alert alert-error mt-6">
						<p class="flex items-center">
							<IconCircleMinus size={20} class="mr-2" />
							{$t`Please resolve these issues before continuing.`}
						</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.results-container {
		overflow-y: auto;
		padding-right: 0.5rem;
		outline: none;
		scroll-behavior: smooth;
	}

  /* Forcefully remove all focus indicators */
  .results-container:focus {
      outline: none !important;
      box-shadow: none !important;
      -webkit-box-shadow: none !important;
      -moz-box-shadow: none !important;
  }

  .list-row {
		display: flex;
		align-items: center;
	}

	.list-col-grow {
		flex-grow: 1;
	}

	/* Border color styles */
	.border-l-3 {
		border-left-width: 3px;
		border-left-style: solid;
	}
</style>
