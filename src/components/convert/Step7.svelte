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
	} from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { commands } from '$types';
	import { stepState } from '$states/converter.svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import { useConversionProgress } from '$lib/useConversionProgress.svelte';
	import type { StatusMessage } from '$states/conversion.svelte';
	import Confetti from '$components/Confetti.svelte';
	import { fly } from 'svelte/transition';

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
				return 'text-primary';
			default:
				return 'text-base-content/30';
		}
	}

	async function startConversion() {
		stepState.disablePrev = true;
		stepState.disableNext = true;

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
		stepState.reset();
		conversionProgress.reset();
		isComplete = false;
		showConfetti = false;
	}

	onMount(async () => {
		stepState.disablePrev = true;
		stepState.disableNext = true;

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

	// Smoother Global Progress based on image completion across all volumes
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
	<div
		class="flex h-full w-full flex-col overflow-y-auto p-4"
		style="max-height: calc(100vh - 8rem)"
	>
		<div class="card">
			<div class="card-header">
				<div class="flex items-center gap-2">
					<IconLoader class="text-primary animate-spin" size={20} />
					<h3 class="text-lg font-semibold">{$t`Converting Volumes...`}</h3>
				</div>
			</div>

			<div class="card-body space-y-6">
				<!-- Overall Progress -->
				<div>
					<div class="mb-2 flex items-center justify-between">
						<span class="text-sm font-medium">{$t`Overall Progress`}</span>
						<span class="text-primary text-sm font-medium">
							{overallProgress.toFixed(1)}%
						</span>
					</div>
					<progress class="progress progress-primary w-full" max="100" value={overallProgress}
					></progress>
					<div class="mt-2 flex items-center justify-between text-xs">
						<span class="flex items-center gap-1 opacity-70">
							<IconBolt size={14} />
							{conversionProgress.activeVolumes.length}
							{$t`active threads`}
						</span>
						<div class="flex gap-3">
							<span class="text-success flex items-center gap-1">
								<IconCircleCheck size={14} />
								{$t({
									message: '{tot} successfull',
									values: { tot: conversionProgress.completed.successful },
								})}
							</span>
							<span class="text-error flex items-center gap-1">
								<IconCircleX size={14} />
								{$t({
									message: '{tot} failed',
									values: { tot: conversionProgress.completed.failed },
								})}
							</span>
						</div>
					</div>
				</div>

				<!-- Active Conversions Grid -->
				{#if conversionProgress.activeVolumes.length > 0}
					<div>
						<h4 class="mb-2 text-sm font-semibold">{$t`Active Conversions`}</h4>
						<div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
							{#each conversionProgress.activeVolumes as volume (volume.index)}
								<div
									class="bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg p-3 shadow-sm"
									in:fly={{ y: 10, duration: 300 }}
								>
									<div class="mb-2 flex items-center justify-between">
										<span class="text-xs font-semibold opacity-70">
											{$t({ message: 'Volume {vol}', values: { vol: volume.index + 1 } })}
										</span>
										<span class="text-primary text-xs font-bold">{volume.progress.toFixed(0)}%</span
										>
									</div>
									<div class="mb-2 truncate text-sm font-medium" title={volume.name}>
										{volume.name}
									</div>
									<progress
										class="progress progress-secondary w-full"
										max="100"
										value={volume.progress}
									></progress>
									<div class="mt-1 flex justify-end">
										<span class="text-[10px] opacity-60">
											{volume.currentImage} / {volume.totalImages}
										</span>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Volume List (Compact) -->
				{#if conversionProgress.volumes.length > 0}
					<div>
						<h4 class="mb-2 text-sm font-semibold">{$t`Volume Status`}</h4>
						<div
							class="border-background-tertiary dark:border-background-dark-tertiary max-h-48 space-y-1 overflow-y-auto rounded-lg border p-2"
						>
							{#each conversionProgress.volumes as volume}
								<div
									class="hover:bg-background-tertiary dark:hover:bg-background-dark-tertiary flex items-center justify-between rounded px-2 py-1.5 text-sm transition-colors"
								>
									<div class="flex min-w-0 flex-1 items-center gap-2">
										{#if volume.status === 'completed'}
											<IconCircleCheck size={14} class={getStatusClass(volume.status)} />
										{:else if volume.status === 'failed'}
											<IconCircleX size={14} class={getStatusClass(volume.status)} />
										{:else if volume.status === 'processing'}
											<div class="relative h-3.5 w-3.5">
												<IconLoader
													size={14}
													class="{getStatusClass(volume.status)} absolute animate-spin"
												/>
											</div>
										{:else}
											<IconCircle size={14} class="opacity-20" />
										{/if}
										<span
											class="truncate {volume.status === 'pending' ? 'opacity-50' : ''}"
											title={volume.name}
										>
											{volume.name || `${$t`Volume`} ${volume.index + 1}`}
										</span>
									</div>
									{#if volume.status !== 'pending'}
										<div class="flex items-center gap-2">
											{#if volume.status === 'processing'}
												<span class="text-xs opacity-70">{volume.progress.toFixed(0)}%</span>
											{/if}
											{#if volume.errorMessage}
												<span title={volume.errorMessage}>
													<IconAlertCircle size={14} class="text-error" />
												</span>
											{/if}
										</div>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Live Status Messages -->
				{#if conversionProgress.statusMessages.length > 0}
					<div>
						<h4 class="mb-2 text-sm font-semibold">{$t`Live Log`}</h4>
						<div
							bind:this={messageLogElement}
							class="bg-background-tertiary dark:bg-background-dark-tertiary max-h-32 overflow-y-auto rounded p-2 font-mono text-[10px] opacity-80"
						>
							{#each conversionProgress.statusMessages as message}
								<div class="border-background-secondary border-b py-0.5 last:border-b-0">
									{formatStatusMessage(message)}
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{:else}
	{#if showConfetti}
		<Confetti count={300} autoStart duration={null} />
	{/if}

	<div
		class="flex h-full w-full flex-col items-center justify-center overflow-y-auto p-4"
		style="max-height: calc(100vh - 8rem)"
	>
		<div class="card w-full max-w-lg">
			<div class="card-header">
				<h3 class="text-lg font-semibold">{$t`Conversion Complete!`}</h3>
			</div>

			<div class="card-body">
				<p class="mb-4 text-center text-lg">
					{#if conversionProgress.completed.failed === 0}
						{$t`All volumes converted successfully!`}
					{:else if conversionProgress.completed.successful === 0}
						{$t`Conversion completed with errors`}
					{:else}
						{$t`Conversion completed with some errors`}
					{/if}
				</p>

				<!-- Statistics -->
				<div class="mb-6 grid grid-cols-2 gap-4">
					<div class="bg-success/10 flex flex-col items-center rounded-lg p-3">
						<div class="flex items-center gap-2">
							<IconCheck size={20} class="text-success" />
							<span class="text-success text-2xl font-bold"
								>{conversionProgress.completed.successful}</span
							>
						</div>
						<span class="text-xs opacity-70">{$t`Successful`}</span>
					</div>

					<div class="bg-error/10 flex flex-col items-center rounded-lg p-3">
						<div class="flex items-center gap-2">
							<IconAlertCircle size={20} class="text-error" />
							<span class="text-error text-2xl font-bold"
								>{conversionProgress.completed.failed}</span
							>
						</div>
						<span class="text-xs opacity-70">{$t`Failed`}</span>
					</div>
				</div>

				<!-- Duration -->
				{#if conversionProgress.durationSeconds !== null}
					<div class="my-6 flex w-full flex-col items-center gap-2">
						<div class="flex items-center gap-3">
							<IconClock size={24} class="text-primary" />
							<span class="text-lg font-medium">{$t`Processing Time`}</span>
						</div>
						<span class="text-primary text-4xl font-bold">
							{formatTime(conversionProgress.durationSeconds)}
						</span>
					</div>
				{/if}

				<!-- Errors Summary -->
				{#if conversionProgress.errors.length > 0}
					<div
						class="bg-error/10 border-error/30 mb-4 max-h-40 overflow-y-auto rounded-lg border p-3"
					>
						<h4 class="text-error mb-2 flex items-center gap-2 text-sm font-semibold">
							<IconAlertCircle size={18} />
							{$t`Failed Volumes`}
						</h4>
						<ul class="space-y-1 text-xs">
							{#each conversionProgress.errors as error}
								<li class="flex flex-col gap-1">
									<span class="font-medium">{error.volumeName}</span>
									<span class="pl-2 opacity-80">{error.error}</span>
								</li>
							{/each}
						</ul>
					</div>
				{/if}

				<div class="flex justify-center">
					<button class="btn btn-lg btn-success gap-2" onclick={resetConversion}>
						<IconCheck size={20} />
						<span>{$t`Convert another manga`}</span>
					</button>
				</div>
			</div>
		</div>
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
