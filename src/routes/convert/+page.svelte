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
	import { onMount, type Snippet, untrack } from 'svelte';
	import { appData } from '$stores/appdata';
	import { keyboard } from '$lib/keyboard';
	import { addToast } from '$states/toast.svelte';
	import { keyHint } from '$states/keyhint.svelte';

	// Reset the state of the stepState
	stepState.reset();

	let steps = $derived.by(() => {
		return [
			{
				title: $t`Choose Source Material Directory`,
				description: $t`Select the directory where the source material is located.`,
				icon: IconFolder,
				cmp: Step1,
				hidden: false
			},
			{
				title: $t`Analysis`,
				description: $t`Analyze the source material for potential conversion issues and improvements.`,
				icon: IconLineScan,
				cmp: Step2,
				hidden: false
			},
			{
				title: $t`Set Metadata`,
				description: $t`Set the metadata for the to be converted material.`,
				icon: IconAdjustments,
				cmp: Step3,
				hidden: false
			},
			{
				title: $t`Bundling`,
				description: $t`Set the volume sizes for the conversion.`,
				Icon: IconHandStop,
				cmp: Step4,
				hidden: true
			},
			{
				title: $t`Filter Images`,
				description: $t`Filter out images that are ads / unwanted for the conversion.`,
				icon: IconFilter,
				cmp: Step5,
				hidden: false
			},
			{
				title: $t`Review`,
				description: $t`Review the settings and the material before conversion.`,
				icon: IconPencil,
				cmp: Step7,
				hidden: false
			}
		];
	}) as {
		title: string;
		description: string;
		icon: any;
		cmp: Snippet;
		hidden: boolean;
	}[];

	let currentStep = $derived(steps[stepState.index]);
	let activeComponent = $derived(currentStep.cmp);

	onMount(() => {
		return keyboard.smartRegister([
			['shift+arrowleft', event => {
				event.preventDefault();
				if (stepState.disablePrev) {
					addToast(
						$t`You cannot return to the previous step.`,
						'warning'
					);
					return;
				}
				stepState.index -= stepState.indexDecrement;
				stepState.indexDecrement = 1;
			}],
			['shift+arrowright', event => {
				event.preventDefault();
				if (stepState.disableNext) {
					addToast(
						$t`You cannot proceed the next step.`,
						'warning'
					);
					return;
				}
				stepState.index += stepState.indexIncrement;
				stepState.indexIncrement = 1;
			}]
		]);
	});

	function keyHintEffect() {
		if (stepState.disablePrev) {
			keyHint.removeKey('shift+arrowleft');
		} else {
			keyHint.addKey('shift+arrowleft', $t`Previous`);
		}

		if (stepState.disableNext) {
			keyHint.removeKey('shift+arrowright');
		} else {
			keyHint.addKey('shift+arrowright', $t`Next`);
		}
	}

	$effect(() => {
		untrack(() => keyHintEffect());
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		stepState.disablePrev;
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		stepState.disableNext;
	});
</script>

<div class="flex h-full w-full flex-col">
	<section class="glass-surface flex h-full w-full flex-col relative">
		<div class="mb-2 ml-2 w-full">
			<h1
				class="dark:text-transparent text-primary bg-clip-text bg-gradient-to-r from-primary via-secondary via-60% to-secondary text-3xl mb-2">
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
				class="absolute bottom-5 left-5  btn btn-soft btn-error select-none"
				onclick={() => {
					stepState.index -= stepState.indexDecrement;
					stepState.indexDecrement = 1;
				}}
				disabled={stepState.disablePrev}
			>
				{$t`Previous`}
			</button>
			<button
				class="absolute bottom-5 right-5  btn btn-soft btn-success select-none"
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
