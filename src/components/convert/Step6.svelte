<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { IconBook } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';
	import Chapter from '$components/convert/step5/Chapter.svelte';

	// Waku Imports
	import { VStack, BentoGrid, BentoItem, HStack } from 'waku/layout';
	import { Badge } from 'waku/components';

	let images: string[][] = $state([]);
	let selectedImages: (string | null)[][] = $state([]);
	let isVisibleState: boolean[][] = $state([]); // true = visible (included), false = hidden (excluded)
	let coverState: boolean[][] = $state([]); // Track cover images
	let imageLoadErrorState: boolean[][] = $state([]); // Track image loading errors
	let unregisterKeyHint: () => void;
	let isGlobalLoading = $state(true);

	// Volume-specific data
	let volumeSizes: number[] = $state([]);
	let volumeChapters: number[][] = $state([]); // Tracks which chapters belong to which volume

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
		unregisterKeyHint = keyHint.register([
			['tab', $t`Navigate`],
			['shift+tab', $t`Navigate`],
		]);

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
			} else {
				// If no volumes defined, treat all chapters as one volume
				volumeChapters = [Array.from(images.keys())];
			}
			// Initialize cover images
			updateCoverStates();
		}

		const totalImages = images.reduce((sum, ch) => sum + ch.length, 0);
		const loadTime = 800 + totalImages;
		setTimeout(() => (isGlobalLoading = false), loadTime);
	});

	onDestroy(async () => {
		// Unregister key hint
		if (unregisterKeyHint) unregisterKeyHint();

		// Filter out null entries and save selected images
		const filteredImages = selectedImages.map((row) => row.filter((image) => image !== null));
		await wrapper(commands.convStateSet({ EditedData: filteredImages }));
	});
</script>

{#if isGlobalLoading}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<LoadingSpinner text={$t`Loading Images...`} />
	</div>
{/if}

<div class="h-full w-full p-3">
	<div class="h-full overflow-y-auto">
		<VStack gap="md" class="mx-auto max-w-6xl pb-4">
			<BentoGrid cols={1} density="compact">
				{#if volumeChapters.length > 0}
					{#each volumeChapters as chapters, volumeIndex}
						<BentoItem variant="glass">
							<HStack align="center" gap="sm" class="text-muted mb-4">
								<IconBook size={18} />
								<span class="text-xs font-bold tracking-wider uppercase">
									{$t`Volume`}
									{volumeIndex + 1}
								</span>
								<Badge variant="secondary" class="ml-auto">
									{getVolumeImageCount(volumeIndex)}
									{$t`Images`}
								</Badge>
							</HStack>

							<VStack gap="md">
								{#each chapters as chapterIndex}
									<Chapter
										{chapterIndex}
										bind:images
										bind:selectedImages
										bind:isVisibleState
										bind:coverState
										bind:imageLoadErrorState
										onImageChange={updateCoverStates}
									/>
								{/each}
							</VStack>
						</BentoItem>
					{/each}
				{:else}
					<!-- Single volume case, showing all chapters -->
					<BentoItem variant="glass">
						<HStack align="center" gap="sm" class="text-muted mb-4">
							<IconBook size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`All Chapters`}</span>
						</HStack>

						<Chapter
							chapterIndex={0}
							bind:images
							bind:selectedImages
							bind:isVisibleState
							bind:coverState
							bind:imageLoadErrorState
							onImageChange={updateCoverStates}
						/>
					</BentoItem>
				{/if}
			</BentoGrid>
		</VStack>
	</div>
</div>
