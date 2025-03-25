<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { t } from 'svelte-i18n-lingui';
	import convState, { stepState } from '$states/converter.svelte';
	import { IconCircleMinus, IconCirclePlus, IconExclamationCircle } from '@tabler/icons-svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';
	import { keyboard } from '$lib/keyboard';
	import { keyHint } from '$states/keyhint.svelte';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';

	let positives: string[] = $state([]);
	let negatives: string[] = $state([]);
	let warnings: string[] = $state([]);
	let isLoading: boolean = $state(true);
	let resultsContainer: HTMLElement | null = $state(null);
	let allListItems: HTMLElement[] = $state([]);
	let currentFocusIndex: number = $state(-1);
	let cleanupKeyboard: (() => void) | null = null;
	let cleanupKeyHint: (() => void) | null = null;

	// Get all list items and focus the first one
	function updateListItems() {
		if (resultsContainer) {
			allListItems = Array.from(resultsContainer.querySelectorAll('.list-row'));

			// Reset focus index when list changes
			if (currentFocusIndex >= allListItems.length) {
				currentFocusIndex = allListItems.length > 0 ? 0 : -1;
			}

			// Focus the current item if needed
			if (currentFocusIndex >= 0 && allListItems.length > 0) {
				focusItem(currentFocusIndex);
			}
		}
	}

	// Focus a specific item and scroll it into view
	function focusItem(index: number) {
		if (index >= 0 && index < allListItems.length) {
			// Remove focus from all items
			allListItems.forEach((item) => item.classList.remove('focused-item'));

			// Add focus to current item
			const currentItem = allListItems[index];
			currentItem.classList.add('focused-item');

			// If this is the first item in a section, try to scroll the heading into view too
			const isFirstItemInSection =
				currentItem.previousElementSibling === null ||
				!currentItem.previousElementSibling.classList.contains('list-row');

			if (
				isFirstItemInSection &&
				currentItem.parentElement &&
				currentItem.parentElement.previousElementSibling
			) {
				// This is likely the first item after a heading, so scroll the heading into view
				currentItem.parentElement.previousElementSibling.scrollIntoView({
					behavior: 'smooth',
					block: 'start'
				});
			} else {
				// Otherwise just scroll the item into view
				currentItem.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
			}

			currentFocusIndex = index;
		}
	}

	// Navigate to the next or previous item
	function navigateItems(direction: 'next' | 'prev') {
		if (allListItems.length === 0) return;

		let newIndex = currentFocusIndex;

		if (direction === 'next') {
			newIndex =
				currentFocusIndex < allListItems.length - 1 ? currentFocusIndex + 1 : currentFocusIndex;
		} else {
			newIndex = currentFocusIndex > 0 ? currentFocusIndex - 1 : currentFocusIndex;
		}

		if (newIndex !== currentFocusIndex) {
			focusItem(newIndex);
		}
	}

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

		// Update list items after the content is loaded
		setTimeout(updateListItems, 100);

		// Register keyboard handlers for navigating between items
		cleanupKeyboard = keyboard.smartRegister([
			[
				'arrowup',
				(event) => {
					event.preventDefault();
					navigateItems('prev');
				}
			],
			[
				'arrowdown',
				(event) => {
					event.preventDefault();
					navigateItems('next');
				}
			]
		]);

		// Show the key hint for navigating
		cleanupKeyHint = keyHint.smartAdd([
			['arrowdown', $t`Scroll down`],
			['arrowup', $t`Scroll up`]
		]);
	});

	onDestroy(() => {
		if (cleanupKeyboard) {
			cleanupKeyboard();
		}

		if (cleanupKeyHint) {
			cleanupKeyHint();
		}
	});

	// Watch for content changes to update the list items
	$effect(() => {
		if (!isLoading && (positives.length || negatives.length || warnings.length)) {
			setTimeout(updateListItems, 100);
		}
	});
</script>

{#snippet item(message: string, Icon: any, bg: string, txt: string)}
	<li class="list-row items-center rounded {bg} mb-2 p-3" style="--text-color: {txt}">
		<Icon class="{txt} icon mr-3" size={24} />
		<span class="list-col-grow text-md {txt}">
			{message}
		</span>
	</li>
{/snippet}

<div class="w-full max-w-3xl">
	{#if isLoading}
		<LoadingSpinner text={$t`Analyzing source material...`} />
	{:else}
		<div class="results-container" bind:this={resultsContainer} tabindex="-1">
			{#if negatives.length > 0}
				<div class="mb-4">
					<h3 class="text-error mb-2 flex items-center text-lg font-semibold">
						{$t`Issues Found`}
					</h3>
					<div class="space-y-2">
						{#each negatives as negative, i (i)}
							{@render item(negative, IconCircleMinus, 'bg-error/20', 'text-error')}
						{/each}
					</div>
				</div>
			{/if}

			{#if warnings.length > 0}
				<div class="mb-4">
					<h3 class="text-warning mb-2 flex items-center text-lg font-semibold">
						{$t`Warnings`}
					</h3>
					<div class="space-y-2">
						{#each warnings as warning, i (i)}
							{@render item(warning, IconExclamationCircle, 'bg-warning/20', 'text-warning')}
						{/each}
					</div>
				</div>
			{/if}

			{#if positives.length > 0}
				<div>
					<h3 class="text-success mb-2 flex items-center text-lg font-semibold">
						{$t`Good Points`}
					</h3>
					<div class="space-y-2">
						{#each positives as positive, i (i)}
							{@render item(positive, IconCirclePlus, 'bg-success/20', 'text-success')}
						{/each}
					</div>
				</div>
			{/if}

			{#if positives.length === 0 && negatives.length === 0 && warnings.length === 0}
				<div class="alert alert-info">
					<p>{$t`No analysis results found. There might be an issue with the source material.`}</p>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.results-container {
		max-height: 70vh;
		overflow-y: auto;
		padding-right: 0.5rem;
		outline: none;
		scroll-behavior: smooth;
	}

	.list-row {
		display: flex;
		align-items: center;
		transition: all 0.3s ease;
		border-left: 3px solid transparent;
		padding-left: calc(0.75rem - 3px);
	}

	.list-row:hover {
		transform: translateX(5px);
		border-left-color: var(--text-color, currentColor);
	}

	.list-row.focused-item {
		transform: translateX(5px);
		border-left-color: var(--text-color, currentColor);
		outline: none;
	}

	.list-col-grow {
		flex-grow: 1;
	}

	.icon {
		transition: transform 0.2s ease;
	}

	.list-row:hover .icon,
	.list-row.focused-item .icon {
		transform: scale(1.15);
	}

	.list-row {
		animation: var(--animate-slide-right);
		opacity: 0;
		animation-fill-mode: forwards;
		animation-delay: calc(var(--index, 0) * 0.1s);
	}

	/* Add staggered animation for multiple items */
	.space-y-2 > :nth-child(1) {
		--index: 1;
	}

	.space-y-2 > :nth-child(2) {
		--index: 2;
	}

	.space-y-2 > :nth-child(3) {
		--index: 3;
	}

	.space-y-2 > :nth-child(4) {
		--index: 4;
	}

	.space-y-2 > :nth-child(5) {
		--index: 5;
	}

	.space-y-2 > :nth-child(n + 6) {
		--index: 6;
	}
</style>
