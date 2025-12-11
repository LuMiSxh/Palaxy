<script lang="ts">
	import { onMount } from 'svelte';
	import { stepMachine } from '$states/stepMachine.svelte';
	import { stepRegistry } from '$lib/steps/registry.svelte';
	import '$lib/steps/definitions';
	import { VStack, HStack } from 'waku/layout';
	import { Button, Badge } from 'waku/components';
	import { keyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import { t } from 'svelte-i18n-lingui';
	import { appData } from '$stores/appdata';
	import type { KeyCombination } from '$types/keys';
	import {
		IconFolder,
		IconLineScan,
		IconAdjustments,
		IconHandStop,
		IconFilter,
		IconPencil,
	} from '@tabler/icons-svelte';

	// Derived state from step machine
	let currentStep = $derived(stepMachine.currentStep);
	let visibleSteps = $derived(stepRegistry.getVisibleSteps());
	let currentIndex = $derived(stepMachine.currentIndex);
	let canGoNext = $derived(stepMachine.canGoNext);
	let canGoBack = $derived(stepMachine.canGoBack);
	let isTransitioning = $derived(stepMachine.isTransitioning);

	let unregisterKeyboard: (() => void) | undefined;

	onMount(() => {
		// Reset to first step
		stepMachine.reset();

		// Register keyboard shortcuts
		unregisterKeyboard = keyboard.smartRegister([
			[
				'shift+arrowright',
				async () => {
					if (canGoNext) {
						await stepMachine.next();
					}
				},
			],
			[
				'shift+arrowleft',
				async () => {
					if (canGoBack) {
						await stepMachine.back();
					}
				},
			],
		]);

		return () => {
			if (unregisterKeyboard) unregisterKeyboard();
		};
	});

	// Update key hints based on current state
	$effect(() => {
		const hints: [KeyCombination, string][] = [['tab', $t`Navigate fields`]];

		if (canGoBack) {
			hints.push(['shift+arrowleft', $t`Previous`]);
		}

		if (canGoNext) {
			hints.push(['shift+arrowright', $t`Next`]);
		}

		return keyHint.register(hints);
	});

	// Get icon for current step
	function getCurrentIcon() {
		if (!currentStep) return IconFolder;
		const stepId = currentStep.id;

		switch (stepId) {
			case 'select-source':
				return IconFolder;
			case 'analysis':
				return IconLineScan;
			case 'metadata':
				return IconAdjustments;
			case 'bundling':
			case 'conversion':
				return IconHandStop;
			case 'filter':
				return IconFilter;
			case 'review':
				return IconPencil;
			default:
				return IconFolder;
		}
	}

	let CurrentIcon = $derived(getCurrentIcon());
</script>

<div class="flex h-full w-full flex-col overflow-hidden">
	<!-- Header Section - Sticky -->
	<div class="bg-surface-0 z-10 shrink-0 px-3 pt-4 pb-3">
		<div class="flex items-center justify-between">
			<VStack gap="xs">
				<HStack gap="sm" align="center">
					<CurrentIcon size={28} class="text-accent-500" />
					<h1 class="text-2xl font-bold">
						{currentStep?.title || ''}
					</h1>
				</HStack>
				<p class="text-muted text-sm">
					{currentStep?.description || ''}
				</p>
			</VStack>

			<Badge variant="secondary" class="text-xs">
				{$t`Step`}
				{currentIndex + 1} / {visibleSteps.length}
			</Badge>
		</div>

		<!-- Progress Bar -->
		<div class="bg-surface-2 relative mt-3 h-2 w-full overflow-hidden rounded-full">
			<div
				class="from-accent-500 to-accent-400 h-full bg-linear-to-r transition-all duration-500 ease-out"
				style="width: {stepMachine.progress}%"
			></div>
		</div>
	</div>

	<!-- Step Content -->
	<div class="flex-1 overflow-x-hidden overflow-y-auto">
		{#if currentStep?.component}
			{@const StepComponent = currentStep.component}
			<StepComponent />
		{/if}
	</div>

	<!-- Navigation Buttons -->
	{#if $appData.mouseSupport}
		<HStack justify="between" class="bg-surface-0 shrink-0 px-3 pt-2 pb-4">
			<Button
				variant="outline"
				onclick={() => stepMachine.back()}
				disabled={!canGoBack || isTransitioning}
				class="min-w-[120px]"
			>
				{$t`Previous`}
			</Button>
			<Button
				variant="primary"
				onclick={() => stepMachine.next()}
				disabled={!canGoNext || isTransitioning}
				class="min-w-[120px]"
			>
				{$t`Next`}
			</Button>
		</HStack>
	{/if}
</div>
