<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { IconEyeClosed, IconNotes, IconPhoto, IconPhotoPlus } from '@tabler/icons-svelte';
	import { flip } from 'svelte/animate';
	import { fade } from 'svelte/transition';
	import { t } from 'svelte-i18n-lingui';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import convState from '$states/converter.svelte';
	import { open } from '@tauri-apps/plugin-dialog';

	interface Props {
		onImageChange: () => void;
		chapterIndex: number;
		images: string[][];
		selectedImages: (string | null)[][];
		isVisibleState: boolean[][];
		coverState: boolean[][];
		imageLoadErrorState: boolean[][];
	}

	let {
		onImageChange = () => {},
		chapterIndex = $bindable(0),
		images = $bindable(),
		selectedImages = $bindable(),
		isVisibleState = $bindable(),
		coverState = $bindable(),
		imageLoadErrorState = $bindable()
	}: Props = $props();

	let draggedOverIndex: number | null = $state(null);

	function toggleOverlay(chapIndex: number, imageIndex: number) {
		// Toggle visibility state
		isVisibleState[chapIndex][imageIndex] = !isVisibleState[chapIndex][imageIndex];

		// Toggle selection (null = excluded)
		selectedImages[chapIndex][imageIndex] = isVisibleState[chapIndex][imageIndex]
			? images[chapIndex][imageIndex]
			: null;

		// Add excluded +1 to convState
		if (selectedImages[chapIndex][imageIndex] === null) {
			// Exclude image
			convState.excludedImages += 1;
		} else {
			// Include image
			convState.excludedImages -= 1;
		}

		// Emit event so parent can update cover states
		onImageChange();
	}

	function handleImageError(chapIndex: number, imageIndex: number) {
		imageLoadErrorState[chapIndex][imageIndex] = true;
	}

	function handleKeyboardNavigation(event: KeyboardEvent, chapIndex: number, imageIndex: number) {
		if (event.key === 'Enter') {
			toggleOverlay(chapIndex, imageIndex);
			event.preventDefault();
		} else if (event.key === 'ArrowRight') {
			const isLast = imageIndex === images[chapterIndex].length - 1;
			moveImage(chapterIndex, imageIndex, isLast ? 0 : imageIndex + 1);
			convState.changedOrder = true;
			event.preventDefault();
		} else if (event.key === 'ArrowLeft') {
			const isFirst = imageIndex === 0;
			moveImage(
				chapterIndex,
				imageIndex,
				isFirst ? images[chapterIndex].length - 1 : imageIndex - 1
			);
			convState.changedOrder = true;
			event.preventDefault();
		}
	}

	function getChapterImageCount(): number {
		return images[chapterIndex]?.length || 0;
	}

	// Function to reorder an image within a chapter
	function moveImage(chapterIndex: number, fromIndex: number, toIndex: number) {
		// Remove and re-insert the element at the new position in all arrays
		const item = images[chapterIndex].splice(fromIndex, 1)[0];
		images[chapterIndex].splice(toIndex, 0, item);

		const selItem = selectedImages[chapterIndex].splice(fromIndex, 1)[0];
		selectedImages[chapterIndex].splice(toIndex, 0, selItem);

		const visItem = isVisibleState[chapterIndex].splice(fromIndex, 1)[0];
		isVisibleState[chapterIndex].splice(toIndex, 0, visItem);

		const covItem = coverState[chapterIndex].splice(fromIndex, 1)[0];
		coverState[chapterIndex].splice(toIndex, 0, covItem);

		const errItem = imageLoadErrorState[chapterIndex].splice(fromIndex, 1)[0];
		imageLoadErrorState[chapterIndex].splice(toIndex, 0, errItem);

		requestAnimationFrame(() => {
			const element = document.getElementById(`image-${chapterIndex}-${toIndex}`);
			element?.focus();
		});

		onImageChange();
	}

	async function addImage(chapterIndex: number) {
		const selection = await open({
			multiple: true,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]
		});

		if (selection === null) return;

		function add(imagePath: string) {
			// Extract file extension
			const lastDotIndex = imagePath.lastIndexOf('.');
			if (lastDotIndex === -1) return; // No extension, skip file

			const extension = imagePath.substring(lastDotIndex + 1);
			const validExtensions = ['png', 'jpg', 'jpeg', 'webp'];

			// Reject if extension is not lowercase or not valid
			if (extension !== extension.toLowerCase() || !validExtensions.includes(extension)) return;

			// Add the path to arrays only if it passes validation
			images[chapterIndex].push(imagePath);
			selectedImages[chapterIndex].push(imagePath);
			isVisibleState[chapterIndex].push(true);
			coverState[chapterIndex].push(false);
			imageLoadErrorState[chapterIndex].push(false);
			convState.newImages++;
		}

		// When its multiple, add all selected images
		if (Array.isArray(selection)) {
			// Check if the selection is empty
			if (selection.length === 0) return;

			for (const imagePath of selection) {
				add(imagePath);
			}
		} else {
			// Single file selected
			add(selection);
		}
	}
