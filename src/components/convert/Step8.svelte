<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import {
		IconCheck,
		IconClock,
		IconAlertCircle,
		IconCircleCheck,
		IconCircleX,
		IconBolt,
		IconBook,
		IconLoader2,
	} from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { commands } from '$types';
	import { keyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import { useConversionProgress } from '$lib/useConversionProgress.svelte';
	import Confetti from '$components/Confetti.svelte';
	import { stepMachine } from '$states/stepMachine.svelte';
	import convState from '$states/converter.svelte';

	// Waku Imports
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Button, Badge } from 'waku/components';

	const conversionProgress = useConversionProgress();

	let unregisterKey: () => void;
	let showConfetti = $state(false);
	let isComplete = $state(false);
	let completionContainer = $state<HTMLDivElement>();

	function formatTime(seconds: number): string {
		if (seconds < 60) {
			return `${seconds.toFixed(1)} ${$t`seconds`}`;
		} else {
			const minutes = Math.floor(seconds / 60);
			const remainingSeconds = seconds % 60;
			return `${minutes} ${minutes === 1 ? $t`minute` : $t`minutes`} ${remainingSeconds.toFixed(0)} ${$t`seconds`}`;
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

			// Focus the action BentoItem after completion
			await tick();
			const actionItem = document.querySelector('[data-id="action-bentoitem"]');
			if (actionItem) {
				(actionItem as HTMLElement).focus();
			}
		}
	}

	function resetConversion() {
		conversionProgress.reset();
		convState.reset();
		stepMachine.reset();
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
</script>

{#if !isComplete}
	<!-- Converting State -->
	<div class="h-full w-full p-3">
		<div class="flex h-full flex-col items-center justify-center">
			<VStack gap="lg" align="center" class="max-w-md text-center">
				<!-- Spinner -->
				<div
					class="bg-accent-500/10 border-accent-500/20 flex h-32 w-32 items-center justify-center rounded-full border-2"
				>
					<IconLoader2 class="text-accent-500 animate-spin" size={64} />
				</div>

				<!-- Progress -->
				<VStack gap="xs" align="center">
					<div class="text-6xl font-bold">
						{overallProgress.toFixed(0)}<span class="text-muted text-4xl">%</span>
					</div>
					<span class="text-muted text-sm font-medium">{$t`Converting your manga...`}</span>
				</VStack>

				<!-- Progress Bar -->
				<div class="bg-surface-2 h-3 w-full overflow-hidden rounded-full">
					<div
						class="from-accent-500 to-accent-400 h-full bg-linear-to-r transition-all duration-300"
						style="width: {overallProgress}%"
					></div>
				</div>

				<!-- Stats -->
				<HStack gap="lg" justify="center" class="w-full pt-4">
					<VStack gap="xs" align="center">
						<div class="text-success text-2xl font-bold">
							{conversionProgress.completed.successful}
						</div>
						<div class="text-muted text-xs">{$t`Completed`}</div>
					</VStack>

					<div class="bg-surface-2 h-12 w-px"></div>

					<VStack gap="xs" align="center">
						<div class="text-accent-500 text-2xl font-bold">
							{conversionProgress.activeVolumes.length}
						</div>
						<div class="text-muted text-xs">{$t`Processing`}</div>
					</VStack>

					<div class="bg-surface-2 h-12 w-px"></div>

					<VStack gap="xs" align="center">
						<div class="text-muted text-2xl font-bold">
							{conversionProgress.volumes.length -
								conversionProgress.completed.successful -
								conversionProgress.completed.failed -
								conversionProgress.activeVolumes.length}
						</div>
						<div class="text-muted text-xs">{$t`Remaining`}</div>
					</VStack>
				</HStack>
			</VStack>
		</div>
	</div>
{:else}
	<!-- Conversion Complete -->
	{#if showConfetti}
		<Confetti count={300} autoStart duration={null} />
	{/if}

	<div class="h-full w-full overflow-y-auto p-3" bind:this={completionContainer}>
		<VStack gap="md" class="mx-auto max-w-5xl pb-4">
			<BentoGrid cols={3} density="compact">
				<!-- Success Header -->
				<BentoItem colspan={3} glass>
					<div class="text-center">
						<div class="mb-4 inline-flex items-center justify-center">
							{#if conversionProgress.completed.failed === 0}
								<div
									class="bg-success/20 border-success/30 flex h-20 w-20 items-center justify-center rounded-full border-2"
								>
									<IconCircleCheck size={40} class="text-success" />
								</div>
							{:else}
								<div
									class="border-accent-500/30 bg-accent-500/20 flex h-20 w-20 items-center justify-center rounded-full border-2"
								>
									<IconCheck size={40} class="text-accent-500" />
								</div>
							{/if}
						</div>

						<h2 class="mb-2 text-2xl font-bold">{$t`Conversion Complete!`}</h2>
						<p class="text-muted">
							{#if conversionProgress.completed.failed === 0}
								{$t`All volumes have been converted successfully!`}
							{:else if conversionProgress.completed.successful === 0}
								{$t`Conversion completed with errors. Please check the failed volumes below.`}
							{:else}
								{$t`Conversion completed with some errors. Check the details below.`}
							{/if}
						</p>
					</div>
				</BentoItem>

				<!-- Summary Cards -->
				<BentoItem glass>
					<div class="flex flex-col items-center justify-center p-2 text-center">
						<IconBook class="text-accent-500 mb-3" size={32} />
						<div class="text-3xl font-bold">{conversionProgress.volumes.length}</div>
						<div class="text-muted mt-1 text-sm">{$t`Total Volumes`}</div>
					</div>
				</BentoItem>

				<BentoItem glass>
					<div
						class="bg-success/5 flex flex-col items-center justify-center rounded-lg p-2 text-center"
					>
						<IconCircleCheck class="text-success mb-3" size={32} />
						<div class="text-success text-3xl font-bold">
							{conversionProgress.completed.successful}
						</div>
						<div class="text-muted mt-1 text-sm">{$t`Successful`}</div>
					</div>
				</BentoItem>

				<BentoItem glass>
					<div
						class="bg-error/5 flex flex-col items-center justify-center rounded-lg p-2 text-center"
					>
						<IconCircleX class="text-error mb-3" size={32} />
						<div class="text-error text-3xl font-bold">
							{conversionProgress.completed.failed}
						</div>
						<div class="text-muted mt-1 text-sm">{$t`Failed`}</div>
					</div>
				</BentoItem>

				<!-- Processing Time -->
				{#if conversionProgress.durationSeconds !== null}
					<BentoItem colspan={3} glass>
						<HStack align="center" justify="center" gap="sm" class="p-2">
							<IconClock size={24} class="text-accent-500" />
							<span class="text-muted text-sm">{$t`Completed in`}</span>
							<span class="text-accent-500 text-xl font-bold">
								{formatTime(conversionProgress.durationSeconds)}
							</span>
						</HStack>
					</BentoItem>
				{/if}

				<!-- Failed Volumes (if any) -->
				{#if conversionProgress.errors.length > 0}
					<BentoItem colspan={3} glass>
						<HStack align="center" gap="sm" class="text-error mb-3">
							<IconAlertCircle size={18} />
							<span class="text-xs font-bold tracking-wider uppercase">{$t`Failed Volumes`}</span>
							<Badge variant="danger" class="ml-auto">{conversionProgress.errors.length}</Badge>
						</HStack>

						<VStack gap="sm">
							{#each conversionProgress.errors as err (err)}
								<div class="bg-error/5 border-error/20 rounded-lg border p-3">
									<div class="mb-1 font-medium">{err.volumeName}</div>
									<div class="text-muted text-sm">{err.error}</div>
								</div>
							{/each}
						</VStack>
					</BentoItem>
				{/if}

				<!-- Action Button -->
				<BentoItem data-id="action-bentoitem" colspan={3} glass tabindex={0}>
					<Button
						variant="primary"
						size="lg"
						onclick={resetConversion}
						style="seamless"
						class="w-full"
					>
						<HStack align="center" justify="center" gap="sm">
							<IconBolt size={20} />
							<span>{$t`Convert Another Manga`}</span>
						</HStack>
					</Button>
				</BentoItem>
			</BentoGrid>
		</VStack>
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
