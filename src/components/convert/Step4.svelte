<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { IconFileZip, IconRefresh, IconSettings, IconAdjustments } from '@tabler/icons-svelte';
	import convState from '$states/converter.svelte';
	import { commands, type BundleFlag } from '$types';
	import { addToast } from '$states/toast.svelte';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';
	import { t, plural } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import ManualBundling from '$components/convert/step4/ManualBundling.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Badge, LoadingSpinner, Button, Select } from 'waku/components';

	let unregisterKeyHint: () => void;
	let bundleFlag: BundleFlag = $state(
		convState.bundle ? convState.bundle : convState.bundleRecommendation
	);

	const bundleOptions = $derived([
		{ value: 'MANUAL', label: $t`Manual` },
		{ value: 'IMAGE', label: $t`Image Detection` },
		...(convState.bundleRecommendation !== 'MANUAL'
			? [{ value: 'NAME', label: $t`Name Detection` }]
			: []),
	]);

	async function runBundler() {
		step4State.loading = true;

		// Save bundle type first
		convState.bundle = bundleFlag;
		await wrapper(commands.convStateSet({ BundleFlag: bundleFlag }));

		// Wait for state update
		await tick();
		const result = await wrapper(commands.convBundle(step4State.sensibility));

		step4State.loading = false;

		if (result !== null && result.payload !== null) {
			step4State.result = result.payload;
			convState.chapterSizes = result.payload.chapter_sizes ?? [];

			// If image/name bundling detected no volumes, switch to manual
			if (convState.chapterSizes.length === 0 && convState.bundle !== 'MANUAL') {
				addToast($t`No volumes detected. Switching to manual bundling.`, 'warning');
				bundleFlag = 'MANUAL';
				convState.bundle = 'MANUAL';
			}
		} else {
			// Handle the error case
			addToast($t`Failed to run bundler. Please try again.`, 'error');
		}
	}

	function switchToBundleType(type: BundleFlag) {
		bundleFlag = type;
	}

	// Watch for bundle type changes
	$effect(() => {
		if (bundleFlag !== convState.bundle) {
			convState.bundle = bundleFlag;
			runBundler();
		}
	});

	onMount(async () => {
		unregisterKeyHint = keyHint.register([['tab', $t`Navigate fields`]]);

		// If bundle type already set, run bundler
		if (convState.bundle) {
			bundleFlag = convState.bundle;
			await runBundler();
		}
	});

	onDestroy(async () => {
		if (unregisterKeyHint) unregisterKeyHint();

		// Save chapter sizes and bundle type
		await wrapper(commands.convStateSet({ BundleFlag: bundleFlag }));
		await wrapper(commands.convStateSet({ VolumeSizes: convState.chapterSizes }));
	});

	function getChaptersPercentage(): number {
		const total = step4State.result?.total_chapters ?? 0;
		if (total === 0) return 0;
		const used = convState.chapterSizes.reduce((sum, size) => sum + size, 0);
		return Math.min((used / total) * 100, 100);
	}

	function getTotalChapters(): number {
		return convState.chapterSizes.reduce((sum, size) => sum + size, 0);
	}
</script>

