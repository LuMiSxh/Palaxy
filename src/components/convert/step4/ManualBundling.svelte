<script lang="ts">
	import {
		checkChapterLimits,
		getChaptersPercentage,
		getTotalChapters,
		step4State,
	} from '$components/convert/step4/utils.svelte';
	import convState from '$states/converter.svelte';
	import {
		IconFileZip,
		IconPlus,
		IconX,
		IconChevronRight,
		IconChartBar,
		IconListNumbers,
	} from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { onMount, tick } from 'svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Badge, Button, Input } from 'waku/components';

	let volVisSelectedIdx = $state(-1);
	let newVolumeChapters: string | undefined = $state(undefined);
	let volumeListContainer: HTMLDivElement | null = $state(null);
	let editingInput: HTMLInputElement | null = $state(null);
	let isEditing = $state(false);

	function addVolume() {
		const chapters = Number(newVolumeChapters);
		if (newVolumeChapters === undefined || chapters <= 0 || isNaN(chapters)) {
			return;
		}

		convState.chapterSizes = [...convState.chapterSizes, chapters];
		newVolumeChapters = undefined;

		volVisSelectedIdx = convState.chapterSizes.length - 1;

		checkChapterLimits();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			addVolume();
		}
	}

	function handleVolumeListKeyDown(event: KeyboardEvent) {
		if (!convState.chapterSizes.length) return;

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
				startEditing(event.key);
			}
		}
	}

	function startEditing(initialDigit?: string) {
		if (volVisSelectedIdx < 0) return;

		isEditing = true;

		if (initialDigit) {
			convState.chapterSizes[volVisSelectedIdx] = parseInt(initialDigit);
		}

		void tick().then(() => {
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

	function preventFocusOnClick(event: MouseEvent) {
		event.preventDefault();
	}

	onMount(() => {
		if (convState.chapterSizes.length > 0 && volVisSelectedIdx < 0) {
			volVisSelectedIdx = 0;
		}

		setTimeout(() => {
			volumeListContainer?.focus();
		}, 100);
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={3} density="comfortable" rows="auto 1fr auto" class="h-full">
		<!-- Summary Cards Row -->
		<BentoItem glass padding="sm" class="max-h-24">
			<HStack gap="md" align="center" justify="between">
				<HStack gap="md" align="center">
					<div
						class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconFileZip size={20} class="text-accent-500" />
					</div>
					<VStack gap="none">
						<span class="text-muted text-xs font-medium tracking-wide uppercase"
							>{$t`Detected`}</span
						>
						<div class="text-2xl leading-tight font-bold">
							{step4State.result?.total_chapters ?? 0}
						</div>
					</VStack>
				</HStack>
			</HStack>
		</BentoItem>

		<BentoItem glass padding="sm" class="max-h-24">
			<HStack gap="md" align="center" justify="between">
				<HStack gap="md" align="center">
					{@const isEqual = getTotalChapters() === (step4State.result?.total_chapters ?? 0)}
					{@const isLess = getTotalChapters() < (step4State.result?.total_chapters ?? 0)}
					<div
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {isEqual
							? 'bg-success/10'
							: isLess
								? 'bg-warning/10'
								: 'bg-danger/10'}"
					>
						<IconListNumbers
							size={20}
							class={isEqual ? 'text-success' : isLess ? 'text-warning' : 'text-danger'}
						/>
					</div>
					<VStack gap="none">
						<span class="text-muted text-xs font-medium tracking-wide uppercase">{$t`Used`}</span>
						<div
							class="text-2xl leading-tight font-bold {isEqual
								? 'text-success'
								: isLess
									? 'text-warning'
									: 'text-danger'}"
						>
							{getTotalChapters()}
						</div>
					</VStack>
				</HStack>
			</HStack>
		</BentoItem>

		<BentoItem glass padding="sm" class="max-h-24">
			<HStack gap="md" align="center" justify="between">
				<HStack gap="md" align="center">
					<div
						class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconChartBar size={20} class="text-accent-500" />
					</div>
					<VStack gap="none">
						<span class="text-muted text-xs font-medium tracking-wide uppercase">{$t`Volumes`}</span
						>
						<div class="text-2xl leading-tight font-bold">
							{convState.chapterSizes.length}
						</div>
					</VStack>
				</HStack>
			</HStack>
		</BentoItem>

		<!-- Main Content Area -->
		<BentoItem colspan={2} glass class="flex min-h-0 flex-col">
			<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
				<IconFileZip size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Management`}</span>
				<Badge variant="primary" class="ml-auto">{convState.chapterSizes.length}</Badge>
			</HStack>

			<div
				class="custom-scrollbar mb-3 flex-1 overflow-y-auto rounded px-1 py-1"
				tabindex="-1"
				role="tablist"
				bind:this={volumeListContainer}
				onkeydown={handleVolumeListKeyDown}
				onmousedown={preventFocusOnClick}
			>
				{#if convState.chapterSizes.length > 0}
					<VStack gap="sm">
						{#each convState.chapterSizes as chapters, i}
							{@const totalChapters = step4State.result?.total_chapters || 1}
							{@const currentPercentage = (chapters / totalChapters) * 100}

							<button
								class="volume-item bg-surface-2 hover:bg-surface-1 group w-full rounded-lg p-3 text-left transition-all duration-150"
								class:ring-2={volVisSelectedIdx === i}
								class:ring-accent-500={volVisSelectedIdx === i}
								class:bg-surface-1={volVisSelectedIdx === i}
								onclick={(evt) => {
									evt.preventDefault();
									volVisSelectedIdx = i;
								}}
								tabindex="-1"
								type="button"
							>
								<HStack gap="sm" align="center">
									<div
										class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {i %
											2 ===
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

										<div class="bg-surface-0 h-1.5 w-full overflow-hidden rounded-full">
											<div
												class="bg-accent-500 h-full rounded-full transition-all"
												style="width: {Math.min(currentPercentage, 100)}%"
											></div>
										</div>
									</VStack>

									<HStack gap="xs" align="center">
										{#if isEditing && volVisSelectedIdx === i}
											<input
												bind:this={editingInput}
												type="number"
												min="1"
												max="999"
												class="bg-surface-0 border-accent-500 h-9 w-16 rounded border text-center focus:outline-none"
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
											<button
												class="bg-surface-0 hover:bg-accent-500/10 hover:border-accent-500 flex h-9 w-16 items-center justify-center rounded border border-transparent text-center font-medium transition-colors"
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
											class="hover:bg-danger/10 border-danger/20 text-danger hover:border-danger flex h-9 w-9 items-center justify-center rounded border transition-colors"
											onclick={(e) => {
												e.stopPropagation();
												volVisSelectedIdx = i;
												deleteSelectedVolume();
											}}
											tabindex="-1"
										>
											<IconX size={18} />
										</button>
									</HStack>
								</HStack>
							</button>
						{/each}
					</VStack>
				{:else}
					<div
						class="border-waku-border/50 flex h-full items-center justify-center rounded-lg border border-dashed p-8"
					>
						<VStack gap="sm" align="center" class="text-center">
							<div class="bg-surface-2 flex h-16 w-16 items-center justify-center rounded-full">
								<IconFileZip size={32} class="text-muted" />
							</div>
							<p class="text-muted max-w-xs text-sm">
								{$t`No volumes added yet. Add your first volume using the panel on the right.`}
							</p>
						</VStack>
					</div>
				{/if}
			</div>
		</BentoItem>

		<!-- Side Panel: Chapter Distribution -->
		<BentoItem glass class="flex flex-col" onclick={() => {}}>
			<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
				<IconChartBar size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Distribution`}</span>
			</HStack>

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

				<VStack gap="md" class="flex-1">
					<!-- Progress Bar -->
					<div>
						<HStack justify="between" align="center" class="mb-2">
							<span class="text-sm font-medium">{$t`Usage`}</span>
							<span class="font-mono text-sm">
								{getTotalChapters()}/{step4State.result.total_chapters}
							</span>
						</HStack>
						<div class="bg-surface-0 h-2 w-full overflow-hidden rounded-full">
							<div
								class="h-full rounded-full transition-all duration-300"
								class:bg-success={getTotalChapters() === step4State.result.total_chapters}
								class:bg-warning={getTotalChapters() < step4State.result.total_chapters}
								class:bg-danger={getTotalChapters() > step4State.result.total_chapters}
								style="width: {getChaptersPercentage()}%"
							></div>
						</div>
					</div>

					<!-- Visual Distribution -->
					<div>
						<span class="text-muted mb-2 block text-xs">{$t`Chapter Map`}</span>
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
				</VStack>
			{:else}
				<div class="flex flex-1 items-center justify-center">
					<p class="text-muted text-center text-sm">{$t`No chapter data available`}</p>
				</div>
			{/if}
		</BentoItem>

		<!-- Add Volume Section -->
		<BentoItem
			colspan={3}
			glass
			padding="sm"
			onclick={addVolume}
			data-keyhint={`enter;${$t`Add Volume`}`}
			disabled={!newVolumeChapters ||
				Number(newVolumeChapters) <= 0 ||
				isNaN(Number(newVolumeChapters))}
		>
			<HStack gap="sm" align="center">
				<div
					class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
				>
					<IconPlus size={20} class="text-accent-500" />
				</div>
				<VStack gap="xs" class="flex-1">
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Add New Volume`}</span>
					<HStack gap="sm">
						<Input
							id="new-volume"
							type="number"
							style="seamless"
							placeholder={$t`Number of chapters` || ''}
							bind:value={newVolumeChapters}
							onkeydown={(e) => {
								if (e.key === ' ') {
									e.stopPropagation();
								}
								handleKeyDown(e);
							}}
							class="flex-1"
						/>
						<Button variant="primary" onclick={addVolume} style="seamless">
							<HStack gap="sm" align="center">
								<IconPlus size={18} />
								<span>{$t`Add`}</span>
							</HStack>
						</Button>
					</HStack>
				</VStack>
			</HStack>
		</BentoItem>
	</BentoGrid>
</div>

<style>
	.volume-item {
		cursor: pointer;
	}

	.custom-scrollbar {
		scrollbar-width: thin;
		scrollbar-color: var(--waku-surface-2) transparent;
	}

	.custom-scrollbar::-webkit-scrollbar {
		width: 8px;
	}

	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb {
		background-color: var(--waku-surface-2);
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background-color: var(--waku-surface-1);
	}
</style>
