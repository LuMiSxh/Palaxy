<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { msg, t } from 'svelte-i18n-lingui';
	import convState from '$states/converter.svelte';
	import { commands, type ConvState } from '$types';
	import { truncatePath, wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import {
		IconBook,
		IconChartPie,
		IconCopy,
		IconPhoto,
		IconVocabulary,
		IconFolder,
		IconFileText,
		IconDirection,
		IconFolderPlus,
		IconPhoto as IconPhotoFormat,
	} from '@tabler/icons-svelte';

	// Waku Imports
	import { VStack, HStack, BentoGrid, BentoItem, Separator } from 'waku/layout';
	import { Label, Badge } from 'waku/components';

	let convStateData: ConvState | null = $state(null);
	let unregisterKeyHint: () => void;
	let totalImages = $state(0);
	let coverImages = $state(0);
	let maxVolumeImageCount = $state(0);
	let images: string[][] = $state([]);
	let scrollContainer: HTMLDivElement;

	// eslint-disable-next-line @typescript-eslint/no-unused-expressions
	msg`Left to Right`;
	// eslint-disable-next-line @typescript-eslint/no-unused-expressions
	msg`Right to Left`;

	// Function to calculate the total number of images in a volume
	function getVolumeImageCount(volumeIndex: number): number {
		if (!convStateData?.volume_sizes || !convStateData?.data) return 0;

		let chapterIndex = 0;
		for (let i = 0; i < volumeIndex; i++) {
			chapterIndex += convStateData.volume_sizes[i];
		}

		let count = 0;
		for (let i = 0; i < convStateData.volume_sizes[volumeIndex]; i++) {
			if (chapterIndex + i < images.length) {
				count += images[chapterIndex + i].length;
			}
		}

		return count;
	}

	// Calculate max volume image count for chart scaling
	function calculateMaxVolumeImageCount(): void {
		if (!convStateData?.volume_sizes) return;

		let max = 0;
		for (let i = 0; i < convStateData.volume_sizes.length; i++) {
			const count = getVolumeImageCount(i);
			if (count > max) max = count;
		}

		maxVolumeImageCount = max;
	}

	// Calculate total images and cover images
	function calculateImageStats() {
		if (!convStateData?.data) return;

		totalImages = images.reduce((sum: number, chapter: string[]) => sum + chapter.length, 0);
		coverImages = convStateData?.volume_sizes?.length || 0;
		calculateMaxVolumeImageCount();
	}

	onMount(async () => {
		unregisterKeyHint = keyHint.register([
			['arrowup', $t`Scroll up`],
			['arrowdown', $t`Scroll down`],
		]);

		const result = await wrapper(commands.convStateGet());
		if (result !== null && result.payload !== null) {
			convStateData = result.payload;
			images = convStateData.edited_data || convStateData.data;
			calculateImageStats();
		}

		if (scrollContainer) {
			scrollContainer.focus();
		}
	});

	onDestroy(() => {
		if (unregisterKeyHint) unregisterKeyHint();
	});
</script>

<div class="h-full w-full p-3">
	<div class="h-full overflow-y-auto" bind:this={scrollContainer} tabindex="0" role="tab">
		<VStack gap="md" class="mx-auto max-w-6xl pb-4">
			<BentoGrid cols={2} density="compact">
				<!-- General Information -->
				<BentoItem colspan={2} glass>
					<HStack align="center" gap="sm" class="text-muted mb-4">
						<IconCopy size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`General Information`}</span
						>
					</HStack>

					<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
						<!-- Project Name -->
						<VStack gap="xs">
							<Label text={$t`Project Name`} class="mb-0!" />
							<div class="bg-surface-2 rounded-lg px-3 py-2 font-medium opacity-75">
								{convStateData?.name || $t`Unnamed Project`}
							</div>
						</VStack>

						<!-- Source Path -->
						<VStack gap="xs">
							<Label text={$t`Source Path`} class="mb-0!" />
							<div
								class="bg-surface-2 truncate rounded-lg px-3 py-2 font-mono text-sm opacity-75"
								title={convStateData?.source || ''}
							>
								{truncatePath(convStateData?.source || $t`Not specified`)}
							</div>
						</VStack>

						<!-- File Type -->
						<VStack gap="xs">
							<Label text={$t`File Type`} class="mb-0!" />
							<div class="bg-surface-2 rounded-lg px-3 py-2 font-medium opacity-75">
								{convStateData?.format || $t`Not specified`}
							</div>
						</VStack>

						<!-- Target Location -->
						<VStack gap="xs">
							<Label text={$t`Target Location`} class="mb-0!" />
							<div
								class="bg-surface-2 truncate rounded-lg px-3 py-2 font-mono text-sm opacity-75"
								title={convState.target || ''}
							>
								{truncatePath(convState.target || $t`Not specified`)}
							</div>
						</VStack>

						<!-- Reading Direction -->
						<VStack gap="xs">
							<Label text={$t`Reading Direction`} class="mb-0!" />
							<HStack align="center" gap="sm">
								<div class="bg-surface-2 flex-1 rounded-lg px-3 py-2 opacity-75">
									{convStateData ? $t(convStateData.direction) : $t`Left to Right`}
								</div>
								{#if convStateData?.format !== 'EPUB'}
									<Badge variant="neutral">{$t`EPUB only`}</Badge>
								{/if}
							</HStack>
						</VStack>

						<!-- Create Folder -->
						<VStack gap="xs">
							<Label text={$t`Create New Folder`} class="mb-0!" />
							<div class="bg-surface-2 flex items-center gap-3 rounded-lg px-3 py-2 opacity-75">
								<div
									class="h-5 w-9 rounded-full transition-colors {convStateData?.create_directory
										? 'bg-accent-500'
										: 'bg-surface-0'}"
								>
									<div
										class="h-5 w-5 rounded-full bg-white shadow-sm transition-transform {convStateData?.create_directory
											? 'translate-x-4'
											: 'translate-x-0'}"
									></div>
								</div>
								<span class="text-sm">
									{convStateData?.create_directory
										? $t`Will create a new folder for output files`
										: $t`Will save directly in target location`}
								</span>
							</div>
						</VStack>

						<!-- Image Output Format -->
						<VStack gap="xs" class="md:col-span-2">
							<Label text={$t`Image Output Format`} class="mb-0!" />
							<div class="bg-surface-2 rounded-lg px-3 py-2">
								<div class="mb-1 font-medium opacity-75">
									{convStateData?.image_format ?? 'None'}
								</div>
								<span class="text-muted text-sm">
									{#if convStateData?.image_format === 'None'}
										{$t`Images will keep their original format`}
									{:else if convStateData?.image_format === 'WebP'}
										{$t`Images will be converted to WebP format`}
									{:else if convStateData?.image_format === 'AVIF'}
										{$t`Images will be converted to AVIF format (smaller, slower)`}
									{/if}
								</span>
							</div>
						</VStack>
					</div>
				</BentoItem>

				<!-- Statistics Overview -->
				<BentoItem colspan={2} glass>
					<HStack align="center" gap="sm" class="text-muted mb-4">
						<IconChartPie size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Conversion Summary`}</span>
					</HStack>

					<!-- Summary Cards -->
					<div class="mb-6 grid grid-cols-3 gap-4">
						<div
							class="bg-accent-500/10 border-accent-500/20 flex flex-col items-center rounded-lg border p-4"
						>
							<IconBook class="text-accent-500 mb-2" size={28} />
							<span class="text-2xl font-bold">{convStateData?.volume_sizes?.length ?? 0}</span>
							<span class="text-muted text-sm">{$t`Volumes`}</span>
						</div>
						<div
							class="bg-accent-500/10 border-accent-500/20 flex flex-col items-center rounded-lg border p-4"
						>
							<IconVocabulary class="text-accent-500 mb-2" size={28} />
							<span class="text-2xl font-bold">{convStateData?.data?.length ?? 0}</span>
							<span class="text-muted text-sm">{$t`Chapters`}</span>
						</div>
						<div
							class="bg-accent-500/10 border-accent-500/20 flex flex-col items-center rounded-lg border p-4"
						>
							<IconPhoto class="text-accent-500 mb-2" size={28} />
							<span class="text-2xl font-bold">{totalImages - (convState.excludedImages || 0)}</span
							>
							<span class="text-muted text-sm">{$t`Images`}</span>
						</div>
					</div>

					<Separator class="my-4!" />

					<!-- Image Statistics -->
					<div class="mb-6">
						<h4 class="mb-3 flex items-center font-medium">
							<IconPhoto class="text-accent-500 mr-2" size={18} />
							{$t`Image Statistics`}
						</h4>
						<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
							<div class="bg-surface-2 rounded-lg p-3 text-center">
								<div class="text-muted mb-1 text-xs">{$t`Total Images`}</div>
								<div class="text-lg font-semibold">{totalImages}</div>
							</div>
							<div class="bg-surface-2 rounded-lg p-3 text-center">
								<div class="text-muted mb-1 text-xs">{$t`Cover Images`}</div>
								<div class="text-lg font-semibold">{coverImages}</div>
							</div>
							<div class="bg-surface-2 rounded-lg p-3 text-center">
								<div class="text-muted mb-1 text-xs">{$t`Excluded`}</div>
								<div class="text-lg font-semibold">{convState.excludedImages || 0}</div>
							</div>
							<div class="bg-surface-2 rounded-lg p-3 text-center">
								<div class="text-muted mb-1 text-xs">{$t`New Images`}</div>
								<div class="text-lg font-semibold">{convState.newImages || 0}</div>
							</div>
						</div>
					</div>

					<!-- Volume Distribution Graph -->
					{#if convStateData?.volume_sizes && convStateData.volume_sizes.length > 0}
						<div>
							<h4 class="mb-3 flex items-center font-medium">
								<IconBook class="text-accent-500 mr-2" size={18} />
								{$t`Volume Distribution`}
							</h4>
							<div class="bg-surface-2 rounded-lg p-3">
								<div class="chart-container">
									{#each convStateData.volume_sizes as size, index}
										{@const imageCount = getVolumeImageCount(index)}
										{@const percentage =
											maxVolumeImageCount > 0 ? (imageCount / maxVolumeImageCount) * 100 : 0}
										<div class="chart-item">
											<div class="chart-label">{$t`Vol`} {index + 1}</div>
											<div class="chart-bar-container">
												<div
													class="chart-bar"
													style="width: {percentage}%"
													title="{imageCount} {$t`images`}, {size} {$t`chapters`}"
												>
													<!-- Chapter separators -->
													{#if images}
														{@const chapterStartIndex = convStateData.volume_sizes
															.slice(0, index)
															.reduce((sum, val) => sum + val, 0)}
														{@const volumeChapters = images.slice(
															chapterStartIndex,
															chapterStartIndex + size
														)}
														{@const chapterImageCounts = volumeChapters.map(
															(chapter) => chapter.length
														)}
														{@const totalVolumeImages = chapterImageCounts.reduce(
															(sum, val) => sum + val,
															0
														)}
														<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
														{#each chapterImageCounts as _, chapterIdx}
															{@const chapterEndPosition =
																(chapterImageCounts
																	.slice(0, chapterIdx + 1)
																	.reduce((sum, val) => sum + val, 0) /
																	totalVolumeImages) *
																100}
															{#if chapterIdx < chapterImageCounts.length - 1}
																<div
																	class="chapter-separator"
																	style="left: {chapterEndPosition}%"
																	title="{$t`Chapter`} {chapterStartIndex +
																		chapterIdx +
																		1} {$t`end`}"
																></div>
															{/if}
														{/each}
													{/if}
													<span class="chart-value">{imageCount}</span>
												</div>
											</div>
										</div>
									{/each}
								</div>
								<div class="text-muted mt-2 text-center text-xs">
									{$t`Images per volume`} | {$t`Hover for details`}
								</div>
							</div>
						</div>
					{/if}
				</BentoItem>
			</BentoGrid>
		</VStack>
	</div>
</div>

<style>
	/* Remove focus indicators while preserving accessibility */
	div[tabindex='0']:focus {
		outline: none;
		box-shadow: none;
		border-color: transparent;
	}

	/* Chart styles */
	.chart-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.chart-item {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.chart-label {
		width: 45px;
		text-align: right;
		font-size: 0.875rem;
	}

	.chart-bar-container {
		flex: 1;
		height: 24px;
		background-color: rgba(0, 0, 0, 0.1);
		border-radius: 4px;
		overflow: hidden;
	}

	.chart-bar {
		position: relative;
		height: 100%;
		background-color: var(--waku-accent, var(--color-primary));
		background-size: 1rem 1rem;
		min-width: 30px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding: 0 8px;
		border-radius: 3px;
		transition: width 0.5s ease-in-out;
	}

	.chapter-separator {
		position: absolute;
		height: 100%;
		width: 2px;
		background-color: rgba(255, 255, 255, 0.3);
		top: 0;
		z-index: 1;
	}

	.chart-value {
		color: white;
		font-size: 0.875rem;
		font-weight: 600;
		position: relative;
		z-index: 2;
	}
</style>
