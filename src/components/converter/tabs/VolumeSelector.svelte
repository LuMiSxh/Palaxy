<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { converterTab, tabDisableBack, tabDisableNext, toast } from '$lib/stores';
	import { BundlerFlag, type TabVolumeResult } from '$components/converter/types';
	import { onMount } from 'svelte';
	import { bridge } from '$lib/functions';
	import { ProgressRadial, RangeSlider } from '@skeletonlabs/skeleton';
	import { InfoType, type Toast } from '$lib/types';
	import Plus from '@tabler/icons-svelte/IconPlus.svelte';
	import Minus from '@tabler/icons-svelte/IconMinus.svelte';

	export let bundler: Writable<BundlerFlag | null>;
	export let sourceManga: Writable<string | null>;
	export let cpv: Writable<Array<number>>;

	let loading = true;
	let results: TabVolumeResult | undefined;
	let sensibility: number = 75;

	// Initial state
	$tabDisableBack = false;
	$tabDisableNext = true;

	// Change the state only when there are cpv
	cpv.subscribe((value) => {
		$tabDisableNext = value.length <= 0;
	});

	async function runBundler() {
		loading = true;
		$tabDisableNext = true;
		if ($sourceManga === null) {
			$converterTab = 0;
			return;
		}

		const result = await bridge<TabVolumeResult>('flow_volume', {
			basePath: $sourceManga,
			bundlerFlag: $bundler,
			sensibility
		});
		loading = false;

		if (result) {
			results = result;
			$cpv = results.chapters_per_volume ? results.chapters_per_volume : [];
		}
	}

	onMount(async () => {
		await runBundler();
	});

	// Functions for manual bundling //

	let tempChapters: number | undefined;

	// This is used to make the enter-key work for adding chapters and focusing on the next field
	let input: HTMLInputElement;

	function addVolume() {
		if (tempChapters === undefined) {
			return;
		}

		$cpv.push(tempChapters);
		$cpv = $cpv;
		tempChapters = undefined;
		input.focus();

		// Check if the total number of chapters exceeds the detected number of chapters
		if (results !== undefined && $cpv.reduce((acc, cur) => acc + cur, 0) > results.total_chapters) {
			toast.set({
				type: InfoType.WARNING,
				message: `The total number of chapters
				(${$cpv.reduce((acc, cur) => acc + cur, 0)}) used exceeds the detected
				number of chapters (${results.total_chapters})`,
				timeout: 5000
			} as Toast);
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			addVolume();
		}
	}

	// Functions for image bundling //
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	{#if loading}
		<div class="h-full w-full flex items-center justify-center animate-pulse">
			<ProgressRadial value={undefined} strokeLinecap="round" meter="stroke-secondary-500" />
		</div>
	{:else if $bundler === BundlerFlag.MANUAL}
		<div class="relative table-container max-h-[40vh] w-2/3 overflow-y-auto">
			<table class="table table-interactive">
				<thead>
					<tr class="select-none">
						<th>Volume</th>
						<th>Chapters</th>
					</tr>
				</thead>
				<tbody>
					<!-- eslint-disable-next-line -->
					{#each $cpv as _, i}
						<tr class="select-none">
							<td class="font-bold text-primary-500">
								<span class="pl-8">
									{i + 1}
								</span>
							</td>
							<td class="flex items-center justify-evenly">
								<input
									class="input w-1/2 h-7"
									type="number"
									min="1"
									max="999"
									bind:value={$cpv[i]}
								/>
								<button
									class="btn variant-filled-error dark:variant-soft-error h-7"
									on:click={() => {
										$cpv.splice(i, 1);
										$cpv = $cpv;
									}}
								>
									<Minus size={18} />
								</button>
							</td>
						</tr>
					{/each}
					<tr class="animate-pulse opacity-75 select-none">
						<td class="font-bold text-secondary-500">
							<span class="pl-8">
								{$cpv.length + 1}
							</span>
						</td>
						<td class="flex items-center justify-evenly">
							<input
								class="input w-1/2 h-7"
								type="number"
								min="1"
								max="999"
								bind:value={tempChapters}
								bind:this={input}
								on:keydown={handleKeyDown}
							/>
							<button
								class="btn variant-filled-success dark:variant-soft-success h-7"
								on:click={addVolume}
							>
								<Plus size={18} />
							</button>
						</td>
					</tr>
				</tbody>
			</table>
		</div>
	{:else}
		<div class="flex w-full h-full items-start justify-evenly">
			<div class="table-container w-1/3 max-h-[45vh]">
				<table class="table table-hover">
					<thead>
						<tr>
							<th></th>
							<th>Results</th>
						</tr>
					</thead>
					<tbody>
						<tr>
							<td>Detected Chapters</td>
							<td>
								<code class="code">
									{results?.total_chapters}
								</code>
							</td>
						</tr>
						<tr>
							<td>Detected Volumes</td>
							<td>
								<code class="code">
									{results?.total_volumes}
								</code>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
			<div
				class="w-1/3 flex flex-col items-center justify-center"
				class:px-3={$bundler === BundlerFlag.IMAGE}
			>
				<div class="table-container max-h-[40vh] overflow-y-auto">
					<table class="table table-hover">
						<thead>
							<tr>
								<th>Volume</th>
								<th>Chapters</th>
							</tr>
						</thead>
						<tbody>
							{#each $cpv as chapters, i}
								<tr>
									<td class="font-bold text-primary-500">
										<span class="pl-8">
											{i + 1}
										</span>
									</td>
									<td>
										<span class="pl-8">
											<code class="code">
												{chapters}
											</code>
										</span>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
			{#if $bundler === BundlerFlag.IMAGE}
				<div class="w-1/3 flex flex-col items-center justify-center pl-4">
					<RangeSlider
						name="range-slider"
						class="w-full"
						bind:value={sensibility}
						max={100}
						step={5}
						ticked
					>
						Grayscale Sensibility:
						<code class="code">
							{sensibility}%
						</code>
					</RangeSlider>
					<div class="pt-4 w-full h-full flex items-center justify-around">
						<button
							class="btn variant-filled-success dark:variant-soft-success"
							on:click={runBundler}
						>
							Rerun Bundler
						</button>
						<button
							class="btn variant-filled-warning dark:variant-soft-warning"
							on:click={() => ($bundler = BundlerFlag.MANUAL)}
						>
							Manual Bundling
						</button>
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>
