<script lang="ts">
	import { writable } from "svelte/store"
	import { type CommandGetData } from "$components/converter/types"
	import { onDestroy, onMount, tick } from "svelte"
	import { bridge } from "$lib/functions"
	import { disableBack, disableNext, loading } from "$components/converter/stores"
	import { convertFileSrc } from "@tauri-apps/api/core"

	// Initial state
	$disableBack = false
	$disableNext = false
	$loading = true

	let images: string[][] = []
	const imagesNew = writable<Array<Array<string | null>>>([])
	const hiddenState = writable<Array<Array<boolean>>>([])
	let imagesLoaded = 0

	function toggleOverlay(row: number, col: number) {
		hiddenState.update(state => {
			state[row][col] = !state[row][col]

			return state
		})

		imagesNew.update(images => {
			if (images[row][col] === null) {
				images[row][col] = images[row][col]
			} else images[row][col] = null

			return images
		})
	}

	async function checkAllImagesLoaded() {
		if (imagesLoaded === images.flat().length) {
			await tick()
			$loading = false
		}
	}

	onMount(async () => {
		$loading = true
		const result = await bridge<CommandGetData>("get_data")
		if (result) {
			images = result.data
			$imagesNew = structuredClone(result.data) // Needed so both variables are not pointing to the same reference
			hiddenState.set(result.data.map(row => row.map(() => true)))
		}
	})

	onDestroy(async () => {
		// remove the images that are null
		$imagesNew = $imagesNew.map(row => row.filter(image => image !== null))
		await bridge("set_data", { data: $imagesNew })
	})
</script>

<div
	class="mx-0.5 grid h-full w-fit grid-cols-6 gap-1 overflow-y-auto"
	style="height: calc(100vh - 14rem)"
>
	{#each images as row, i}
		{#each row as image, j}
			<div
				class="relative h-full w-full select-none"
				on:click={() => toggleOverlay(i, j)}
				role="button"
				tabindex={i + j}
				on:keypress
			>
				<img
					src={convertFileSrc(image)}
					alt={image}
					class="h-full w-full object-contain shadow-inner"
					on:load={async () => {
						imagesLoaded++
						await checkAllImagesLoaded()
					}}
					loading="lazy"
					draggable="false"
				/>
				{#if !$hiddenState[i][j]}
					<div
						class="absolute left-0 top-0 z-50 h-full w-full bg-gradient-to-bl from-primary-500/35 to-secondary-500/35"
					></div>
				{/if}
			</div>
		{/each}
	{/each}
</div>
