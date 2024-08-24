<script lang="ts">
	import { toast } from "$lib/stores"
	import { BundleFlag, type CommandBundle } from "$components/converter/types"
	import { onDestroy, onMount } from "svelte"
	import { bridge } from "$lib/functions"
	import { RangeSlider } from "@skeletonlabs/skeleton"
	import { InfoType, type Toast } from "$lib/types"
	import { IconMinus, IconPlus } from "@tabler/icons-svelte"
	import {
		bundle,
		chapterSizes,
		disableBack,
		disableNext,
		loading,
	} from "$components/converter/stores"

	let results: CommandBundle | undefined
	let sensibility: number = 75

	// Initial state
	$disableBack = false
	$disableNext = true

	// Change the state only when there are cpv
	const unsubscribe = chapterSizes.subscribe(value => {
		$disableNext = value.length <= 0
	})

	async function runBundler() {
		$loading = true
		$disableBack = true
		$disableNext = true

		await bridge("set_bundle_flag", { flag: $bundle })

		const result = await bridge<CommandBundle>("bundle", {
			sensibility,
		})
		$loading = false

		if (result) {
			console.log(result)
			results = result
			$chapterSizes = results.chapter_sizes ?? []
		}

		$disableBack = false
		$disableNext = false
	}

	onMount(async () => {
		await runBundler()
	})

	onDestroy(async () => {
		unsubscribe()
		await bridge("set_volume_sizes", { sizes: $chapterSizes })
	})

	// Functions for manual bundling //

	let tempChapters: number | undefined

	// This is used to make the enter-key work for adding chapters and focusing on the next field
	let input: HTMLInputElement

	function addVolume() {
		if (tempChapters === undefined) {
			return
		}

		$chapterSizes.push(tempChapters)
		$chapterSizes = $chapterSizes
		tempChapters = undefined
		input.focus()

		// Check if the total number of chapters exceeds the detected number of chapters
		if (
			results !== undefined &&
			$chapterSizes.reduce((acc, cur) => acc + cur, 0) > results.total_chapters
		) {
			toast.set({
				type: InfoType.WARNING,
				message: `The total number of chapters
				(${$chapterSizes.reduce((acc, cur) => acc + cur, 0)}) used exceeds the detected
				number of chapters (${results.total_chapters})`,
				timeout: 5000,
			} as Toast)
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		// If the enter key is pressed, add the volume and focus on the next input field
		if (event.key === "Enter") {
			addVolume()
			input.focus()
			// if a delete key is pressed, remove the last volume
		} else if (event.key === "Delete") {
			$chapterSizes.pop()
			$chapterSizes = $chapterSizes
		}
	}
</script>

<div
	class="flex h-full w-full flex-col items-center justify-center"
	style="height: calc(100vh - 14rem)"
>
	{#if $loading}
		<div class="w-0.5"></div>
	{:else if $bundle === BundleFlag.MANUAL}
		<div class="table-container relative h-full w-full overflow-y-auto p-2">
			<table class="table table-interactive">
				<thead>
					<tr class="select-none">
						<th class="text-center">Volume</th>
						<th class="text-center">Chapters</th>
					</tr>
				</thead>
				<tbody>
					<!-- eslint-disable-next-line -->
					{#each $chapterSizes as _, i}
						<tr class="select-none">
							<td class="text-center font-bold">
								{i + 1}
							</td>
							<td class="flex items-center justify-evenly">
								<input
									class="input h-7 w-1/2"
									type="number"
									min="1"
									max="999"
									bind:value={$chapterSizes[i]}
								/>
								<button
									class="variant-filled-secondary btn h-7"
									on:click={() => {
										$chapterSizes.splice(i, 1)
										$chapterSizes = $chapterSizes
									}}
								>
									<IconMinus size={18} />
								</button>
							</td>
						</tr>
					{/each}
					<tr class="select-non">
						<td class="text-center font-bold text-secondary-500">
							{$chapterSizes.length + 1}
						</td>
						<td class="flex items-center justify-evenly">
							<input
								class="input h-7 w-1/2"
								type="number"
								min="1"
								max="999"
								bind:value={tempChapters}
								bind:this={input}
								on:keydown={handleKeyDown}
							/>
							<button class="variant-filled-primary btn h-7" on:click={addVolume}>
								<IconPlus size={18} />
							</button>
						</td>
					</tr>
				</tbody>
			</table>
		</div>
	{:else}
		<div
			class={"grid h-full w-full grid-rows-1 gap-2 " +
				($bundle === BundleFlag.IMAGE ? "grid-cols-3" : "grid-cols-2")}
		>
			<div class="table-container h-full w-full p-2">
				<table class="table table-hover">
					<thead>
						<tr>
							<th></th>
							<th class="text-center">Results</th>
						</tr>
					</thead>
					<tbody>
						<tr>
							<td>Detected Chapters</td>
							<td>
								<code class="">
									{results?.total_chapters}
								</code>
							</td>
						</tr>
						<tr>
							<td>Detected Volumes</td>
							<td>
								<code class="">
									{results?.total_volumes}
								</code>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
			<div class="table-container h-full w-full overflow-y-auto p-2">
				<table class="table table-hover">
					<thead>
						<tr>
							<th class="text-center">Volume</th>
							<th class="text-center">Chapters</th>
						</tr>
					</thead>
					<tbody>
						{#each $chapterSizes as chapters, i}
							<tr>
								<td class="text-center font-bold text-primary-500">
									{i + 1}
								</td>
								<td class="text-center">
									<code class="">
										{chapters}
									</code>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			{#if $bundle === BundleFlag.IMAGE}
				<div class="table-container h-full w-full p-2">
					<table class="table table-hover">
						<thead>
							<tr>
								<th>Image Detection</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>
									<RangeSlider
										name="range-slider"
										class="w-full"
										bind:value={sensibility}
										max={100}
										step={5}
										ticked
									>
										Grayscale Sensibility:
										<code class="text-white">
											{sensibility}%
										</code>
									</RangeSlider>
								</td>
							</tr>
							<tr>
								<td class="flex h-full w-full items-center justify-around pt-4">
									<button
										class="variant-filled-primary btn"
										on:click={runBundler}
									>
										Rerun Bundler
									</button>
									<button
										class="variant-filled-secondary btn"
										on:click={() => ($bundle = BundleFlag.MANUAL)}
									>
										Manual Bundling
									</button>
								</td>
							</tr></tbody
						>
					</table>
				</div>
			{/if}
		</div>
	{/if}
</div>
