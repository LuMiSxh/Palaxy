<script lang="ts">
	import {
		IconAdjustments,
		IconFileZip,
		IconFilter,
		IconFolder,
		IconHandStop,
		IconLineScan,
		IconPencil
	} from '@tabler/icons-svelte';
	import { Step1, Step2, Step3, Step4, Step5, Step6, Step7 } from '$components/convert';

	let index = $state(0);

	const classActive = 'step-secondary';

	function cls(i: number): string {
		return index >= i ? classActive : '';
	}

	let steps = $derived.by(() => {
		return [
			{
				title: 'Choose Base Directory',
				icon: IconFolder,
				cmp: Step1,
				cls: cls(0),
				hidden: false
			},
			{
				title: 'Analysis',
				icon: IconLineScan,
				cmp: Step2,
				cls: cls(1),
				hidden: false
			},
			{
				title: 'Choose Bundling Method',
				icon: IconFileZip,
				cmp: Step3,
				cls: cls(2),
				hidden: false
			},
			{
				title: 'Manual Bundling',
				Icon: IconHandStop,
				cmp: Step4,
				cls: cls(3),
				hidden: true
			},
			{
				title: 'Filter Pages',
				icon: IconFilter,
				cmp: Step5,
				cls: cls(4),
				hidden: false
			},
			{
				title: 'Choose Conversion Options',
				icon: IconAdjustments,
				cmp: Step6,
				cls: cls(5),
				hidden: false
			},
			{
				title: 'Review',
				icon: IconPencil,
				cmp: Step7,
				cls: cls(6),
				hidden: false
			}
		];
	});

	let activeComponent = $derived(steps[index].cmp);
</script>

<div class="flex h-full w-full">
	<ul class="glass-surface steps steps-vertical h-full mr-2">
		{#each steps as step}
			{#if !step.hidden}
				<li class="step {step.cls}">
					<span class="step-icon"><step.icon /></span>
					<span class="w-full h-full flex justify-center items-center">{step.title}</span>
				</li>
			{/if}
		{/each}
	</ul>
	<section class="w-full h-full glass-surface ml-2">
		<div class="w-full h-[90%]">
			{@render activeComponent()}
		</div>
		<div class="w-full h-[10%] flex justify-between items-center">
			<button class="btn btn-soft btn-error" onclick={() => index--} disabled={index === 0}>Previous</button>
			<button class="btn btn-soft btn-success" onclick={() => index++} disabled={index === steps.length - 1}>Next</button>
		</div>
	</section>
</div>
