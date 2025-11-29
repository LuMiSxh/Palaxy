<script lang="ts">
	import {
		IconAdjustments,
		IconFilter,
		IconFolder,
		IconHandStop,
		IconLineScan,
		IconPencil,
	} from '@tabler/icons-svelte';
	import { Step1, Step2, Step3, Step4, Step5, Step6, Step7 } from '$components/convert';
	import { stepState } from '$states/converter.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { type Snippet } from 'svelte';
	import { appData } from '$stores/appdata';
	import { keyHint } from '$states/keyhint.svelte';
	import type { KeyCombination } from '$types/keys';

	// Reset the state of the stepState
	stepState.reset();

	let steps = $derived.by(() => {
		return [
			{
				title: $t`Choose Source Material Directory`,
				description: $t`Select the directory where the source material is located.`,
				icon: IconFolder,
				cmp: Step1,
			},
			{
				title: $t`Analysis`,
				description: $t`Analyze the source material for potential conversion issues and improvements.`,
				icon: IconLineScan,
				cmp: Step2,
			},
			{
				title: $t`Set Metadata`,
				description: $t`Set the metadata for the to be converted material.`,
				icon: IconAdjustments,
				cmp: Step3,
			},
			{
				title: $t`Bundling`,
				description: $t`Set the volume sizes for the conversion.`,
				Icon: IconHandStop,
				cmp: Step4,
			},
			{
				title: $t`Filter Images`,
				description: $t`Filter out unwanted images, change image order, and set cover images.`,
				icon: IconFilter,
				cmp: Step5,
			},
			{
				title: $t`Review`,
				description: $t`Review all your settings before starting the conversion.`,
				icon: IconPencil,
				cmp: Step6,
			},
			{
				title: $t`Conversion`,
				description: $t`Palaxy is now converting your material. Please wait.`,
				icon: IconHandStop,
				cmp: Step7,
				hidden: true,
			},
		];
	}) as {
		title: string;
		description: string;
		icon: any;
		cmp: Snippet;
	}[];

	let currentStep = $derived(steps[stepState.index]);
	let activeComponent = $derived(currentStep.cmp);

	$effect(() => {
		const hints: [KeyCombination, string][] = [['tab', $t`Navigate fields`]];

		if (!stepState.disablePrev) {
			hints.push(['shift+arrowleft', $t`Previous`]);
		}

		if (!stepState.disableNext) {
			hints.push(['shift+arrowright', $t`Next`]);
		}

		// register() adds these keys on a new layer.
		// Returning the result tells Svelte to run the cleanup function
		// (removing this layer) whenever dependencies change or component destroys.
		return keyHint.register(hints);
	});
</script>

<div class="flex h-full w-full flex-col">
	<section class="glass-surface relative flex h-full w-full flex-col">
		<div class="mb-2 ml-2 w-full">
			<h1
				class="text-primary from-primary via-secondary to-secondary mb-2 bg-linear-to-r via-60% bg-clip-text text-3xl dark:text-transparent"
			>
				{currentStep.title}
			</h1>
			<p class="text-lg">
				{currentStep.description}
			</p>
		</div>
		<progress
			id="step-progress"
			class="progress h-5 w-full"
			max={steps.length - 1}
			value={stepState.index}
		></progress>
		<div class="flex w-full grow flex-col items-center justify-center">
			{@render activeComponent()}
		</div>
		{#if $appData.mouseSupport}
			<button
				class="btn btn-error absolute bottom-5 left-5 shadow-lg select-none"
				onclick={() => {
					stepState.index -= stepState.indexDecrement;
					stepState.indexDecrement = 1;
				}}
				disabled={stepState.disablePrev}
			>
				{$t`Previous`}
			</button>
			<button
				class="btn btn-success absolute right-5 bottom-5 shadow-lg select-none"
				onclick={() => {
					stepState.index += stepState.indexIncrement;
					stepState.indexIncrement = 1;
				}}
				disabled={stepState.disableNext}
			>
				{$t`Next`}
			</button>
		{/if}
	</section>
</div>

<style>
	@keyframes blink {
		0% {
			opacity: 1;
		}
		50% {
			opacity: 0.8;
		}
		100% {
			opacity: 1;
		}
	}

	#step-progress {
		transition: value 0.5s ease;
	}

	#step-progress::-webkit-progress-value {
		background: linear-gradient(to left, var(--color-primary), var(--color-secondary));
		animation: blink 1.5s infinite;
		transition: width 0.5s ease-in-out;
	}

	#step-progress::-moz-progress-bar {
		background: linear-gradient(to left, var(--color-primary), var(--color-secondary));
		animation: blink 1.5s infinite;
		transition: width 0.5s ease-in-out;
	}
</style>
