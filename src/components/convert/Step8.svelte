<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import {
		IconCheck,
		IconClock,
		IconAlertCircle,
		IconLoader,
		IconCircleCheck,
		IconCircleX,
		IconCircle,
		IconBolt,
		IconChartPie,
	} from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { commands } from '$types';
	import { keyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import { useConversionProgress } from '$lib/useConversionProgress.svelte';
	import type { StatusMessage } from '$states/conversion.svelte';
	import Confetti from '$components/Confetti.svelte';
	import { fly } from 'svelte/transition';

	// Waku Imports
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Button, Badge } from 'waku/components';

	const conversionProgress = useConversionProgress();

	let unregisterKey: () => void;
	let showConfetti = $state(false);
	let isComplete = $state(false);

	function formatTime(seconds: number): string {
		if (seconds < 60) {
			return `${seconds.toFixed(1)} ${$t`seconds`}`;
		} else {
			const minutes = Math.floor(seconds / 60);
			const remainingSeconds = seconds % 60;
			return `${minutes} ${minutes === 1 ? $t`minute` : $t`minutes`} ${remainingSeconds.toFixed(0)} ${$t`seconds`}`;
		}
	}

	function getStatusClass(status: string) {
		switch (status) {
			case 'completed':
				return 'text-success';
			case 'failed':
				return 'text-error';
			case 'processing':
				return 'text-accent-500';
			default:
				return 'opacity-30';
		}
	}

	async function startConversion() {
		const result = await wrapper(commands.convConvert());

		if (result != null) {
			isComplete = true;
			await tick();
			if (conversionProgress.completed.failed === 0) {
				setTimeout(() => {
					showConfetti = true;
				}, 100);
			}
			keyHint.addKey('enter', $t`Convert another manga`);
		}
	}

	function resetConversion() {
		conversionProgress.reset();
		isComplete = false;
		showConfetti = false;
	}

	onMount(async () => {
		unregisterKey = keyboard.smartRegister([
			[
				'enter',
				() => {
					if (isComplete) {
						resetConversion();
					}
				},
			],
		]);

		await startConversion();
	});

	onDestroy(() => {
		if (unregisterKey) unregisterKey();
		keyHint.removeKey('enter');
	});

	let overallProgress = $derived(conversionProgress.globalProgress);

	function formatStatusMessage(message: StatusMessage): string {
		switch (message.type) {
			case 'volume_started':
				return $t({
					message: 'Starting Volume {volumeIdx}: {volName}',
					values: { volumeIdx: message.volumeIndex + 1, volName: message.volumeName },
				});
			case 'page_added':
				return $t({
					message: 'Adding page {pageNum} of {pageTot} to Volume {volIdx}',
					values: {
						pageNum: message.pageNumber ?? 0,
						pageTot: message.totalPages ?? 0,
						volIdx: message.volumeIndex + 1,
					},
				});
			case 'volume_finished':
				if (message.success) {
					return $t({
						message: 'Finished Volume {volIdx}: {volName}',
						values: { volIdx: message.volumeIndex + 1, volName: message.volumeName },
					});
				} else {
					return $t({
						message: 'Failed Volume {volIdx}: {volName}',
						values: { volIdx: message.volumeIndex + 1, volName: message.volumeName },
					});
				}
			default:
				return '';
		}
	}

	let messageLogElement = $state<HTMLDivElement | undefined>();
	$effect(() => {
		if (messageLogElement && conversionProgress.statusMessages.length > 0) {
			messageLogElement.scrollTop = messageLogElement.scrollHeight;
		}
	});
</script>

