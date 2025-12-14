<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { t } from 'svelte-i18n-lingui';
	import convState from '$states/converter.svelte';
	import {
		IconCircleMinus,
		IconCirclePlus,
		IconExclamationCircle,
		IconAlertTriangle,
		IconChecklist,
	} from '@tabler/icons-svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { LoadingSpinner, Badge } from 'waku/components';

	let positives: string[] = $state([]);
	let negatives: string[] = $state([]);
	let warnings: string[] = $state([]);
	let isLoading: boolean = $state(true);
	let cleanupKeyHint: (() => void) | null = null;

	onMount(async () => {
		const result = await wrapper(commands.convAnalyze());
		isLoading = false;

		if (result !== null && result.payload !== null) {
			convState.bundleRecommendation = result.payload.flag;
			positives = result.payload.positive;
			negatives = result.payload.negative;
			warnings = result.payload.warning;

			// Store in convState for validation
			convState.analysisNegatives = negatives;
			convState.analysisWarnings = warnings;
			convState.analysisPositives = positives;
		}

		// Show the key hint for navigating
		cleanupKeyHint = keyHint.register([
			['arrowdown', $t`Scroll down`],
			['arrowup', $t`Scroll up`],
		]);
	});

	onDestroy(() => {
		if (cleanupKeyHint) {
			cleanupKeyHint();
		}
	});
</script>

