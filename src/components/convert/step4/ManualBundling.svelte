<script lang="ts">
	import {
		checkChapterLimits,
		getChaptersPercentage,
		getTotalChapters,
		step4State
	} from '$components/convert/step4/utils.svelte';
	import convState from '$states/converter.svelte';
	import { IconChevronRight, IconFileZip, IconPlus, IconX } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { onMount, tick } from 'svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';

	let volVisSelectedIdx = $state(-1);
	let newVolumeChapters: number | undefined = $state(undefined);
	let volumeListContainer: HTMLDivElement | null = $state(null);
	let editingInput: HTMLInputElement | null = $state(null);
	let newVolumeInput: HTMLInputElement | null = $state(null);
	let isEditing = $state(false);

	function addVolume() {
		if (newVolumeChapters === undefined || newVolumeChapters <= 0) {
			return;
		}

		convState.chapterSizes = [...convState.chapterSizes, newVolumeChapters];
		newVolumeChapters = undefined;

		// Select the newly added volume
		volVisSelectedIdx = convState.chapterSizes.length - 1;
		newVolumeInput?.focus();

		checkChapterLimits();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			addVolume();
		}
	}

	function handleVolumeListKeyDown(event: KeyboardEvent) {
		if (!convState.chapterSizes.length) return;

		// Only handle navigation when not editing
		if (!isEditing) {
			if (event.key === 'ArrowUp') {
				event.preventDefault();
				volVisSelectedIdx = Math.max(0, volVisSelectedIdx - 1);
				scrollToSelectedVolume();
			} else if (event.key === 'ArrowDown') {
				event.preventDefault();
				volVisSelectedIdx = Math.min(convState.chapterSizes.length - 1, volVisSelectedIdx + 1);
				scrollToSelectedVolume();
			} else if (event.key === 'Delete' && volVisSelectedIdx >= 0) {
				deleteSelectedVolume();
			} else if (/^\d$/.test(event.key) && volVisSelectedIdx >= 0) {
				// Start editing on number press
				startEditing(event.key);
			}
		}
	}

	function startEditing(initialDigit?: string) {
		if (volVisSelectedIdx < 0) return;

		isEditing = true;

		// Set initial value based on first digit pressed
		if (initialDigit) {
			convState.chapterSizes[volVisSelectedIdx] = parseInt(initialDigit);
		}

		// Focus the input after the DOM updates
		tick().then(() => {
			if (editingInput) {
				editingInput.focus();
				editingInput.select();
			}
		});
	}

	function finishEditing() {
		isEditing = false;
		checkChapterLimits();
		volumeListContainer?.focus();
	}

	function deleteSelectedVolume() {
		if (volVisSelectedIdx < 0) return;

		convState.chapterSizes = [
			...convState.chapterSizes.slice(0, volVisSelectedIdx),
			...convState.chapterSizes.slice(volVisSelectedIdx + 1)
		];

		// Adjust selected index
		if (volVisSelectedIdx >= convState.chapterSizes.length) {
			volVisSelectedIdx = Math.max(0, convState.chapterSizes.length - 1);
		}

		checkChapterLimits();
	}

	function scrollToSelectedVolume() {
		if (volVisSelectedIdx < 0 || !volumeListContainer) return;

		const volumeItems = volumeListContainer.querySelectorAll('.volume-item');
		if (volumeItems[volVisSelectedIdx]) {
			volumeItems[volVisSelectedIdx].scrollIntoView({
				behavior: 'smooth',
				block: 'nearest'
			});
		}
	}

	// Prevent focus on mouse interactions but allow keyboard focus
	function preventFocusOnClick(event: MouseEvent) {
		event.preventDefault();
	}

	onMount(() => {
		// Initialize selection on mount if volumes exist
		if (convState.chapterSizes.length > 0 && volVisSelectedIdx < 0) {
			volVisSelectedIdx = 0;
		}

		// Focus input or list on mount
		setTimeout(() => {
			if (convState.chapterSizes.length > 0) {
				volumeListContainer?.focus();
			} else {
				newVolumeInput?.focus();
			}
		}, 100);
	});
</script>