<div class="h-full p-3 pb-3">
	{#if step4State.loading}
		<div class="flex w-full items-center justify-center py-12">
			<VStack gap="sm" align="center">
				<LoadingSpinner size="lg" />
				<span class="text-muted text-sm">{$t`Running Bundler...`}</span>
			</VStack>
		</div>
	{:else if bundleFlag === 'MANUAL'}
		<ManualBundling />
	{:else}
		<!-- Automatic/Image Detection Layout -->
		<div class="mx-auto max-w-7xl">
			<BentoGrid cols={2} density="compact">
				<!-- Left Column: Controls -->
				<BentoItem class="flex flex-col gap-4">
					<!-- Bundle Type Selection -->
					<div>
						<HStack gap="sm" align="center" class="text-muted mb-3">
							<IconSettings size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Bundle Type`}</span>
						</HStack>

						<VStack gap="sm">
							<Select
								id="bundle-type"
								options={bundleOptions}
								bind:value={bundleFlag}
								variant="seamless"
							/>
							<p class="text-muted text-xs">
								{#if bundleFlag === 'IMAGE'}
									{$t`Detect volumes using image analysis`}
								{:else}
									{$t`Detect volumes from file/folder names`}
								{/if}
							</p>
						</VStack>

						{#if bundleFlag === 'IMAGE' && convState.bundleRecommendation === 'MANUAL'}
							<div class="bg-warning/10 border-warning/30 mt-3 rounded-lg border p-3">
								<HStack gap="sm" align="start">
									<div class="text-warning mt-0.5 shrink-0">⚠</div>
									<VStack gap="xs">
										<span class="text-sm font-medium">{$t`Image detection unavailable`}</span>
										<span class="text-muted text-xs">
											{$t`Source doesn't support image-based detection. Use Manual or Name detection instead.`}
										</span>
									</VStack>
								</HStack>
							</div>
						{/if}
					</div>

					<!-- Detection Results -->
					<div>
						<HStack gap="sm" align="center" class="text-muted mb-3">
							<IconFileZip size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Detection Results`}</span
							>
						</HStack>

						<VStack gap="sm">
							<HStack justify="between" align="center" class="bg-surface-2 rounded-lg p-3">
								<HStack gap="sm" align="center">
									<div
										class="bg-accent-500/10 flex h-10 w-10 items-center justify-center rounded-full"
									>
										<IconFileZip size={20} class="text-accent-500" />
									</div>
									<span class="text-sm font-medium">{$t`Total Chapters`}</span>
								</HStack>
								<Badge variant="primary" class="text-lg"
									>{step4State.result?.total_chapters ?? 0}</Badge
								>
							</HStack>

							<HStack justify="between" align="center" class="bg-surface-2 rounded-lg p-3">
								<HStack gap="sm" align="center">
									<div
										class="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10"
									>
										<IconFileZip size={20} class="text-success" />
									</div>
									<span class="text-sm font-medium">{$t`Detected Volumes`}</span>
								</HStack>
								<Badge variant="success" class="text-lg"
									>{step4State.result?.total_volumes ?? 0}</Badge
								>
							</HStack>

							<!-- Chapter Usage Progress -->
							{#if step4State.result && step4State.result.total_chapters > 0}
								<div class="bg-surface-2 rounded-lg p-3">
									<HStack justify="between" align="center" class="mb-2">
										<span class="text-sm font-medium">{$t`Chapter Usage`}</span>
										<span class="font-mono text-sm">
											{getTotalChapters()}/{step4State.result.total_chapters}
										</span>
									</HStack>
									<div class="bg-surface-0 h-2 w-full overflow-hidden rounded-full">
										<div
											class="h-full rounded-full transition-all duration-300"
											class:bg-success={getTotalChapters() === step4State.result.total_chapters}
											class:bg-warning={getTotalChapters() < step4State.result.total_chapters}
											class:bg-danger={getTotalChapters() > step4State.result.total_chapters}
											style="width: {getChaptersPercentage()}%"
										></div>
									</div>
								</div>
							{/if}
						</VStack>
					</div>

					<!-- Image Sensibility (Only for IMAGE mode) -->
					{#if bundleFlag === 'IMAGE'}
						<div>
							<HStack gap="sm" align="center" class="text-muted mb-3">
								<IconAdjustments size={18} />
								<span class="text-xs font-bold tracking-wider uppercase"
									>{$t`Image Sensitivity`}</span
								>
							</HStack>

							<VStack gap="sm">
								<HStack justify="between" align="center">
									<span class="text-sm">{$t`Grayscale Threshold`}</span>
									<Badge variant="secondary" class="font-mono text-xs">
										{step4State.sensibility}%
									</Badge>
								</HStack>
								<input
									type="range"
									min="0"
									max="100"
									step="5"
									class="range-accent w-full"
									bind:value={step4State.sensibility}
									use:handleKeyHint={{
										keys: [
											['arrowleft', $t`Decrease`],
											['arrowright', $t`Increase`],
										],
									}}
								/>
								<HStack justify="between" class="text-muted flex justify-between!">
									<span class="text-xs">0%</span>
									<span class="ml-4 text-xs">50%</span>
									<span class="text-xs">100%</span>
								</HStack>
								<p class="text-muted text-xs">
									{$t`Adjust sensitivity for detecting volume boundaries in images`}
								</p>
							</VStack>
						</div>
					{/if}

					<!-- Actions -->
					<div>
						<HStack gap="sm" align="center" class="text-muted mb-3">
							<IconRefresh size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Actions`}</span>
						</HStack>

						<VStack gap="sm">
							<Button
								variant="primary"
								onclick={runBundler}
								class="w-full"
								data-keyhint={`enter;${$t`Rerun`}`}
							>
								<HStack gap="sm" align="center">
									<IconRefresh size={18} />
									<span>{$t`Rerun Detection`}</span>
								</HStack>
							</Button>
						</VStack>
					</div>
				</BentoItem>

				<!-- Right Column: Volume Visualization -->
				<BentoItem rowspan={2} class="flex flex-col">
					<HStack gap="sm" align="center" class="text-muted mb-4 shrink-0">
						<IconFileZip size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Distribution`}</span
						>
					</HStack>

					{#if convState.chapterSizes.length > 0}
						<div class="custom-scrollbar max-h-[600px] space-y-3 overflow-y-auto pr-2">
							{#each convState.chapterSizes as chapters, i}
								{@const totalChapters = step4State.result?.total_chapters || 1}
								{@const previousChapters = convState.chapterSizes
									.slice(0, i)
									.reduce((sum, c) => sum + c, 0)}
								{@const currentPercentage = (chapters / totalChapters) * 100}

								<div class="bg-surface-2 group hover:bg-surface-1 rounded-lg p-4 transition-all">
									<HStack justify="between" align="center" class="mb-3">
										<HStack gap="sm" align="center">
											<div
												class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {i %
													2 ===
												0
													? 'bg-accent-500/20'
													: 'bg-accent-400/20'}"
											>
												<IconFileZip size={20} class="text-accent-500" />
											</div>
											<VStack gap="xs">
												<span class="text-sm font-semibold">
													{$t({ message: 'Volume {vol}', values: { vol: i + 1 } })}
												</span>
												<span class="text-muted text-xs">
													{$plural(chapters, {
														one: '# chapter',
														other: '# chapters',
													})}
												</span>
											</VStack>
										</HStack>
										<Badge variant="primary" class="font-mono">
											{currentPercentage.toFixed(1)}%
										</Badge>
									</HStack>

									<!-- Chapter Blocks -->
									<div class="mb-3 flex flex-wrap gap-1">
										{#each Array(Math.min(chapters, 50)) as _, j}
											<div
												class="bg-accent-500 h-4 w-4 rounded-sm transition-all hover:scale-110"
												style="opacity: {0.6 + (j / chapters) * 0.4}"
												title={$t({
													message: 'Chapter {chap}',
													values: { chap: previousChapters + j + 1 },
												})}
											></div>
										{/each}
										{#if chapters > 50}
											<div
												class="text-muted flex h-4 items-center text-xs"
												title={$t({
													message: '+{more} more chapters',
													values: { more: chapters - 50 },
												})}
											>
												+{chapters - 50}
											</div>
										{/if}
									</div>

									<!-- Progress Bar -->
									<div class="bg-surface-0 h-2 w-full overflow-hidden rounded-full">
										<div
											class="bg-accent-500 h-full rounded-full transition-all"
											style="width: {Math.min(currentPercentage, 100)}%"
										></div>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div
							class="border-waku-border/50 flex min-h-[400px] items-center justify-center rounded-lg border border-dashed p-8"
						>
							<VStack gap="sm" align="center" class="text-center">
								<div class="bg-surface-2 flex h-16 w-16 items-center justify-center rounded-full">
									<IconFileZip size={32} class="text-muted" />
								</div>
								<p class="text-muted max-w-xs text-sm">
									{$t`No volumes detected. Try adjusting settings and rerunning the detection.`}
								</p>
							</VStack>
						</div>
					{/if}
				</BentoItem>
			</BentoGrid>
		</div>
	{/if}
</div>

<style>
	/* Custom range slider styling */
	.range-accent {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 8px;
		border-radius: var(--radius-sm);
		background: var(--waku-surface-2);
		outline: none;
		cursor: pointer;
		position: relative;
	}

	/* Webkit (Chrome, Safari, Edge) */
	.range-accent::-webkit-slider-track {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 8px;
		border-radius: var(--radius-sm);
		background: var(--waku-surface-2);
		cursor: pointer;
	}

	.range-accent::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--accent-500);
		cursor: pointer;
		transition: var(--transition-fast);
		border: 3px solid var(--waku-surface-1);
		box-shadow: var(--shadow-md);
	}

	.range-accent::-webkit-slider-thumb:hover {
		transform: scale(1.15);
		box-shadow:
			var(--shadow-lg),
			0 0 0 8px oklch(from var(--waku-accent) l c h / 0.15);
	}

	.range-accent::-webkit-slider-thumb:active {
		transform: scale(1.05);
		box-shadow: var(--shadow-sm);
	}

	/* Firefox */
	.range-accent::-moz-range-track {
		width: 100%;
		height: 8px;
		border-radius: var(--radius-sm);
		background: var(--waku-surface-2);
		cursor: pointer;
		border: none;
	}

	.range-accent::-moz-range-thumb {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--accent-500);
		border: 3px solid var(--waku-surface-1);
		cursor: pointer;
		transition: var(--transition-fast);
		box-shadow: var(--shadow-md);
	}

	.range-accent::-moz-range-thumb:hover {
		transform: scale(1.15);
		box-shadow:
			var(--shadow-lg),
			0 0 0 8px oklch(from var(--waku-accent) l c h / 0.15);
	}

	.range-accent::-moz-range-thumb:active {
		transform: scale(1.05);
		box-shadow: var(--shadow-sm);
	}

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
