<script lang="ts">
	import { converter, toast } from '$lib/stores';
	import Background from '@tabler/icons-svelte/IconBackground.svelte';
	import { popup, type PopupSettings, ProgressRadial, RangeSlider } from '@skeletonlabs/skeleton';
	import Display from '$components/display.svelte';
	import { bridge } from '$lib/functions';
	import { type AnalyzeResult, InfoType, type Toast } from '$lib/types';

	const popupSensibility: PopupSettings = {
		event: 'click',
		target: 'popupSensibility',
		placement: 'bottom'
	};

	// function
	async function analysis() {
		$converter.analysis.running = true;
		const result = await bridge<AnalyzeResult>(
			'analyze',
			{
				sourceDirectory: $converter.sourceDirectory,
				sensibility: $converter.analysis.sensibility
			}
		);
		$converter.analysis.running = false;

		if (result) {
			if (result.chapter_per_volume) {
				$converter.volumes = result.chapter_per_volume;
			} else {
				$toast = {
					type: InfoType.WARNING,
					message: 'No volumes detected in the selected directory. \nTry adjusting the sensibility.',
					timeout: 5000
				} as Toast;
			}
		}

	}
</script>

<div class="card row-span-1">
	<header class="card-header">
		<h2 class="text-xl font-bold">
			<span
				class="bg-gradient-to-bl from-fuchsia-400 to-indigo-500 bg-clip-text text-transparent box-decoration-clone"
			>
					Analysis
			</span>
		</h2>
	</header>
	<section class="p-4">
		<div class="table-container flex flex-col items-center justify-center">
			<!-- Native Table Element -->
			<table class="table table-interactive">
				<tbody>
				<tr use:popup={popupSensibility} class="select-none">
					<td class="font-bold text-secondary-500 flex items-center">
						<Background class="mr-0.5" />
						Cover Detection Sensibility
						<Display
							high
							placement="bottom"
						>
							The cover detection sensibility is a value between <code>0</code> and <code>100</code> that determines how
							sensitive the cover detection algorithm is.
							<br />
							A cover is determined by the amount of pixels that are black and white and if it is the first image in the
							chapter.
							<br />
							A sensibility of <code>75%</code> will detect covers that are less than <code>75%</code> black and white.
							<br />
							A sensibility of <code>75%</code> is recommended.
						</Display>
					</td>
					<td>
						<code class="code whitespace-pre-wrap">
							{$converter.analysis.sensibility}
						</code>
					</td>
				</tr>
				</tbody>
			</table>
			<div class="flex flex-row items-center justify-center mt-2">
				<button
					class="btn variant-filled-success dark:variant-soft-success"
					on:click={analysis}
					disabled={$converter.analysis.running || $converter.running}
				>
					{#if $converter.analysis.running}
						<ProgressRadial
							value={undefined}
							width="w-6"
							stroke={75}
						/>
					{:else}
						Run Analysis
					{/if}
				</button>
				<Display
					color="dark:text-success-500/60 text-black"
					high
				>
					The analysis will determine the amount of volumes the selected directory contains and how many chapters to
					allocate to each volume.
					<br />
					The found volume data can still be edited in the tab left to this one.
				</Display>
			</div>
		</div>
	</section>
</div>
<!-- popups -->
<div class="card variant-glass-surface w-1/3 p-4 shadow-xl" data-popup="popupSensibility">
	<RangeSlider name="range-slider" bind:value={$converter.analysis.sensibility} min={0} max={100} step={1}>
	</RangeSlider>
	<div class="arrow variant-glass-surface" />
</div>
