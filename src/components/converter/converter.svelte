<script lang="ts">
	import { appData, converter } from '$lib/stores';
	import { ProgressRadial } from '@skeletonlabs/skeleton';
	import { converterDefault } from '$lib/constants';
	import { bridge } from '$lib/functions';
	import type { ConvertResult } from '$lib/types';

	// functions
	function resetConverter() {
		$converter = { ...converterDefault, targetDirectory: $appData.paths.converted ?? converterDefault.targetDirectory };
	}

	async function convert() {
		$converter.running = true;
		await bridge<ConvertResult>('convert', {
			sourceDirectory: $converter.sourceDirectory,
			targetDirectory: $converter.targetDirectory,
			chapterPerVolume: $converter.volumes,
			fileFormat: $converter.format,
			direction: $converter.direction
		});
		$converter.running = false;
	}
</script>

<div class="card row-span-1">
	<header class="card-header">
		<h2 class="text-xl font-bold">
			<span
				class="bg-gradient-to-bl from-primary-500 to-fuchsia-400 bg-clip-text text-transparent box-decoration-clone"
			>
					Conversion
			</span>
		</h2>
	</header>
	<section class="p-4">
		<div class="grid grid-cols-2 grid-rows-1 gap-4 items-center justify-center">
			<button
				on:click={convert}
				class="mt-2 btn variant-filled-success dark:variant-soft-success"
				disabled={$converter.analysis.running || $converter.running}
			>
				{#if $converter.analysis.running}
					<ProgressRadial
						value={undefined}
						width="w-6"
						stroke={75}
					/>
				{:else}
					Run Converter
				{/if}
			</button>
			<button
				on:click={resetConverter}
				class="mt-2 btn variant-filled-error dark:variant-soft-error"
				disabled={$converter.analysis.running || $converter.running}
			>
				{#if $converter.analysis.running}
					<ProgressRadial
						value={undefined}
						width="w-6"
						stroke={75}
					/>
				{:else}
					Reset Converter
				{/if}
			</button>
		</div>
	</section>
</div>