<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { IconEyeClosed, IconNotes, IconPhoto, IconPhotoPlus } from '@tabler/icons-svelte';
	import { flip } from 'svelte/animate';
	import { t } from 'svelte-i18n-lingui';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import convState from '$states/converter.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { Badge } from 'waku/components';
	import { HStack } from 'waku/layout';

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
		chapterIndex = 0,
		images = $bindable(),
		selectedImages = $bindable(),
		isVisibleState = $bindable(),
		coverState = $bindable(),
		imageLoadErrorState = $bindable(),
	}: Props = $props();

	let draggedOverIndex: number | null = $state(null);

	function toggleOverlay(imageIndex: number) {
		// Toggle visibility state
		isVisibleState[chapterIndex][imageIndex] = !isVisibleState[chapterIndex][imageIndex];

		// Toggle selection (null = excluded)
		selectedImages[chapterIndex][imageIndex] = isVisibleState[chapterIndex][imageIndex]
			? images[chapterIndex][imageIndex]
			: null;

		// Add excluded +1 to convState
		if (selectedImages[chapterIndex][imageIndex] === null) {
			// Exclude image
			convState.excludedImages += 1;
		} else {
			// Include image
			convState.excludedImages -= 1;
		}

		// Emit event so parent can update cover states
		onImageChange();
	}

	function handleImageError(imageIndex: number) {
		imageLoadErrorState[chapterIndex][imageIndex] = true;
	}

	function handleKeyboardNavigation(event: KeyboardEvent, imageIndex: number) {
		if (event.key === 'Enter') {
			toggleOverlay(imageIndex);
			event.preventDefault();
		} else if (event.key === 'ArrowRight') {
			const isLast = imageIndex === images[chapterIndex].length - 1;
			moveImage(imageIndex, isLast ? 0 : imageIndex + 1);
			convState.changedOrder = true;
			event.preventDefault();
		} else if (event.key === 'ArrowLeft') {
			const isFirst = imageIndex === 0;
			moveImage(imageIndex, isFirst ? images[chapterIndex].length - 1 : imageIndex - 1);
			convState.changedOrder = true;
			event.preventDefault();
		}
	}

	function getChapterImageCount(): number {
		return images[chapterIndex]?.length || 0;
	}

	// Function to reorder an image within a chapter
	function moveImage(fromIndex: number, toIndex: number) {
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

	async function addImage() {
		const selection = await open({
			multiple: true,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
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

	function dragEnter(event: DragEvent, imageIndex: number) {
		event.preventDefault();
		event.stopPropagation();

		// Only respond if it's our type of drag
		if (event.dataTransfer?.types.includes('application/image-drag')) {
			draggedOverIndex = imageIndex;
		}
	}

	function dragStart(event: DragEvent, imageIndex: number) {
		// Set a specific application type to identify our drag operation
		event.dataTransfer?.setData('application/image-drag', 'true');
		event.dataTransfer?.setData('text/plain', `${imageIndex}`);
		// Set effectAllowed to move to make it clearer this is a move operation
		if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
	}

	function dragOver(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
		// Only respond if it's our type of drag
		if (event.dataTransfer?.types.includes('application/image-drag')) {
			event.dataTransfer.dropEffect = 'move';
		}
	}

	function dragLeave(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
		draggedOverIndex = null;
	}

	function drop(event: DragEvent, imageIndex: number) {
		event.preventDefault();
		event.stopPropagation();
		// Only process if it's our specific drag type
		if (event.dataTransfer?.types.includes('application/image-drag')) {
			const fromIndex = parseInt(event.dataTransfer?.getData('text/plain') || '0');
			moveImage(fromIndex, imageIndex);
			convState.changedOrder = true;
		}
		draggedOverIndex = null;
	}
</script>

<div>
	<HStack gap="sm" align="center" class="text-muted mb-3">
		<IconNotes size={16} />
		<span class="text-xs font-medium">
			{$t`Chapter`}
			{chapterIndex + 1}
		</span>
		<Badge variant="neutral" class="ml-auto">
			{getChapterImageCount()}
			{$t`images`}
		</Badge>
	</HStack>
	<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
		{#each images[chapterIndex] as imagePath, imageIndex (imagePath)}
			<div
				class="group relative h-40 w-full cursor-grab overflow-hidden rounded-lg transition-all select-none"
				class:ring-2={draggedOverIndex === imageIndex}
				class:ring-accent-500={draggedOverIndex === imageIndex}
				class:ring-offset-2={draggedOverIndex === imageIndex}
				class:ring-offset-surface-0={draggedOverIndex === imageIndex}
				animate:flip={{ duration: 300 }}
				onclick={() => toggleOverlay(imageIndex)}
				onkeydown={(e) => handleKeyboardNavigation(e, imageIndex)}
				role="button"
				tabindex={0}
				use:handleKeyHint={{
					keys: [
						['enter', $t`Toggle`],
						['arrowleft', $t`Swap prev`],
						['arrowright', $t`Swap next`],
					],
				}}
				id={'image-' + chapterIndex + '-' + imageIndex}
				draggable="true"
				ondragenter={(evt) => dragEnter(evt, imageIndex)}
				ondragover={dragOver}
				ondragleave={dragLeave}
				ondragstart={(evt) => dragStart(evt, imageIndex)}
				ondrop={(evt) => drop(evt, imageIndex)}
			>
				{#if imageLoadErrorState[chapterIndex][imageIndex]}
					<div
						class="bg-surface-2 text-muted flex h-full w-full flex-col items-center justify-center gap-2"
					>
						<IconPhoto size={32} opacity={0.5} />
						<span class="text-xs">{$t`Image Error`}</span>
					</div>
				{:else}
					<img
						src={convertFileSrc(imagePath)}
						alt={`Chapter ${chapterIndex + 1}, Image ${imageIndex + 1}`}
						class="bg-surface-2 h-full w-full object-contain"
						onerror={() => handleImageError(imageIndex)}
						draggable="false"
					/>
				{/if}

				{#if !isVisibleState[chapterIndex][imageIndex]}
					<div
						class="absolute inset-0 z-10 flex items-center justify-center bg-black/40 backdrop-blur-sm"
					>
						<IconEyeClosed size={32} class="text-white drop-shadow-lg" />
					</div>
				{/if}

				{#if coverState[chapterIndex][imageIndex]}
					<Badge variant="primary" style="solid" class="absolute top-2 left-2">
						{$t`Cover`}
					</Badge>
				{/if}
			</div>
		{/each}
		<div
			class="hover:bg-surface-2 border-muted text-muted group relative flex h-40 w-full cursor-pointer items-center justify-center rounded-lg border-2 border-dashed transition-colors"
			tabindex={0}
			role="button"
			onclick={async () => await addImage()}
			use:handleKeyHint={{
				keys: [['enter', $t`Add Image`]],
			}}
			onkeydown={async (e) => {
				if (e.key === 'Enter') {
					e.preventDefault();
					await addImage();
				}
			}}
		>
			<IconPhotoPlus size={32} class="group-hover:text-accent-500 transition-colors" />
		</div>
	</div>
</div>
