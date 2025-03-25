<script lang="ts">
	import convState from '$states/converter.svelte';
	import { t } from 'svelte-i18n-lingui';
	import {
		checkChapterLimits,
		getChaptersPercentage,
		getTotalChapters,
		step4State
	} from '$components/convert/step4/utils.svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';

	let volVisContainer: HTMLDivElement | null = $state(null);
	let volVisFocused = $state(false);
	let volVisSelectedIdx = $state(-1);

	function scrollToSelectedVolume() {
		if (volVisSelectedIdx < 0 || !volVisContainer) return;

		const volumeItems = volVisContainer.querySelectorAll('.volume-item');
		if (volumeItems[volVisSelectedIdx]) {
			volumeItems[volVisSelectedIdx].scrollIntoView({
				behavior: 'smooth',
				block: 'nearest'
			});
		}
	}

	function handleVolumeContainerKeyDown(event: KeyboardEvent) {
		if (!convState.chapterSizes.length) return;

		if (event.key === 'ArrowUp') {
			event.preventDefault();
			volVisSelectedIdx = Math.max(0, volVisSelectedIdx - 1);
			scrollToSelectedVolume();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			volVisSelectedIdx = Math.min(convState.chapterSizes.length - 1, volVisSelectedIdx + 1);
			scrollToSelectedVolume();
		} else if (event.key === 'Delete' && volVisSelectedIdx >= 0) {
			// Remove selected volume
			convState.chapterSizes = [
				...convState.chapterSizes.slice(0, volVisSelectedIdx),
				...convState.chapterSizes.slice(volVisSelectedIdx + 1)
			];

			// Adjust selected index after deletion
			if (volVisSelectedIdx >= convState.chapterSizes.length) {
				volVisSelectedIdx = Math.max(0, convState.chapterSizes.length - 1);
			}

			checkChapterLimits();
		}
	}
</script>

<div
	class="card flex h-full flex-col overflow-hidden"
	class:ring-2={volVisFocused}
	class:ring-primary={volVisFocused}
>
	<div
		class="card-body flex flex-col overflow-y-auto"
		tabindex="0"
		role="tab"
		bind:this={volVisContainer}
		onkeydown={handleVolumeContainerKeyDown}
		onfocus={() => (volVisFocused = true)}
		onblur={() => (volVisFocused = false)}
		use:handleKeyHint={{
			keys: [
				['arrowup', $t`Scroll up`],
				['arrowdown', $t`Scroll down`]
			]
		}}
	>
		<h3 class="mb-4 font-semibold">{$t`Volume Distribution`}</h3>

		{#if convState.chapterSizes.length > 0}
			<div class="flex-1 space-y-4">
				{#each convState.chapterSizes as chapters, i}
					<div
						class="volume-item border-background-tertiary dark:border-background-dark-tertiary rounded-lg border p-4"
					>
						<div class="mb-2 flex items-center justify-between">
							<span class="font-medium">{$t`Volume ${i + 1}`}</span>
							<span class="badge badge-secondary"
								>{chapters} {chapters === 1 ? $t`chapter` : $t`chapters`}</span
							>
						</div>

						<div class="mt-3 flex gap-1">
							<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
							{#each Array(chapters) as _, j}
								<div
									class="bg-primary h-4 flex-1 rounded-sm opacity-80 transition-opacity hover:opacity-100"
									style="--index: {j}; animation-delay: calc(var(--index) * 30ms);"
									title={$t`Chapter ${j + 1} of Volume ${i + 1}`}
								></div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div
				class="border-background-tertiary dark:border-background-dark-tertiary flex h-24 items-center justify-center rounded-lg border border-dashed p-4"
			>
				<p class="text-content-secondary dark:text-content-dark-secondary">
					{$t`No volumes detected. Try running the bundler again.`}
				</p>
			</div>
		{/if}

		{#if step4State.result && step4State.result.total_chapters > 0}
			<div
				class="border-background-tertiary dark:border-background-dark-tertiary mt-6 border-t pt-4"
			>
				<div class="mb-2 flex items-center justify-between">
					<span class="font-medium">{$t`Total Chapter Usage`}</span>
					<span class="">{getTotalChapters()}/{step4State.result.total_chapters}</span>
				</div>
				<div
					class="bg-background-tertiary dark:bg-background-dark-tertiary h-2 w-full overflow-hidden rounded-full"
				>
					<div
						class="h-full rounded-full transition-all duration-300 ease-out"
						class:bg-success={getTotalChapters() === step4State.result.total_chapters}
						class:bg-warning={getTotalChapters() < step4State.result.total_chapters}
						class:bg-error={getTotalChapters() > step4State.result.total_chapters}
						style="width: {getChaptersPercentage()}%"
					></div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	/* Ensure the scrollable container has smooth scrolling */
	.card-body.overflow-y-auto {
		scroll-behavior: smooth;
		-webkit-overflow-scrolling: touch;
	}

	/* Make sure the scroll container reacts to wheel events */
	.card-body.overflow-y-auto:focus {
		outline: none;
	}

	/* Ensure volume items have proper hover states */
	.volume-item {
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			border-color 0.15s ease;
	}

	.volume-item:hover:not(.selected) {
		border-color: var(--color-primary);
	}
</style>
