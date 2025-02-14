<script lang="ts">
	import {
		IconAdjustments,
		IconFilter,
		IconFolder,
		IconHandStop,
		IconLineScan,
		IconPencil
	} from '@tabler/icons-svelte';
	import { Step1, Step2, Step3, Step4, Step5, Step6, Step7 } from '$components/convert';
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
				title: $t`Choose Base Directory`,
				icon: IconFolder,
				cmp: Step1,
				cls: cls(0),
				hidden: false
			},
			{
				title: $t`Analysis`,
				icon: IconLineScan,
				cmp: Step2,
				cls: cls(1),
				hidden: false
			},
			{
				title: $t`Set Metadata`,
				icon: IconAdjustments,
				cmp: Step3,
				cls: cls(2),
				hidden: false
			},
			{
				title: $t`Manual Bundling`,
				Icon: IconHandStop,
				cmp: Step4,
				cls: cls(3),
				hidden: true
			},
			{
				title: $t`Filter Pages`,
				icon: IconFilter,
				cmp: Step5,
				cls: cls(4),
				hidden: false
			},
			{
				title: $t`Choose Conversion Options`,
				icon: IconAdjustments,
				cmp: Step6,
				cls: cls(5),
				hidden: false
			},
			{
				title: $t`Review`,
				icon: IconPencil,
				cmp: Step7,
				cls: cls(6),
				hidden: false
			}
		];
	});

	let activeComponent = $derived(steps[stepState.index].cmp);
</script>

<div class="flex h-full w-full">
	<ul class="glass-surface steps steps-vertical mr-2 h-full">
		{#each steps as step}
			{#if !step.hidden}
				<li class="step {step.cls}">
					<span class="step-icon"><step.icon /></span>
					<span class="flex h-full w-full items-center justify-center">{step.title}</span>
				</li>
			{/if}
		{/each}
	</ul>
	<section class="glass-surface ml-2 h-full w-full">
		<div class="flex h-[90%] w-full flex-col items-center justify-center">
			{@render activeComponent()}
		</div>
		<div class="flex h-[10%] w-full items-center justify-between">
			<button
				class="btn btn-soft btn-error"
				onclick={() => {stepState.index-=stepState.indexDecrement; stepState.indexDecrement = 1}}
				disabled={stepState.disablePrev}
			>
				{$t`Previous`}
			</button>
			<button
				class="btn btn-soft btn-success"
				onclick={() => {stepState.index+=stepState.indexIncrement; stepState.indexIncrement = 1}}
				disabled={stepState.disableNext}
			>
				{$t`Next`}
			</button>
		</div>
	</section>
</div>
