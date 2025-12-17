<script lang="ts">
	import { IconFileZip, IconRefresh, IconSettings } from '@tabler/icons-svelte';
	import convState from '$states/converter.svelte';
	import { commands, type BundleFlag } from '$types';
	import { addToast } from '$states/toast.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import { tick, onMount } from 'svelte';
	import { BentoGrid, BentoItem, VStack, HStack } from 'waku/layout';
	import { Badge, Button, Select } from 'waku/components';
	import VolumeVisualisation from './VolumeVisualisation.svelte';
	import { keyboard } from '$lib/keyboard';

	let volumeVisualisationContainer: HTMLDivElement | null = $state(null);
	let volumeVizBentoFocused = $state(false);

	onMount(() => {
		const cleanup = keyboard.smartRegister([
			[
				'arrowdown',
				(event) => {
					if (volumeVizBentoFocused && volumeVisualisationContainer) {
						event.preventDefault();
						volumeVisualisationContainer.scrollTop += 40;
					}
				},
			],
			[
				'arrowup',
				(event) => {
					if (volumeVizBentoFocused && volumeVisualisationContainer) {
						event.preventDefault();
						volumeVisualisationContainer.scrollTop -= 40;
					}
				},
			],
		]);

		return cleanup;
	});

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

		convState.bundle = bundleFlag;
		await wrapper(commands.convStateSet({ BundleFlag: bundleFlag }));

		await tick();
		const result = await wrapper(commands.convBundle(step4State.sensibility));

		step4State.loading = false;

		if (result !== null && result.payload !== null) {
			step4State.result = result.payload;
			convState.chapterSizes = result.payload.chapter_sizes ?? [];

			if (convState.chapterSizes.length === 0 && convState.bundle !== 'MANUAL') {
				addToast($t`No volumes detected. Switching to manual bundling.`, 'warning');
				bundleFlag = 'MANUAL';
				convState.bundle = 'MANUAL';
			}
		} else {
			addToast($t`Failed to run bundler. Please try again.`, 'error');
		}
	}

	$effect(() => {
		if (bundleFlag !== convState.bundle) {
			convState.bundle = bundleFlag;
			runBundler();
		}
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={3} density="comfortable" rows="auto 1fr auto" class="h-full">
		<!-- Summary Cards Row -->
		<BentoItem glass>
			<HStack gap="md" align="center" justify="between">
				<HStack gap="md" align="center">
					<div
						class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconFileZip size={20} class="text-accent-500" />
					</div>
					<VStack gap="none">
						<span class="text-muted text-xs font-medium tracking-wide uppercase"
							>{$t`Detected`}</span
						>
						<div class="text-2xl leading-tight font-bold">
							{step4State.result?.total_chapters ?? 0}
						</div>
					</VStack>
				</HStack>
			</HStack>
		</BentoItem>

		<BentoItem glass>
			<HStack gap="md" align="center" justify="between">
				<HStack gap="md" align="center">
					<div
						class="bg-success/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconFileZip size={20} class="text-success" />
					</div>
					<VStack gap="none">
						<span class="text-muted text-xs font-medium tracking-wide uppercase">{$t`Volumes`}</span
						>
						<div class="text-success text-2xl leading-tight font-bold">
							{step4State.result?.total_volumes ?? 0}
						</div>
					</VStack>
				</HStack>
			</HStack>
		</BentoItem>

		<!-- Detection Method -->
		<BentoItem glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconSettings size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Detection Method`}</span>
			</HStack>

			<VStack gap="sm">
				<Select id="bundle-type" options={bundleOptions} bind:value={bundleFlag} style="seamless" />
				<p class="text-muted text-xs">
					{#if bundleFlag === 'IMAGE'}
						{$t`Detect volumes using image analysis`}
					{:else}
						{$t`Detect volumes from file/folder names`}
					{/if}
				</p>

				{#if bundleFlag === 'IMAGE' && convState.bundleRecommendation === 'MANUAL'}
					<div class="bg-warning/10 border-warning/30 rounded-lg border p-3">
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
			</VStack>
		</BentoItem>

		<!-- Main Content Area -->
		<BentoItem colspan={2} glass class="flex min-h-0 flex-col">
			{#if bundleFlag === 'IMAGE'}
				<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
					<IconSettings size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Image Sensitivity`}</span>
				</HStack>

				<VStack gap="sm">
					<HStack justify="between" align="center">
						<span class="text-sm">{$t`Grayscale Threshold`}</span>
						<Badge variant="neutral" class="font-mono text-xs">
							{step4State.sensibility}%
						</Badge>
					</HStack>
					<input
						id="range"
						type="range"
						min="0"
						max="100"
						step="5"
						tabindex={0}
						class="range-accent w-full"
						bind:value={step4State.sensibility}
						data-keyhint={`arrowleft;${$t`Adjust sensitivity`}|arrowright;${$t`Adjust sensitivity`}`}
					/>
					<HStack justify="between" class="text-muted">
						<span class="text-xs">0%</span>
						<span class="text-xs">50%</span>
						<span class="text-xs">100%</span>
					</HStack>
					<p class="text-muted text-xs">
						{$t`Adjust sensitivity for detecting volume boundaries in images`}
					</p>
				</VStack>
			{:else}
				<div class="flex h-full items-center justify-center">
					<VStack gap="sm" align="center" class="text-center">
						<IconFileZip size={48} class="text-muted" opacity={0.5} />
						<p class="text-muted max-w-xs text-sm">
							{$t`Volumes detected from file and folder names`}
						</p>
					</VStack>
				</div>
			{/if}
		</BentoItem>

		<!-- Volume Visualization -->
		<BentoItem
			glass
			rowspan={2}
			class="flex flex-col"
			onclick={() => {}}
			onfocus={() => (volumeVizBentoFocused = true)}
			onblur={() => (volumeVizBentoFocused = false)}
			data-keyhint={`arrowup;${$t`Scroll volumes`}|arrowdown;${$t`Scroll volumes`}`}
		>
			<VolumeVisualisation bind:scrollContainer={volumeVisualisationContainer} />
		</BentoItem>

		<!-- Rerun Button Section -->
		<BentoItem colspan={2} glass onclick={runBundler} data-keyhint={`enter;${$t`Rerun`}`}>
			<HStack gap="sm" align="center">
				<div
					class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
				>
					<IconRefresh size={20} class="text-accent-500" />
				</div>
				<VStack gap="xs" class="flex-1">
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Redetect Volumes`}</span>
					<HStack gap="sm">
						<Button
							variant="primary"
							onclick={runBundler}
							tabindex={-1}
							style="seamless"
							class="w-full"
						>
							<HStack gap="sm" align="center">
								<IconRefresh size={18} />
								<span>{$t`Rerun Detection`}</span>
							</HStack>
						</Button>
					</HStack>
				</VStack>
			</HStack>
		</BentoItem>
	</BentoGrid>
</div>

<style>
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

	.range-accent:focus {
		outline: none;
	}

	.range-accent:focus-visible {
		outline: none;
	}

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

	.range-accent:focus::-webkit-slider-thumb,
	.range-accent:focus-visible::-webkit-slider-thumb {
		transform: scale(1.1);
		border-width: 2px;
		box-shadow:
			var(--shadow-lg),
			0 0 0 0px var(--waku-surface-1),
			0 0 0 4px var(--accent-500),
			0 0 0 6px oklch(from var(--waku-accent) l c h / 0.3);
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

	.range-accent:focus::-moz-range-thumb,
	.range-accent:focus-visible::-moz-range-thumb {
		transform: scale(1.1);
		border-width: 2px;
		box-shadow:
			var(--shadow-lg),
			0 0 0 0px var(--waku-surface-1),
			0 0 0 4px var(--accent-500),
			0 0 0 6px oklch(from var(--waku-accent) l c h / 0.3);
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
</style>