{#if isLoading}
	<div class="flex h-full w-full items-center justify-center p-3">
		<VStack gap="sm" align="center">
			<LoadingSpinner size="lg" />
			<span class="text-muted text-sm">{$t`Analyzing source material...`}</span>
		</VStack>
	</div>
{:else}
	<div class="h-full w-full p-3">
		{#if negatives.length > 0 || warnings.length > 0 || positives.length > 0}
			<BentoGrid cols={3} density="comfortable" class="h-full auto-rows-min">
				<!-- Summary Cards Row -->
				<BentoItem
					variant="glass"
					class="relative flex max-h-24 items-center justify-center overflow-hidden"
				>
					<div class="relative z-10 flex w-full items-center justify-between px-2">
						<HStack gap="md" align="center">
							<div
								class="bg-danger/10 border-danger/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border"
							>
								<IconCircleMinus size={20} class="text-danger" />
							</div>
							<VStack gap="0">
								<span class="text-muted text-xs font-medium tracking-wide uppercase"
									>{$t`Issues`}</span
								>
								<div class="text-danger text-2xl leading-tight font-bold">{negatives.length}</div>
							</VStack>
						</HStack>
					</div>
					{#if negatives.length > 0}
						<div
							class="from-danger/5 absolute inset-0 bg-linear-to-br to-transparent opacity-50"
						></div>
					{/if}
				</BentoItem>

				<BentoItem
					variant="glass"
					class="relative flex max-h-24 items-center justify-center overflow-hidden"
				>
					<div class="relative z-10 flex w-full items-center justify-between px-2">
						<HStack gap="md" align="center">
							<div
								class="bg-warning/10 border-warning/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border"
							>
								<IconExclamationCircle size={20} class="text-warning" />
							</div>
							<VStack gap="0">
								<span class="text-muted text-xs font-medium tracking-wide uppercase"
									>{$t`Warnings`}</span
								>
								<div class="text-warning text-2xl leading-tight font-bold">{warnings.length}</div>
							</VStack>
						</HStack>
					</div>
					{#if warnings.length > 0}
						<div
							class="from-warning/5 absolute inset-0 bg-linear-to-br to-transparent opacity-50"
						></div>
					{/if}
				</BentoItem>

				<BentoItem
					variant="glass"
					class="relative flex max-h-24 items-center justify-center overflow-hidden"
				>
					<div class="relative z-10 flex w-full items-center justify-between px-2">
						<HStack gap="md" align="center">
							<div
								class="bg-success/10 border-success/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border"
							>
								<IconCirclePlus size={20} class="text-success" />
							</div>
							<VStack gap="0">
								<span class="text-muted text-xs font-medium tracking-wide uppercase"
									>{$t`Good`}</span
								>
								<div class="text-success text-2xl leading-tight font-bold">{positives.length}</div>
							</VStack>
						</HStack>
					</div>
					{#if positives.length > 0}
						<div
							class="from-success/5 absolute inset-0 bg-linear-to-br to-transparent opacity-50"
						></div>
					{/if}
				</BentoItem>

				<!-- Detailed Results Section -->
				<BentoItem
					colspan={3}
					variant="glass"
					class="row-span-2 flex min-h-0 flex-col overflow-hidden"
				>
					<HStack gap="sm" align="center" class="text-muted mb-4 shrink-0">
						<IconChecklist size={20} />
						<span class="text-sm font-bold tracking-wider uppercase">{$t`Analysis Details`}</span>
						<Badge variant="secondary" class="ml-auto">
							{negatives.length + warnings.length + positives.length}
						</Badge>
					</HStack>

					<div class="custom-scrollbar flex-1 space-y-6 overflow-y-auto pr-2">
						<!-- Issues Section -->
						{#if negatives.length > 0}
							<div>
								<HStack gap="sm" align="center" class="mb-3">
									<IconCircleMinus size={18} class="text-danger" />
									<span class="font-semibold">{$t`Critical Issues`}</span>
									<Badge variant="danger" class="ml-auto text-xs">{negatives.length}</Badge>
								</HStack>

								<ul class="space-y-2">
									{#each negatives as negative, i (i)}
										<li
											class="bg-danger/5 hover:bg-danger/10 group flex gap-3 rounded-lg p-3 transition-colors"
										>
											<div
												class="bg-danger mt-1 h-1.5 w-1.5 shrink-0 rounded-full opacity-60"
											></div>
											<span class="text-sm leading-relaxed">{negative}</span>
										</li>
									{/each}
								</ul>
							</div>
						{/if}

						<!-- Warnings Section -->
						{#if warnings.length > 0}
							<div>
								<HStack gap="sm" align="center" class="mb-3">
									<IconExclamationCircle size={18} class="text-warning" />
									<span class="font-semibold">{$t`Warnings`}</span>
									<Badge variant="warning" class="ml-auto text-xs">{warnings.length}</Badge>
								</HStack>

								<ul class="space-y-2">
									{#each warnings as warning, i (i)}
										<li
											class="bg-warning/5 hover:bg-warning/10 group flex gap-3 rounded-lg p-3 transition-colors"
										>
											<div
												class="bg-warning mt-1 h-1.5 w-1.5 shrink-0 rounded-full opacity-60"
											></div>
											<span class="text-sm leading-relaxed">{warning}</span>
										</li>
									{/each}
								</ul>
							</div>
						{/if}

						<!-- Positives Section -->
						{#if positives.length > 0}
							<div>
								<HStack gap="sm" align="center" class="mb-3">
									<IconCirclePlus size={18} class="text-success" />
									<span class="font-semibold">{$t`Quality Indicators`}</span>
									<Badge variant="success" class="ml-auto text-xs">{positives.length}</Badge>
								</HStack>

								<ul class="space-y-2">
									{#each positives as positive, i (i)}
										<li
											class="bg-success/5 hover:bg-success/10 group flex gap-3 rounded-lg p-3 transition-colors"
										>
											<div
												class="bg-success mt-1 h-1.5 w-1.5 shrink-0 rounded-full opacity-60"
											></div>
											<span class="text-sm leading-relaxed">{positive}</span>
										</li>
									{/each}
								</ul>
							</div>
						{/if}
					</div>
				</BentoItem>

				<!-- Error Block at Bottom -->
				{#if negatives.length > 0}
					<BentoItem colspan={3} variant="danger" class="max-h-24">
						<HStack gap="sm" align="center">
							<div class="bg-danger/20 flex h-10 w-10 items-center justify-center rounded-full">
								<IconAlertTriangle size={20} class="text-danger" />
							</div>
							<VStack gap="xs" class="flex-1">
								<p class="font-semibold">{$t`Action Required`}</p>
								<p class="text-sm opacity-90">
									{$t`Please resolve the critical issues above before continuing with the conversion process.`}
								</p>
							</VStack>
						</HStack>
					</BentoItem>
				{/if}
			</BentoGrid>
		{:else}
			<BentoGrid cols={1} density="comfortable" class="h-full">
				<BentoItem variant="glass" class="flex items-center justify-center">
					<VStack gap="md" align="center" class="text-center">
						<div
							class="bg-info/10 border-info/20 flex h-20 w-20 items-center justify-center rounded-full border-2"
						>
							<IconAlertTriangle size={40} class="text-info" />
						</div>
						<VStack gap="sm" align="center">
							<h3 class="text-lg font-semibold">{$t`No Analysis Results`}</h3>
							<p class="text-muted max-w-md text-sm">
								{$t`No analysis results found. There might be an issue with the source material or the analysis process.`}
							</p>
						</VStack>
					</VStack>
				</BentoItem>
			</BentoGrid>
		{/if}
	</div>
{/if}

<style>
	/* Custom scrollbar */
	.custom-scrollbar {
		scrollbar-width: thin;
		scrollbar-color: var(--waku-surface-2) transparent;
	}

	.custom-scrollbar::-webkit-scrollbar {
		width: 8px;
	}

	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb {
		background-color: var(--waku-surface-2);
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background-color: var(--waku-surface-1);
	}
</style>