{#if !isComplete}
	<div class="h-full w-full p-3">
		<BentoGrid cols={3} density="comfortable" rows="auto 1fr" class="h-full">
			<!-- Summary Cards -->
			<BentoItem glass padding="sm">
				<VStack gap="sm" align="center" class="text-center">
					<div
						class="bg-accent-500/10 border-accent-500/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
					>
						<IconLoader class="text-accent-500 animate-spin" size={32} />
					</div>
					<div class="text-accent-500 text-4xl font-bold">
						{overallProgress.toFixed(1)}%
					</div>
					<span class="text-muted text-sm font-medium">{$t`Overall Progress`}</span>
				</VStack>
			</BentoItem>

			<BentoItem glass padding="sm">
				<VStack gap="sm" align="center" class="text-center">
					<div
						class="bg-success/10 border-success/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
					>
						<IconCircleCheck size={32} class="text-success" />
					</div>
					<div class="text-success text-4xl font-bold">
						{conversionProgress.completed.successful}
					</div>
					<span class="text-muted text-sm font-medium">{$t`Successful`}</span>
				</VStack>
			</BentoItem>

			<BentoItem glass padding="sm">
				<VStack gap="sm" align="center" class="text-center">
					<div
						class="bg-error/10 border-error/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
					>
						<IconCircleX size={32} class="text-error" />
					</div>
					<div class="text-error text-4xl font-bold">
						{conversionProgress.completed.failed}
					</div>
					<span class="text-muted text-sm font-medium">{$t`Failed`}</span>
				</VStack>
			</BentoItem>

			<!-- Current Volume -->
			{#if conversionProgress.activeVolumes.length > 0}
				{@const volume = conversionProgress.activeVolumes[0]}
				<BentoItem colspan={2} glass>
					<div in:fly={{ y: 10, duration: 300 }}>
						<HStack gap="sm" align="center" class="text-muted mb-3">
							<IconLoader class="animate-spin" size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Current Volume`}</span>
						</HStack>
						<VStack gap="sm" class="bg-surface-2 rounded-lg p-4">
							<HStack align="center" justify="between">
								<span class="text-sm font-semibold">
									{$t({ message: 'Volume {vol}', values: { vol: volume.index + 1 } })}
								</span>
								<Badge variant="primary" class="text-lg font-bold">
									{volume.progress.toFixed(0)}%
								</Badge>
							</HStack>
							<div class="truncate text-base font-medium" title={volume.name}>
								{volume.name}
							</div>
							<div class="bg-surface-0 h-2 w-full overflow-hidden rounded-full">
								<div
									class="bg-accent-500 h-full rounded-full transition-all duration-150"
									style="width: {volume.progress}%"
								></div>
							</div>
							<HStack align="center" justify="between" class="text-sm">
								<span class="text-muted">{$t`Images`}</span>
								<span class="font-medium">
									{volume.currentImage} / {volume.totalImages}
								</span>
							</HStack>
						</VStack>
					</div>
				</BentoItem>
			{:else}
				<BentoItem colspan={2} glass>
					<HStack gap="sm" align="center" class="text-muted mb-3">
						<IconLoader class="animate-spin" size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Current Volume`}</span>
					</HStack>
					<div class="bg-surface-2 flex items-center justify-center rounded-lg p-8">
						<span class="text-muted text-sm">{$t`Preparing conversion...`}</span>
					</div>
				</BentoItem>
			{/if}

			<!-- Live Status Messages -->
			{#if conversionProgress.statusMessages.length > 0}
				<BentoItem glass>
					<HStack gap="sm" align="center" class="text-muted mb-3">
						<IconBolt size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Live Log`}</span>
					</HStack>
					<div
						bind:this={messageLogElement}
						class="bg-surface-2 max-h-48 overflow-y-auto rounded-lg p-3 font-mono text-[10px]"
					>
						{#each conversionProgress.statusMessages as message}
							<div class="border-waku-border border-b py-1 opacity-80 last:border-b-0">
								{formatStatusMessage(message)}
							</div>
						{/each}
					</div>
				</BentoItem>
			{/if}

			<!-- All Volumes List -->
			{#if conversionProgress.volumes.length > 0}
				<BentoItem colspan={3} glass>
					<HStack gap="sm" align="center" class="text-muted mb-3">
						<IconChartPie size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`All Volumes`}</span>
						<Badge variant="neutral" class="ml-auto">{conversionProgress.volumes.length}</Badge>
					</HStack>
					<div class="bg-surface-2 max-h-64 space-y-1 overflow-y-auto rounded-lg p-2">
						{#each conversionProgress.volumes as volume}
							<div
								class="hover:bg-surface-0 flex items-center justify-between rounded-md px-3 py-2 text-sm transition-colors"
							>
								<HStack align="center" gap="sm" class="min-w-0 flex-1">
									{#if volume.status === 'completed'}
										<IconCircleCheck size={16} class={getStatusClass(volume.status)} />
									{:else if volume.status === 'failed'}
										<IconCircleX size={16} class={getStatusClass(volume.status)} />
									{:else if volume.status === 'processing'}
										<div class="relative h-4 w-4">
											<IconLoader
												size={16}
												class="{getStatusClass(volume.status)} absolute animate-spin"
											/>
										</div>
									{:else}
										<IconCircle size={16} class="opacity-20" />
									{/if}
									<span
										class="truncate {volume.status === 'pending' ? 'opacity-50' : ''}"
										title={volume.name}
									>
										{volume.name || `${$t`Volume`} ${volume.index + 1}`}
									</span>
								</HStack>
								{#if volume.status !== 'pending'}
									<HStack align="center" gap="sm">
										{#if volume.status === 'processing'}
											<span class="text-muted text-xs">{volume.progress.toFixed(0)}%</span>
										{/if}
										{#if volume.errorMessage}
											<span title={volume.errorMessage}>
												<IconAlertCircle size={16} class="text-error" />
											</span>
										{/if}
									</HStack>
								{/if}
							</div>
						{/each}
					</div>
				</BentoItem>
			{/if}
		</BentoGrid>
	</div>
{:else}
	{#if showConfetti}
		<Confetti count={300} autoStart duration={null} />
	{/if}

	<div class="flex h-full w-full flex-col items-center justify-center p-3">
		<BentoGrid cols={1} density="comfortable" class="w-full max-w-2xl">
			<!-- Header -->
			<BentoItem glass class="text-center">
				<div class="mb-2 inline-flex items-center justify-center">
					{#if conversionProgress.completed.failed === 0}
						<div
							class="bg-success/20 border-success/30 flex h-16 w-16 items-center justify-center rounded-full border-2"
						>
							<IconCircleCheck size={32} class="text-success" />
						</div>
					{:else}
						<div
							class="border-accent-500/30 bg-accent-500/20 flex h-16 w-16 items-center justify-center rounded-full border-2"
						>
							<IconCheck size={32} class="text-accent-500" />
						</div>
					{/if}
				</div>
				<h3 class="mb-2 text-xl font-bold">{$t`Conversion Complete!`}</h3>
				<p class="text-muted">
					{#if conversionProgress.completed.failed === 0}
						{$t`All volumes converted successfully!`}
					{:else if conversionProgress.completed.successful === 0}
						{$t`Conversion completed with errors`}
					{:else}
						{$t`Conversion completed with some errors`}
					{/if}
				</p>
			</BentoItem>

			<!-- Statistics -->
			<BentoItem glass>
				<HStack align="center" gap="sm" class="text-muted mb-4">
					<IconChartPie size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Results`}</span>
				</HStack>

				<div class="grid grid-cols-2 gap-4">
					<div
						class="bg-success/10 border-success/20 flex flex-col items-center rounded-lg border p-3"
					>
						<HStack align="center" gap="sm" class="mb-2">
							<IconCircleCheck size={24} class="text-success" />
							<span class="text-success text-3xl font-bold">
								{conversionProgress.completed.successful}
							</span>
						</HStack>
						<span class="text-muted text-xs">{$t`Successful`}</span>
					</div>

					<div class="bg-error/10 border-error/20 flex flex-col items-center rounded-lg border p-3">
						<HStack align="center" gap="sm" class="mb-2">
							<IconCircleX size={24} class="text-error" />
							<span class="text-error text-3xl font-bold">
								{conversionProgress.completed.failed}
							</span>
						</HStack>
						<span class="text-muted text-xs">{$t`Failed`}</span>
					</div>
				</div>
			</BentoItem>

			<!-- Duration -->
			{#if conversionProgress.durationSeconds !== null}
				<BentoItem glass class="text-center">
					<HStack align="center" justify="center" gap="sm" class="mb-3">
						<IconClock size={24} class="text-accent-500" />
						<span class="text-lg font-medium">{$t`Processing Time`}</span>
					</HStack>
					<div class="text-accent-500 text-4xl font-bold">
						{formatTime(conversionProgress.durationSeconds)}
					</div>
				</BentoItem>
			{/if}

			<!-- Errors Summary -->
			{#if conversionProgress.errors.length > 0}
				<BentoItem glass>
					<h4 class="text-error mb-3 flex items-center gap-2 font-semibold">
						<IconAlertCircle size={18} />
						{$t`Failed Volumes`}
					</h4>
					<div class="bg-error/10 border-error/20 max-h-40 overflow-y-auto rounded-lg border p-3">
						<VStack gap="sm" class="text-sm">
							{#each conversionProgress.errors as error}
								<div class="border-error/20 border-b pb-2 last:border-b-0">
									<div class="font-medium">{error.volumeName}</div>
									<div class="text-muted mt-1 text-xs">{error.error}</div>
								</div>
							{/each}
						</VStack>
					</div>
				</BentoItem>
			{/if}

			<!-- Action Button -->
			<BentoItem glass class="text-center">
				<Button
					variant="primary"
					size="lg"
					onclick={resetConversion}
					style="seamless"
					class="w-full"
				>
					<HStack align="center" gap="sm">
						<IconBolt size={20} />
						<span>{$t`Convert another manga`}</span>
					</HStack>
				</Button>
			</BentoItem>
		</BentoGrid>
	</div>
{/if}

<style>
	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	:global(.animate-spin) {
		animation: spin 1s linear infinite;
	}
</style>
