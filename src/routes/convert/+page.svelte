<script lang="ts">
	import {
		IconAdjustments,
		IconFilter,
		IconFolder,
		IconHandStop,
		IconLineScan,
		IconPencil
	} from '@tabler/icons-svelte';
	import { Step1, Step2, Step3, Step4, Step5, Step7 } from '$components/convert';
	import { stepState } from '$states/converter.svelte';
	import { t } from 'svelte-i18n-lingui';

	const classActive = 'step-secondary';

	// Reset the state of the stepState
	stepState.reset();

	function cls(i: number): string {
		return stepState.index >= i ? classActive : '';
	}

	let steps = $derived.by(() => {
		return [
			{
				title: $t`Choose Source Material Directory`,
				description: $t`Select the directory where the source material is located.`,
				icon: IconFolder,
				cmp: Step1,
				cls: cls(0),
				hidden: false
			},
			{
				title: $t`Analysis`,
				description: $t`Analyze the source material for potential conversion issues and improvements.`,
				icon: IconLineScan,
				cmp: Step2,
				cls: cls(1),
				hidden: false
			},
			{
				title: $t`Set Metadata`,
				description: $t`Set the metadata for the to be converted material.`,
				icon: IconAdjustments,
				cmp: Step3,
				cls: cls(2),
				hidden: false
			},
			{
				title: $t`Bundling`,
				description: $t`Set the volume sizes for the conversion.`,
				Icon: IconHandStop,
				cmp: Step4,
				cls: cls(3),
				hidden: true
			},
			{
				title: $t`Filter Images`,
				description: $t`Filter out images that are ads / unwanted for the conversion.`,
				icon: IconFilter,
				cmp: Step5,
				cls: cls(4),
				hidden: false
			},
			{
				title: $t`Review`,
				description: $t`Review the settings and the material before conversion.`,
				icon: IconPencil,
				cmp: Step7,
				cls: cls(6),
				hidden: false
			}
		];
	});

	let activeComponent = $derived(steps[stepState.index].cmp);
</script>

<div class="flex h-full w-full flex-col">
	<!--	<ul class="glass-surface steps steps-vertical mr-2 h-full">-->
	<!--		{#each steps as step}-->
	<!--			{#if !step.hidden}-->
	<!--				<li class="step {step.cls}">-->
	<!--					<span class="step-icon"><step.icon /></span>-->
	<!--					<span class="flex h-full w-full items-center justify-center">{step.title}</span>-->
	<!--				</li>-->
	<!--			{/if}-->
	<!--		{/each}-->
	<!--	</ul>-->
	<progress
		id="step-progress"
		class="progress progress-primary mb-2 h-5 w-full"
		max={steps.length - 1}
		value={stepState.index}
	></progress>
	<section class="glass-surface flex h-full w-full flex-col">
		<div class="mb-2 w-full">
			<h1 class="text-primary text-xl font-bold">
				{steps[stepState.index].title}
			</h1>
			<p>
				{steps[stepState.index].description}
			</p>
		</div>
		<div class="divider m-0 mb-1"></div>
		<div class="flex w-full grow flex-col items-center justify-center">
			{@render activeComponent()}
		</div>
		<div class="flex h-[10%] w-full items-center justify-between">
			<button
				class="btn btn-soft btn-error select-none"
				onclick={() => {
					stepState.index -= stepState.indexDecrement;
					stepState.indexDecrement = 1;
				}}
				disabled={stepState.disablePrev}
			>
				{$t`Previous`}
			</button>
			<button
				class="btn btn-soft btn-success select-none"
				onclick={() => {
					stepState.index += stepState.indexIncrement;
					stepState.indexIncrement = 1;
				}}
				disabled={stepState.disableNext}
			>
				{$t`Next`}
			</button>
		</div>
	</section>
</div>

<style>
	@keyframes blink {
		0% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			opacity: 1;
		}
	}

	#step-progress::-webkit-progress-value {
		background: linear-gradient(to left, var(--color-primary), var(--color-secondary));
		animation: blink 1.5s infinite;
	}

	#step-progress::-moz-progress-bar {
		background: linear-gradient(to left, var(--color-primary), var(--color-secondary));
		animation: blink 1.5s infinite;
	}
</style>
