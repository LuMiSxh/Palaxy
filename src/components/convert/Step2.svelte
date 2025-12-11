<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { t } from 'svelte-i18n-lingui';
	import convState from '$states/converter.svelte';
	import {
		IconCircleMinus,
		IconCirclePlus,
		IconExclamationCircle,
		IconAlertTriangle,
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
	<div class="flex w-full flex-col p-3 pb-3">
		<div class="mx-auto flex w-full max-w-6xl flex-col">
			{#if negatives.length > 0 || warnings.length > 0 || positives.length > 0}
				<BentoGrid cols={3} density="compact" class="flex-1">
					<!-- Summary Cards -->
					<BentoItem>
						<VStack gap="sm" align="center" class="text-center">
							<div
								class="bg-danger/10 border-danger/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
							>
								<IconCircleMinus size={32} class="text-danger" />
							</div>
							<div class="text-danger text-4xl font-bold">{negatives.length}</div>
							<span class="text-muted text-sm font-medium">{$t`Issues Found`}</span>
						</VStack>
					</BentoItem>

					<BentoItem>
						<VStack gap="sm" align="center" class="text-center">
							<div
								class="bg-warning/10 border-warning/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
							>
								<IconExclamationCircle size={32} class="text-warning" />
							</div>
							<div class="text-warning text-4xl font-bold">{warnings.length}</div>
							<span class="text-muted text-sm font-medium">{$t`Warnings`}</span>
						</VStack>
					</BentoItem>

					<BentoItem>
						<VStack gap="sm" align="center" class="text-center">
							<div
								class="bg-success/10 border-success/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
							>
								<IconCirclePlus size={32} class="text-success" />
							</div>
							<div class="text-success text-4xl font-bold">{positives.length}</div>
							<span class="text-muted text-sm font-medium">{$t`Good Points`}</span>
						</VStack>
					</BentoItem>

					<!-- Detailed Results - Scrollable -->
					<BentoItem colspan={3} rowspan={2} class="flex flex-col overflow-hidden">
						<HStack gap="sm" align="center" class="text-muted mb-3">
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Analysis Details`}</span>
							<Badge variant="secondary" class="ml-auto">
								{negatives.length + warnings.length + positives.length}
							</Badge>
						</HStack>

						<div class="custom-scrollbar flex-1 space-y-4 overflow-y-auto pr-2">
							<!-- Issues Section -->
							{#if negatives.length > 0}
								<div>
									<HStack gap="sm" align="center" class="text-muted mb-2">
										<IconCircleMinus size={16} />
										<span class="text-xs font-semibold uppercase">{$t`Issues Found`}</span>
										<Badge variant="danger" class="ml-auto text-xs">{negatives.length}</Badge>
									</HStack>

									<VStack gap="xs">
										{#each negatives as negative, i (i)}
											<div
												class="bg-danger/5 border-danger/30 hover:bg-danger/10 rounded-lg border-l-4 p-3 transition-colors"
											>
												<HStack gap="sm" align="start">
													<div class="bg-danger/20 mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full"></div>
													<span class="flex-1 text-sm leading-snug">{negative}</span>
												</HStack>
											</div>
										{/each}
									</VStack>
								</div>
							{/if}

							<!-- Warnings Section -->
							{#if warnings.length > 0}
								<div>
									<HStack gap="sm" align="center" class="text-muted mb-2">
										<IconExclamationCircle size={16} />
										<span class="text-xs font-semibold uppercase">{$t`Warnings`}</span>
										<Badge variant="warning" class="ml-auto text-xs">{warnings.length}</Badge>
									</HStack>

									<VStack gap="xs">
										{#each warnings as warning, i (i)}
											<div
												class="bg-warning/5 border-warning/30 hover:bg-warning/10 rounded-lg border-l-4 p-3 transition-colors"
											>
												<HStack gap="sm" align="start">
													<div class="bg-warning/20 mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full"></div>
													<span class="flex-1 text-sm leading-snug">{warning}</span>
												</HStack>
											</div>
										{/each}
									</VStack>
								</div>
							{/if}

							<!-- Positives Section -->
							{#if positives.length > 0}
								<div>
									<HStack gap="sm" align="center" class="text-muted mb-2">
										<IconCirclePlus size={16} />
										<span class="text-xs font-semibold uppercase">{$t`Good Points`}</span>
										<Badge variant="success" class="ml-auto text-xs">{positives.length}</Badge>
									</HStack>

									<VStack gap="xs">
										{#each positives as positive, i (i)}
											<div
												class="bg-success/5 border-success/30 hover:bg-success/10 rounded-lg border-l-4 p-3 transition-colors"
											>
												<HStack gap="sm" align="start">
													<div class="bg-success/20 mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full"></div>
													<span class="flex-1 text-sm leading-snug">{positive}</span>
												</HStack>
											</div>
										{/each}
									</VStack>
								</div>
							{/if}
						</div>
					</BentoItem>

					<!-- Error Block at Bottom -->
					{#if negatives.length > 0}
						<BentoItem colspan={3} variant="danger">
							<HStack gap="sm" align="center">
								<IconCircleMinus size={20} />
								<p class="font-medium">
									{$t`Please resolve these issues before continuing.`}
								</p>
							</HStack>
						</BentoItem>
					{/if}
				</BentoGrid>
			{:else}
				<BentoGrid cols={1} density="compact" class="flex-1">
					<BentoItem class="flex items-center justify-center">
						<VStack gap="sm" align="center" class="text-center">
							<div class="bg-info/10 flex h-14 w-14 items-center justify-center rounded-full">
								<IconAlertTriangle size={28} class="text-info" />
							</div>
							<p class="text-muted max-w-md text-sm">
								{$t`No analysis results found. There might be an issue with the source material.`}
							</p>
						</VStack>
					</BentoItem>
				</BentoGrid>
			{/if}
		</div>
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
	}

	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background-color: var(--waku-surface-1);
	}
</style>
