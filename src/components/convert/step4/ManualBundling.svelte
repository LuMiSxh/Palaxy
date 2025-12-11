<script lang="ts">
	import {
		checkChapterLimits,
		getChaptersPercentage,
		getTotalChapters,
		step4State,
	} from '$components/convert/step4/utils.svelte';
	import convState from '$states/converter.svelte';
	import { IconFileZip, IconPlus, IconX, IconChevronRight } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { onMount, tick } from 'svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { VStack, HStack } from 'waku/layout';
	import { Badge, Label } from 'waku/components';

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
			...convState.chapterSizes.slice(volVisSelectedIdx + 1),
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
				block: 'nearest',
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

<div class="flex h-full w-full gap-4 overflow-hidden p-3 pb-3">
	<!-- Left side: Stats panel -->
	<VStack gap="sm" class="w-80 shrink-0">
		<!-- Bundle Summary -->
		<div class="glass-subtle rounded-xl p-4">
			<h3 class="mb-3 text-base font-semibold">{$t`Bundle Summary`}</h3>

			<VStack gap="sm">
				<HStack justify="between" align="center">
					<span class="text-sm">{$t`Detected Chapters`}</span>
					<Badge variant="primary">{step4State.result?.total_chapters ?? 0}</Badge>
				</HStack>

				<HStack justify="between" align="center">
					<span class="text-sm">{$t`Used Chapters`}</span>
					<Badge
						variant={getTotalChapters() === (step4State.result?.total_chapters ?? 0)
							? 'success'
							: getTotalChapters() < (step4State.result?.total_chapters ?? 0)
								? 'warning'
								: 'danger'}
					>
						{getTotalChapters()}
					</Badge>
				</HStack>

				<div>
					<div class="bg-surface-1 h-2 w-full overflow-hidden rounded-full">
						<div
							class="h-full rounded-full transition-all duration-300 ease-out"
							class:bg-success={getTotalChapters() === (step4State.result?.total_chapters ?? 0)}
							class:bg-warning={getTotalChapters() < (step4State.result?.total_chapters ?? 0)}
							class:bg-danger={getTotalChapters() > (step4State.result?.total_chapters ?? 0)}
							style="width: {getChaptersPercentage()}%"
						></div>
					</div>
				</div>

				<HStack justify="between" align="center">
					<span class="text-sm">{$t`Current Volumes`}</span>
					<Badge variant="primary">{convState.chapterSizes.length}</Badge>
				</HStack>
			</VStack>
		</div>

		<!-- Chapter Distribution Visualization -->
		{#if step4State.result && step4State.result.total_chapters > 0}
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

			<div class="glass-subtle rounded-xl p-4">
				<h3 class="mb-3 text-base font-semibold">{$t`Chapter Distribution`}</h3>
				<div class="flex flex-wrap gap-1">
					{#each Array(step4State.result.total_chapters) as _, i}
						{@const volumeIndex = chapterToVolumeMap[i]}
						{@const isUsed = volumeIndex !== -1}
						{@const isExceeded =
							getTotalChapters() > step4State.result.total_chapters &&
							i >= step4State.result.total_chapters}

						<div
							class="h-3 w-3 cursor-help rounded-sm transition-colors duration-200"
							class:bg-accent-500={isUsed && !isExceeded && volumeIndex % 2 === 0}
							class:bg-accent-400={isUsed && !isExceeded && volumeIndex % 2 === 1}
							class:bg-danger={isExceeded}
							class:bg-surface-1={!isUsed}
							title={$t({ message: 'Chapter {chap}', values: { chap: i + 1 } }) +
								(isUsed
									? ' - ' + $t({ message: 'Volume {vol}', values: { vol: volumeIndex + 1 } })
									: '')}
						></div>
					{/each}
				</div>
			</div>
		{/if}
	</VStack>

	<!-- Right side: Volume management -->
	<div class="glass-subtle flex h-full flex-1 flex-col overflow-hidden rounded-xl p-4">
		<h3 class="mb-3 text-base font-semibold">{$t`Volume Management`}</h3>

		<!-- Volume list -->
		<div
			class="focus:ring-accent-500 mb-3 flex-1 overflow-y-auto rounded px-1 py-1 focus:ring-2 focus:outline-none"
			tabindex="0"
			role="tablist"
			bind:this={volumeListContainer}
			onkeydown={handleVolumeListKeyDown}
			onmousedown={preventFocusOnClick}
			use:handleKeyHint={{
				keys: [
					['arrowup', $t`Navigate up`],
					['arrowdown', $t`Navigate down`],
					['delete', $t`Remove selected`],
				],
			}}
		>
			{#if convState.chapterSizes.length > 0}
				<VStack gap="sm">
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
							class="volume-item glass-subtle group rounded-lg p-3 transition-all duration-150"
							class:ring-2={volVisSelectedIdx === i}
							class:ring-accent-500={volVisSelectedIdx === i}
							onclick={(evt) => {
								evt.preventDefault();
								volVisSelectedIdx = i;
							}}
							tabindex="-1"
							onkeydown={() => {}}
							role="button"
						>
							<HStack gap="sm" align="center">
								<div
									class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {i % 2 ===
									0
										? 'bg-accent-500/20'
										: 'bg-accent-400/20'}"
								>
									<IconFileZip size={20} class="text-accent-500" />
								</div>

								<VStack gap="xs" class="flex-1 overflow-hidden">
									<HStack gap="xs" align="center" class="text-sm">
										<span class="font-medium">{$t`Volume`} {i + 1}</span>
										<IconChevronRight size={14} class="text-muted" />
										<span class="text-muted"
											>{chapters} {chapters === 1 ? $t`chapter` : $t`chapters`}</span
										>
									</HStack>

									<div class="bg-surface-2 h-1.5 w-full overflow-hidden rounded-full">
										<div class="relative h-full w-full">
											<!-- Previous chapters (grey) -->
											<div
												class="absolute h-full rounded-full transition-all"
												class:bg-surface-1={!isExceeded}
												class:bg-danger={isExceeded}
												style="width: {isExceeded
													? (previousChapters / (previousChapters + chapters)) * 100
													: previousPercentage}%"
											></div>

											<!-- Current chapter (blue) -->
											<div
												class="absolute h-full rounded-full transition-all"
												class:bg-accent-500={!isExceeded}
												class:bg-danger={isExceeded}
												style="width: {isExceeded
													? (chapters / (previousChapters + chapters)) * 100
													: currentPercentage}%;
												 left: {isExceeded
													? (previousChapters / (previousChapters + chapters)) * 100
													: previousPercentage}%"
											></div>
										</div>
									</div>
								</VStack>

								<HStack gap="xs" align="center">
									{#if isEditing && volVisSelectedIdx === i}
										<!-- Editing mode -->
										<input
											bind:this={editingInput}
											type="number"
											min="1"
											max="999"
											class="input h-9 w-16 rounded text-center"
											bind:value={convState.chapterSizes[i]}
											onblur={finishEditing}
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
											class="bg-surface-2 hover:bg-surface-1 flex h-9 w-16 items-center justify-center rounded-lg text-center font-medium transition-colors"
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
										class="btn btn-ghost text-danger hover:bg-danger/10 h-9 w-9 p-0"
										onclick={(e) => {
											e.stopPropagation();
											volVisSelectedIdx = i;
											deleteSelectedVolume();
										}}
									>
										<IconX size={18} />
									</button>
								</HStack>
							</HStack>
						</div>
					{/each}
				</VStack>
			{:else}
				<div
					class="border-waku-border/50 flex h-24 items-center justify-center rounded-lg border border-dashed p-4"
				>
					<p class="text-muted text-center text-sm">
						{$t`No volumes added yet. Add your first volume below.`}
					</p>
				</div>
			{/if}
		</div>

		<!-- Add new volume -->
		<div class="bg-surface-2 rounded-lg p-3">
			<Label text={$t`Add new volume`} for="new-volume" class="mb-2!" />
			<HStack gap="sm">
				<input
					id="new-volume"
					type="number"
					min="1"
					max="999"
					class="input flex-1 rounded pl-2"
					placeholder={$t`Chapters in volume` || ''}
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
					<IconPlus size={18} />
					{$t`Add Volume`}
				</button>
			</HStack>
		</div>
	</div>
</div>

<style>
	/* Improved volume item styling */
	.volume-item {
		cursor: pointer;
	}

	/* Smooth scrolling for the volume list */
	.overflow-y-auto {
		scroll-behavior: smooth;
	}
</style>
