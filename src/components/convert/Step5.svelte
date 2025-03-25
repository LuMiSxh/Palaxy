<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { IconBook, IconNotes, IconPhoto, IconX } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { stepState } from '$states/converter.svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';

	let images: string[][] = $state([]);
	let selectedImages: (string | null)[][] = $state([]);
	let isVisibleState: boolean[][] = $state([]); // true = visible (included), false = hidden (excluded)
	let coverState: boolean[][] = $state([]); // Track cover images
	let imageLoadErrorState: boolean[][] = $state([]); // Track image loading errors
	let unregisterKeyHint: () => void;

	// Volume-specific data
	let volumeSizes: number[] = $state([]);
	let volumeChapters: number[][] = $state([]); // Tracks which chapters belong to which volume
	let volumeLoadingState: { loadedImages: number; totalImages: number }[] = $state([]);

	function toggleOverlay(volumeIndex: number, chapterIndex: number, imageIndex: number) {
		// Toggle visibility state
		isVisibleState[chapterIndex][imageIndex] = !isVisibleState[chapterIndex][imageIndex];

		// Toggle selection (null = excluded)
		selectedImages[chapterIndex][imageIndex] = isVisibleState[chapterIndex][imageIndex]
			? images[chapterIndex][imageIndex]
			: null;

		// Always update cover states when toggling images
		updateCoverStates();
	}

	function updateCoverStates() {
		// Reset all cover states
		coverState = coverState.map((row) => row.map(() => false));

		// Assign cover images for each volume
		if (volumeChapters.length > 0) {
			// For each volume
			for (let volumeIndex = 0; volumeIndex < volumeChapters.length; volumeIndex++) {
				const chaptersInVolume = volumeChapters[volumeIndex];
				let coverFound = false;

				// Look through each chapter in this volume for a visible image to use as cover
				for (const chapterIndex of chaptersInVolume) {
					if (coverFound) break;

					// Look at each image in this chapter
					for (let imageIndex = 0; imageIndex < images[chapterIndex].length; imageIndex++) {
						// If this image is visible (included), mark it as the cover and stop looking
						if (isVisibleState[chapterIndex][imageIndex]) {
							coverState[chapterIndex][imageIndex] = true;
							coverFound = true;
							break;
						}
					}
				}
			}
		}
	}

	// Track image loading for a specific volume
	function handleImageLoaded(volumeIndex: number) {
		if (volumeIndex >= 0 && volumeIndex < volumeLoadingState.length) {
			volumeLoadingState[volumeIndex].loadedImages++;

			// Check if this volume is fully loaded
			const volumeState = volumeLoadingState[volumeIndex];
			if (volumeState.loadedImages === volumeState.totalImages) {
				// This volume is fully loaded
				tick();
			}
		}
	}

	// Handle image loading errors
	function handleImageError(volumeIndex: number, chapterIndex: number, imageIndex: number) {
		// Mark this image as having an error
		imageLoadErrorState[chapterIndex][imageIndex] = true;

		// Still count it as "loaded" to prevent the loading indicator from getting stuck
		handleImageLoaded(volumeIndex);
	}

	// Handle keyboard navigation
	function handleKeyboardNavigation(
		event: KeyboardEvent,
		volumeIndex: number,
		chapterIndex: number,
		imageIndex: number
	) {
		if (event.key === 'Enter') {
			toggleOverlay(volumeIndex, chapterIndex, imageIndex);
			event.preventDefault();
		}
	}

	// Check if all images in a volume are loaded
	function isVolumeLoaded(volumeIndex: number): boolean {
		if (volumeIndex >= 0 && volumeIndex < volumeLoadingState.length) {
			const state = volumeLoadingState[volumeIndex];
			return state.loadedImages === state.totalImages;
		}
		return false;
	}

	// Get loading progress percentage for a volume
	function getVolumeLoadingProgress(volumeIndex: number): number {
		if (volumeIndex >= 0 && volumeIndex < volumeLoadingState.length) {
			const state = volumeLoadingState[volumeIndex];
			if (state.totalImages === 0) return 100;
			return Math.round((state.loadedImages / state.totalImages) * 100);
		}
		return 0;
	}

	// Get the total number of images in a chapter
	function getChapterImageCount(chapterIndex: number): number {
		return images[chapterIndex]?.length || 0;
	}

	// Get the total number of images in a volume
	function getVolumeImageCount(volumeIndex: number): number {
		if (!volumeChapters[volumeIndex]) return 0;

		return volumeChapters[volumeIndex].reduce((total, chapterIndex) => {
			return total + getChapterImageCount(chapterIndex);
		}, 0);
	}

	onMount(async () => {
		unregisterKeyHint = keyHint.smartAdd([
			['tab', $t`Navigate`],
			['shift+tab', $t`Navigate`]
		]);

		// Initialize state
		stepState.disablePrev = false;
		stepState.disableNext = false;

		// Get data from the backend
		const result = await wrapper(commands.convStateGet());

		if (result !== null && result.payload !== null) {
			images = result.payload.data || [];
			volumeSizes = result.payload.volume_sizes || [];

			// Create the arrays for tracking state
			selectedImages = $state.snapshot(images); // Deep copy
			isVisibleState = images.map((row) => row.map(() => true));
			coverState = images.map((row) => row.map(() => false));
			imageLoadErrorState = images.map((row) => row.map(() => false));

			// Organize chapters into volumes
			if (volumeSizes.length > 0) {
				volumeChapters = [];
				let chapterIndex = 0;

				// For each volume, allocate the specified number of chapters
				for (let vol = 0; vol < volumeSizes.length; vol++) {
					const chaptersInThisVolume = [];
					const chapterCount = volumeSizes[vol];

					for (let i = 0; i < chapterCount && chapterIndex < images.length; i++) {
						chaptersInThisVolume.push(chapterIndex);
						chapterIndex++;
					}

					volumeChapters.push(chaptersInThisVolume);
				}

				// Initialize volume loading state
				volumeLoadingState = volumeChapters.map((chapters) => {
					const totalImages = chapters.reduce((count, chapterIdx) => {
						return count + images[chapterIdx].length;
					}, 0);

					return {
						loadedImages: 0,
						totalImages: totalImages
					};
				});
			} else {
				// If no volumes defined, treat all chapters as one volume
				volumeChapters = [Array.from(images.keys())];

				// Count total images across all chapters
				const totalImages = images.reduce((count, chapter) => count + chapter.length, 0);

				volumeLoadingState = [
					{
						loadedImages: 0,
						totalImages: totalImages
					}
				];
			}
			// Initialize cover images
			updateCoverStates();
		}
	});

	onDestroy(async () => {
		// Unregister key hint
		if (unregisterKeyHint) unregisterKeyHint();

		// Filter out null entries and save selected images
		const filteredImages = selectedImages.map((row) => row.filter((image) => image !== null));
		await wrapper(commands.convStateSet({ EditedData: filteredImages }));
	});
