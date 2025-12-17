<script lang="ts">
	import { onMount } from 'svelte';
	import { stepMachine } from '$states/stepMachine.svelte';
	import { stepRegistry } from '$lib/steps/registry.svelte';
	import '$lib/steps/definitions';
	import { VStack, HStack } from 'waku/layout';
	import { Button, Badge } from 'waku/components';
	import { keyHint, clearActiveKeyHint } from '$states/keyhint.svelte';
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

	// Update key hints based on current state and clear old hints when step changes
	$effect(() => {
		// Clear any active keyhints from previous step's focused elements
		clearActiveKeyHint();

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

<div class="h-full w-full overflow-y-auto p-3 md:p-6">
	<VStack gap="sm" class="mx-auto h-full">
		<!-- Header -->
		<div class="flex items-center justify-between gap-4">
			<HStack gap="sm" align="center">
				<CurrentIcon size={28} class="text-accent-500" />
				<h1 class="text-2xl font-bold">
					{currentStep?.title || ''}
				</h1>
				<div class="bg-surface-2 h-4 w-px"></div>
				<p class="text-muted text-sm">
					{currentStep?.description || ''}
				</p>
			</HStack>

			<Badge variant="neutral" class="text-xs">
				{$t`Step`}
				{currentIndex + 1} / {visibleSteps.length}
			</Badge>
		</div>

		<!-- Progress Bar -->
		<div class="bg-surface-2 relative h-2 w-full overflow-hidden rounded-full">
			<div
				class="from-accent-500 to-accent-400 h-full bg-linear-to-r transition-all duration-500 ease-out"
				style="width: {stepMachine.progress}%"
			></div>
		</div>

		<!-- Step Content with Bento Grid -->
		<div class="flex-1 overflow-y-auto">
			{#if currentStep?.component}
				{@const StepComponent = currentStep.component}
				<StepComponent />
			{/if}
		</div>

		<!-- Navigation Buttons -->
		{#if $appData.mouseSupport}
			<HStack justify="between" class="shrink-0">
				<Button
					style="outline"
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
	</VStack>
</div>
