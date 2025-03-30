<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { msg, t } from 'svelte-i18n-lingui';
	import convState, { stepState } from '$states/converter.svelte';
	import { commands, type ConvState } from '$types';
	import { truncatePath, wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import {
		IconBook,
		IconChartPie,
		IconCopy,
		IconPhoto,
		IconVocabulary
	} from '@tabler/icons-svelte';

	let convStateData: ConvState | null = $state(null);
	let unregisterKeyHint: () => void;
	let totalImages = $state(0);
	let coverImages = $state(0);
	let maxVolumeImageCount = $state(0);
	let images: string[][] = $state([]);
	// Add at the top of your script section
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

		// Covers are typically one per volume
		coverImages = convStateData?.volume_sizes?.length || 0;

		calculateMaxVolumeImageCount();
	}

	onMount(async () => {
		unregisterKeyHint = keyHint.smartAdd([
			['arrowup', $t`Scroll up`],
			['arrowdown', $t`Scroll down`]
		]);

		stepState.disablePrev = false;
		stepState.disableNext = false;

		// Load state data
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

<div
	class="flex h-full w-full flex-col overflow-y-auto p-4 focus:outline-none"
	style="max-height: calc(100vh - 8rem)"
	bind:this={scrollContainer}
	tabindex="0"
	role="tab"
>
	<div class="flex flex-col gap-4">
		<!-- General Information -->
		<div class="card bg-background-secondary dark:bg-background-dark-secondary h-fit">
			<div class="card-header">
				<div class="flex items-center">
					<IconCopy class="text-primary mr-2" size={20} />
					<h3 class="text-lg font-semibold">{$t`General Information`}</h3>
				</div>
			</div>
			<div class="card-body">
				<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
					<!-- Project Name -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`Project Name`}</label>
						<input
							type="text"
							class="input input-primary bg-opacity-50"
							value={convStateData?.name || $t`Unnamed Project`}
							readonly
							disabled
						/>
					</div>

					<!-- Bundle Type -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`Source Path`}</label>
						<input
							type="text"
							class="input input-primary bg-opacity-50 font-mono text-sm"
							value={truncatePath(convStateData?.source || $t`Not specified`)}
							readonly
							disabled
							title={convStateData?.source || ''}
						/>
					</div>

					<!-- File Type -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`File Type`}</label>
						<input
							type="text"
							class="input input-primary bg-opacity-50"
							value={convStateData?.format || $t`Not specified`}
							readonly
							disabled
						/>
					</div>

					<!-- Target Location -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`Target Location`}</label>
						<input
							type="text"
							class="input input-primary bg-opacity-50 font-mono text-sm"
							value={truncatePath(convState.target || $t`Not specified`)}
							readonly
							disabled
							title={convState.target || ''}
						/>
					</div>

					<!-- Reading Direction -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`Reading Direction`}</label>
						<div class="flex items-center">
							<input
								type="text"
								class="input input-primary bg-opacity-50 max-w-2/3"
								value={convStateData ? $t(convStateData.direction) : $t`Left to Right`}
								readonly
								disabled
							/>
							{#if convStateData?.format !== 'EPUB'}
								<span class="badge badge-secondary ml-2">{$t`EPUB only`}</span>
							{/if}
						</div>
					</div>

					<!-- Create Folder -->
					<div>
						<label for="" class="mb-2 block font-medium">{$t`Create New Folder`}</label>
						<div class="flex items-center gap-4">
							<label class="toggle toggle-lg pointer-events-none cursor-not-allowed opacity-75">
								<input
									type="checkbox"
									class="toggle-input"
									checked={convStateData?.create_directory}
									disabled
								/>
								<span class="toggle-track">
									<span class="toggle-thumb"></span>
								</span>
							</label>
							<span class="text-sm">
								{convStateData?.create_directory
									? $t`Will create a new folder for output files`
									: $t`Will save directly in target location`}
							</span>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Combined Statistics and Summary Card -->
		<div class="card bg-background-secondary dark:bg-background-dark-tertiary h-fit">
			<div class="card-header">
				<div class="flex items-center">
					<IconChartPie class="text-primary mr-2" size={20} />
					<h3 class="text-lg font-semibold">{$t`Conversion Summary`}</h3>
				</div>
			</div>
			<div class="card-body">
				<!-- Summary Overview with Graphics -->
				<div class="mb-6 flex flex-col items-center">
					<div class="mb-4 grid w-full grid-cols-3 gap-4 text-center">
						<div class="bg-primary/20 flex flex-col items-center rounded-lg p-3">
							<IconBook class="text-primary mb-2" size={24} />
							<span class="text-xl font-bold">{convStateData?.volume_sizes?.length ?? 0}</span>
							<span class="text-sm">{$t`Volumes`}</span>
						</div>
						<div class="bg-secondary/20 flex flex-col items-center rounded-lg p-3">
							<IconVocabulary class="text-primary mb-2" size={24} />
							<span class="text-xl font-bold">{convStateData?.data?.length ?? 0}</span>
							<span class="text-sm">{$t`Chapters`}</span>
						</div>
						<div class="bg-primary/20 flex flex-col items-center rounded-lg p-3">
							<IconPhoto class="text-primary mb-2" size={24} />
							<span class="text-xl font-bold">{totalImages - (convState.excludedImages || 0)}</span>
							<span class="text-sm">{$t`Images`}</span>
						</div>
					</div>
				</div>

				<!-- Image Statistics Section -->
				<div class="mb-6">
					<h4 class="mb-3 flex items-center font-medium">
						<IconPhoto class="text-primary mr-2" size={18} />
						{$t`Image Statistics`}
					</h4>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
						<div
							class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-3 text-center"
						>
							<div class="text-primary mb-1 text-sm">{$t`Total Images`}</div>
							<div class="text-lg font-semibold">{totalImages}</div>
						</div>
						<div
							class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-3 text-center"
						>
							<div class="text-primary mb-1 text-sm">{$t`Cover Images`}</div>
							<div class="text-lg font-semibold">{coverImages}</div>
						</div>
						<div
							class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-3 text-center"
						>
							<div class="dark:text-secondary text-secondary-dark-light mb-1 text-sm">
								{$t`Excluded`}
							</div>
							<div class="text-lg font-semibold">{convState.excludedImages || 0}</div>
						</div>
						<div
							class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-3 text-center"
						>
							<div class="dark:text-secondary text-secondary-dark-light mb-1 text-sm">
								{$t`New Images`}
							</div>
							<div class="text-lg font-semibold">{convState.newImages || 0}</div>
						</div>
					</div>
				</div>

				<!-- Volume Distribution Graph -->
				{#if convStateData?.volume_sizes && convStateData.volume_sizes.length > 0}
					<div class="mb-6">
						<h4 class="mb-3 flex items-center font-medium">
							<IconBook class="text-primary mr-2" size={18} />
							{$t`Volume Distribution`}
						</h4>
						<div class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-4">
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
																title="{$t`Chapter`} {chapterStartIndex + chapterIdx + 1} {$t`end`}"
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
							<div
								class="text-content-secondary dark:text-content-dark-secondary mt-2 text-center text-xs"
							>
								{$t`Images per volume`} | {$t`Hover for details`}
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	/* Smooth scrolling */
	.overflow-y-auto {
		scroll-behavior: smooth;
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
		height: 100%;
		background-color: var(--color-primary);
		background-size: 1rem 1rem;
		min-width: 30px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding: 0 8px;
		border-radius: 3px;
		transition: width 0.5s ease-in-out;
	}

	.chart-bar {
		position: relative;
		height: 100%;
		background-color: var(--color-primary);
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
		background-color: var(--color-secondary);
		top: 0;
		z-index: 1;
	}

	.chart-value {
		color: white;
		font-size: 0.875rem;
		font-weight: 600;
	}
</style>