</script>

<div class="relative flex h-full w-full flex-col p-4" style="max-height: calc(100vh - 8rem)">
	<div class="card flex-1 overflow-hidden">
		<div
			class="card-body grid grid-cols-1 gap-6 overflow-y-auto p-2"
			style="max-height: calc(100vh - 8rem)"
		>
			{#if volumeChapters.length > 0}
				{#each volumeChapters as chapters, volumeIndex}
					<div class="card bg-background-secondary dark:bg-background-dark-tertiary relative">
						<!-- Per-volume loading overlay -->
						{#if !isVolumeLoaded(volumeIndex)}
							<div
								class="bg-background/70 dark:bg-background-dark/70 absolute inset-0 z-50 flex items-center justify-center rounded backdrop-blur-sm"
							>
								<LoadingSpinner text={$t`Loading Images...`} />
							</div>
						{/if}

						<div class="card-header flex items-center">
							<IconBook class="mr-2" size={18} />
							<h4 class="font-semibold">
								{$t`Volume`}
								{volumeIndex + 1} ({getVolumeImageCount(volumeIndex)}
								{$t`Images`})
							</h4>
						</div>
						<div class="card-body p-2">
							{#each chapters as chapterIndex}
								<div class="mb-4">
									<div
										class="border-background-tertiary dark:border-background-dark-tertiary mb-2 flex items-center border-b pb-1"
									>
										<IconNotes class="mr-2" size={16} />
										<h5 class="text-sm font-medium">
											{$t`Chapter`}
											{chapterIndex + 1} ({getChapterImageCount(chapterIndex)}
											{$t`images`})
										</h5>
									</div>
									<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
										{#each images[chapterIndex] as imagePath, imageIndex}
											<div
												class="relative h-40 w-full overflow-hidden rounded select-none"
												onclick={() => toggleOverlay(volumeIndex, chapterIndex, imageIndex)}
												onkeydown={(e) =>
													handleKeyboardNavigation(e, volumeIndex, chapterIndex, imageIndex)}
												role="button"
												tabindex={0}
												use:handleKeyHint={{ keys: [['enter', $t`Toggle`]] }}
											>
												{#if imageLoadErrorState[chapterIndex][imageIndex]}
													<div
														class="bg-background-tertiary dark:bg-background-dark-tertiary text-content-secondary dark:text-content-dark-secondary flex h-full w-full items-center justify-center"
													>
														<IconPhoto size={32} opacity={0.5} />
														<span class="mt-2 text-xs">{$t`Image Error`}</span>
													</div>
												{:else}
													<img
														src={convertFileSrc(imagePath)}
														alt={`Chapter ${chapterIndex + 1}, Image ${imageIndex + 1}`}
														class="bg-background-tertiary dark:bg-background-dark-tertiary h-full w-full object-contain shadow-inner"
														onload={() => handleImageLoaded(volumeIndex)}
														onerror={() => handleImageError(volumeIndex, chapterIndex, imageIndex)}
														draggable="false"
													/>
												{/if}

												{#if !isVisibleState[chapterIndex][imageIndex]}
													<div class="absolute top-0 left-0 z-10 h-full w-full">
														<div
															class="bg-background-tertiary/60 dark:bg-background-dark-tertiary/60 absolute inset-0"
														></div>
														<div
															class="bg-error/90 dark:bg-error-dark/90 absolute top-2 right-2 z-20 rounded-full p-1"
														>
															<IconX size={18} class="text-white" />
														</div>
													</div>
												{/if}

												{#if coverState[chapterIndex][imageIndex]}
													<div
														class="bg-secondary/90 dark:bg-primary/90 absolute top-2 left-2 rounded-md px-2 py-0.5 text-xs font-semibold shadow"
													>
														{$t`Cover`}
													</div>
												{/if}
											</div>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			{:else}
				<!-- Single volume case, showing all chapters -->
				<div class="card bg-background-secondary dark:bg-background-dark-tertiary relative">
					<!-- Single volume case loading overlay -->
					{#if volumeLoadingState.length > 0 && !isVolumeLoaded(0)}
						<div
							class="bg-background/70 dark:bg-background-dark/70 absolute inset-0 z-50 flex items-center justify-center backdrop-blur-sm"
						>
							<LoadingSpinner
								text={$t`Loading Images... ${volumeLoadingState[0].loadedImages}/${volumeLoadingState[0].totalImages} (${getVolumeLoadingProgress(0)}%)`}
							/>
						</div>
					{/if}

					<div class="card-body p-2">
						{#each images as chapter, chapterIndex}
							<div class="mb-4">
								<div
									class="border-background-tertiary dark:border-background-dark-tertiary mb-2 flex items-center border-b pb-1"
								>
									<IconNotes class="mr-2" size={16} />
									<h5 class="text-sm font-medium">
										{$t`Chapter ${chapterIndex + 1}`} ({chapter.length}
										{$t`images`})
									</h5>
								</div>
								<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
									{#each chapter as imagePath, imageIndex}
										<div
											class="relative h-40 w-full overflow-hidden rounded select-none"
											onclick={() => toggleOverlay(0, chapterIndex, imageIndex)}
											onkeydown={(e) => handleKeyboardNavigation(e, 0, chapterIndex, imageIndex)}
											role="button"
											tabindex={0}
											use:handleKeyHint={{ keys: [['enter', $t`Toggle selection`]] }}
										>
											{#if imageLoadErrorState[chapterIndex][imageIndex]}
												<div
													class="bg-background-tertiary dark:bg-background-dark-tertiary text-content-secondary dark:text-content-dark-secondary flex h-full w-full items-center justify-center"
												>
													<IconPhoto size={32} opacity={0.5} />
													<span class="mt-2 text-xs">Image Error</span>
												</div>
											{:else}
												<img
													src={convertFileSrc(imagePath)}
													alt={`Chapter ${chapterIndex + 1}, Image ${imageIndex + 1}`}
													class="bg-background-tertiary dark:bg-background-dark-tertiary h-full w-full object-contain shadow-inner"
													onload={() => handleImageLoaded(0)}
													onerror={() => handleImageError(0, chapterIndex, imageIndex)}
													loading="lazy"
													draggable="false"
												/>
											{/if}

											{#if !isVisibleState[chapterIndex][imageIndex]}
												<div class="absolute top-0 left-0 z-10 h-full w-full">
													<div
														class="bg-background-tertiary/60 dark:bg-background-dark-tertiary/60 absolute inset-0"
													></div>
													<div
														class="bg-error/90 dark:bg-error-dark/90 absolute top-2 right-2 z-20 rounded-full p-1"
													>
														<IconX size={18} class="text-white" />
													</div>
												</div>
											{/if}

											{#if coverState[chapterIndex][imageIndex]}
												<div
													class="bg-secondary/90 dark:bg-secondary-dark-light/90 absolute top-2 left-2 rounded-md px-2 py-0.5 text-xs font-semibold shadow"
												>
													{$t`Cover`}
												</div>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>