</script>

<div class="mb-4">
	<div
		class="border-background-tertiary dark:border-background-dark-tertiary mb-2 flex items-center border-b pb-1"
	>
		<IconNotes class="mr-2" size={16} />
		<h5 class="text-sm font-medium">
			{$t`Chapter`}
			{chapterIndex + 1} ({getChapterImageCount()}
			{$t`images`})
		</h5>
	</div>
	<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
		{#each images[chapterIndex] as imagePath, imageIndex (imagePath)}
			<div
				class="relative h-40 w-full overflow-hidden rounded select-none"
				class:drag-over={draggedOverIndex === imageIndex}
				animate:flip={{ duration: 300 }}
				onclick={() => toggleOverlay(chapterIndex, imageIndex)}
				onkeydown={(e) => handleKeyboardNavigation(e, chapterIndex, imageIndex)}
				role="button"
				tabindex={0}
				use:handleKeyHint={{
					keys: [
						['enter', $t`Toggle`],
						['arrowleft', $t`Swap prev`],
						['arrowright', $t`Swap next`]
					]
				}}
				id={'image-' + chapterIndex + '-' + imageIndex}
				draggable="true"
				ondragstart={(e) => {
					e.stopPropagation();
					// Set a specific application type to identify our drag operation
					e.dataTransfer?.setData('application/image-drag', 'true');
					e.dataTransfer?.setData('text/plain', `${imageIndex}`);
					// Set effectAllowed to move to make it clearer this is a move operation
					if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
				}}
				ondragenter={(e) => {
					e.preventDefault();
					e.stopPropagation();
					// Only respond if it's our type of drag
					if (e.dataTransfer?.types.includes('application/image-drag')) {
						draggedOverIndex = imageIndex;
					}
				}}
				ondragover={(e) => {
					e.preventDefault();
					e.stopPropagation();
					// Only respond if it's our type of drag
					if (e.dataTransfer?.types.includes('application/image-drag')) {
						e.dataTransfer.dropEffect = 'move';
					}
				}}
				ondragleave={(e) => {
					e.preventDefault();
					e.stopPropagation();
					draggedOverIndex = null;
				}}
				ondrop={(e) => {
					e.preventDefault();
					e.stopPropagation();
					// Only process if it's our specific drag type
					if (e.dataTransfer?.types.includes('application/image-drag')) {
						const fromIndex = parseInt(e.dataTransfer?.getData('text/plain') || '0');
						moveImage(chapterIndex, fromIndex, imageIndex);
						convState.changedOrder = true;
					}
					draggedOverIndex = null;
				}}
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
						onerror={() => handleImageError(chapterIndex, imageIndex)}
						draggable="false"
					/>
				{/if}

				{#if !isVisibleState[chapterIndex][imageIndex]}
					<div class="absolute top-0 left-0 z-10 h-full w-full">
						<div
							class="bg-secondary/70 dark:bg-secondary-dark/70 absolute inset-0 flex items-center justify-center"
						>
							<IconEyeClosed size={32} class="stroke-black" />
						</div>
					</div>
				{/if}

				{#if coverState[chapterIndex][imageIndex]}
					<div
						transition:fade={{ duration: 150 }}
						class="bg-secondary/90 dark:bg-primary/90 absolute top-2 left-2 rounded-md px-2 py-0.5 text-xs font-semibold shadow"
					>
						{$t`Cover`}
					</div>
				{/if}
			</div>
		{/each}
		<div
			class="border-content-tertiary text-content-tertiary relative flex h-40 w-full cursor-pointer items-center justify-center rounded border border-dashed"
			tabindex={0}
			role="button"
			onclick={async () => await addImage(chapterIndex)}
			use:handleKeyHint={{
				keys: [['enter', $t`Add Image`]]
			}}
			onkeydown={async (e) => {
				if (e.key === 'Enter') {
					e.preventDefault();
					await addImage(chapterIndex);
				}
			}}
		>
			<IconPhotoPlus size={32} />
		</div>
	</div>
</div>

<style>
	.drag-over {
		outline: 2px solid var(--color-primary);
		outline-offset: -2px;
	}
</style>