<div class="grid h-full grid-cols-[1fr_1.5fr] gap-6 p-4">
	<!-- Left side: Stats panel -->
	<div class="flex flex-col gap-4">
		<div class="card">
			<div class="card-body">
				<h3 class="mb-4 font-semibold">{$t`Bundle Summary`}</h3>

				<div class="space-y-4">
					<div>
						<div class="mb-2 flex items-center justify-between">
							<span class="font-medium">{$t`Detected Chapters`}</span>
							<span class="badge badge-primary">{step4State.result?.total_chapters ?? 0}</span>
						</div>

						<div class="mb-2 flex items-center justify-between">
							<span class="font-medium">{$t`Used Chapters`}</span>
							<span
								class="badge"
								class:badge-success={getTotalChapters() ===
									(step4State.result?.total_chapters ?? 0)}
								class:badge-warning={getTotalChapters() < (step4State.result?.total_chapters ?? 0)}
								class:badge-error={getTotalChapters() > (step4State.result?.total_chapters ?? 0)}
							>
								{getTotalChapters()}
							</span>
						</div>

						<div
							class="bg-background-tertiary dark:bg-background-dark-tertiary h-2 w-full overflow-hidden rounded-full"
						>
							<div
								class="h-full rounded-full transition-all duration-300 ease-out"
								class:bg-success={getTotalChapters() === (step4State.result?.total_chapters ?? 0)}
								class:bg-warning={getTotalChapters() < (step4State.result?.total_chapters ?? 0)}
								class:bg-error={getTotalChapters() > (step4State.result?.total_chapters ?? 0)}
								style="width: {getChaptersPercentage()}%"
							></div>
						</div>
					</div>

					<div>
						<div class="mb-2 flex items-center justify-between">
							<span class="font-medium">{$t`Current Volumes`}</span>
							<span class="badge badge-primary">{convState.chapterSizes.length}</span>
						</div>
					</div>
				</div>
			</div>
		</div>

		{#if step4State.result && step4State.result.total_chapters > 0}
			<!-- god forgive me for this -->
			{@const chapterToVolumeMap = (() => {
				const map = new Array(step4State.result.total_chapters).fill(-1);
				let chapterCount = 0;
				for (let v = 0; v < convState.chapterSizes.length; v++) {
					for (let c = 0; c < convState.chapterSizes[v]; c++) {
						if (chapterCount < map.length) {
							map[chapterCount] = v;
							chapterCount++;
						}
					}
				}
				return map;
			})()}

			<div class="card">
				<div class="card-body">
					<h3 class="mb-4 font-semibold">{$t`Chapter Distribution`}</h3>
					<div class="flex flex-wrap gap-1">
						<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
						{#each Array(step4State.result.total_chapters) as _, i}
							{@const volumeIndex = chapterToVolumeMap[i]}
							{@const isUsed = volumeIndex !== -1}
							{@const isExceeded =
								getTotalChapters() > step4State.result.total_chapters &&
								i >= step4State.result.total_chapters}

							<div
								class="h-3 w-3 cursor-help rounded-sm transition-colors duration-200"
								class:bg-primary={isUsed && !isExceeded && volumeIndex % 2 === 0}
								class:bg-secondary={isUsed && !isExceeded && volumeIndex % 2 === 1}
								class:bg-error={isExceeded}
								class:bg-background-tertiary={!isUsed}
								class:dark:bg-background-dark-tertiary={!isUsed}
								title={$t`Chapter ${i + 1}${isUsed ? ' - ' + $t`Volume ${volumeIndex + 1}` : ''}`}
							></div>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Right side: Volume management -->
	<div class="card flex h-full flex-col">
		<div class="card-body flex flex-col overflow-y-auto">
			<h3 class="mb-4 font-semibold">{$t`Volume Management`}</h3>

			<!-- Volume list -->
			<div
				class="focus:ring-primary mb-4 flex-grow overflow-y-auto rounded !px-1 !py-1 focus:ring-2 focus:outline-none"
				tabindex="0"
				role="tablist"
				bind:this={volumeListContainer}
				onkeydown={handleVolumeListKeyDown}
				onmousedown={preventFocusOnClick}
				use:handleKeyHint={{
					keys: [
						['arrowup', $t`Navigate up`],
						['arrowdown', $t`Navigate down`],
						['delete', $t`Remove selected`]
					]
				}}
			>
				{#if convState.chapterSizes.length > 0}
					<div class="space-y-3">
						{#each convState.chapterSizes as chapters, i}
							{@const totalChapters = step4State.result?.total_chapters || 1}
							{@const previousChapters = convState.chapterSizes
								.slice(0, i)
								.reduce((sum, c) => sum + c, 0)}
							{@const previousPercentage = (previousChapters / totalChapters) * 100}
							{@const currentPercentage = (chapters / totalChapters) * 100}
							{@const totalPercentage = previousPercentage + currentPercentage}
							{@const isExceeded = totalPercentage > 100}

							<div
								class="volume-item flex items-center gap-3 rounded-lg border p-3 transition-colors duration-150"
								class:border-primary={volVisSelectedIdx === i}
								class:bg-background-secondary={volVisSelectedIdx === i}
								class:dark:bg-background-dark-secondary={volVisSelectedIdx === i}
								class:border-background-tertiary={volVisSelectedIdx !== i}
								class:dark:border-background-dark-tertiary={volVisSelectedIdx !== i}
								onclick={(evt) => {
									evt.preventDefault();
									volVisSelectedIdx = i;
								}}
								tabindex="-1"
								onkeydown={() => {}}
								role="button"
							>
								<div
									class="flex h-10 w-10 items-center justify-center rounded-full"
									class:bg-primary-light={i % 2 === 0}
									class:dark:bg-primary-dark-light={i % 2 === 0}
									class:bg-secondary-light={i % 2 === 1}
									class:dark:bg-secondary-dark-light={i % 2 === 1}
								>
									<IconFileZip class="text-primary" size={20} />
								</div>

								<div class="flex-1">
									<div class="flex items-center">
										<span class="font-medium">{$t`Volume` + ' ' + (i + 1)}</span>
										<IconChevronRight size={16} class="mx-1 opacity-60" />
										<span class="">{chapters} {chapters === 1 ? $t`chapter` : $t`chapters`}</span>
									</div>

									<div
										class="bg-background-tertiary dark:bg-background-dark-tertiary mt-1 h-1.5 w-full overflow-hidden rounded-full"
									>
										<div class="relative h-full w-full">
											<!-- Previous chapters (grey) -->
											<div
												class="absolute h-full rounded-full transition-all"
												class:bg-background-secondary={!isExceeded}
												class:dark:bg-background-dark-tertiary={!isExceeded}
												class:bg-error={isExceeded}
												style="width: {isExceeded
													? (previousChapters / (previousChapters + chapters)) * 100
													: previousPercentage}%"
											></div>

											<!-- Current chapter (blue) -->
											<div
												class="absolute h-full rounded-full transition-all"
												class:bg-primary={!isExceeded}
												class:bg-error={isExceeded}
												style="width: {isExceeded
													? (chapters / (previousChapters + chapters)) * 100
													: currentPercentage}%;
												 left: {isExceeded
													? (previousChapters / (previousChapters + chapters)) * 100
													: previousPercentage}%"
											></div>
										</div>
									</div>
								</div>

								<div class="flex items-center gap-2">
									{#if isEditing && volVisSelectedIdx === i}
										<!-- Editing mode -->
										<input
											bind:this={editingInput}
											class="input h-9 w-16 px-2 text-center"
											type="number"
											min="1"
											max="999"
											bind:value={convState.chapterSizes[i]}
											onblur={finishEditing}
											tabindex="-1"
											onkeydown={(e) => {
												if (e.key === 'Enter' || e.key === 'Escape') {
													finishEditing();
													e.preventDefault();
												}
											}}
										/>
									{:else}
										<!-- Display mode -->
										<button
											class="input flex h-9 w-16 items-center justify-center px-2 text-center"
											onclick={(e) => {
												e.stopPropagation();
												volVisSelectedIdx = i;
												startEditing();
											}}
											tabindex="-1"
										>
											{chapters}
										</button>
									{/if}

									<button
										class="btn btn-soft-lighter h-9 w-12 p-0"
										onclick={(e) => {
											e.stopPropagation();
											volVisSelectedIdx = i;
											deleteSelectedVolume();
										}}
										tabindex="-1"
									>
										<IconX class="stroke-error" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div
						class="border-background-tertiary dark:border-background-dark-tertiary flex h-24 items-center justify-center rounded-lg border border-dashed p-4"
					>
						<p class="text-content-secondary dark:text-content-dark-secondary">
							{$t`No volumes added yet. Add your first volume below.`}
						</p>
					</div>
				{/if}
			</div>

			<!-- Add new volume -->
			<div class="mt-auto">
				<div class="flex-1">
					<label for="new-volume" class="mb-2 block font-medium">{$t`Add new volume`}</label>
					<div class="flex gap-3">
						<input
							id="new-volume"
							class="input flex-1"
							type="number"
							min="1"
							max="999"
							placeholder={$t`Chapters in volume`}
							bind:value={newVolumeChapters}
							bind:this={newVolumeInput}
							onkeydown={handleKeyDown}
							use:handleKeyHint={{ keys: [['enter', $t`Add Volume`]] }}
						/>

						<button
							class="btn btn-primary"
							onclick={addVolume}
							disabled={newVolumeChapters === undefined || newVolumeChapters <= 0}
						>
							<IconPlus size={18} class="mr-1" />
							{$t`Add Volume`}
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	/* Improved volume item styling */
	.volume-item {
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.volume-item:hover:not([class*='border-primary']) {
		border-color: var(--color-primary-hover);
	}

	/* Smooth scrolling for the volume list */
	.overflow-y-auto {
		scroll-behavior: smooth;
	}
</style>
