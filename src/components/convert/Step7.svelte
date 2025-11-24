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

	let overallProgress = $derived.by(() => {
		if (conversionProgress.totalVolumes === 0) return 0;
		return (
			((conversionProgress.completed.successful + conversionProgress.completed.failed) /
				conversionProgress.totalVolumes) *
			100
		);
	});

	let currentVolumeProgress = $derived(conversionProgress.currentVolume?.progress || 0);

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

			<div class="card-body space-y-4">
				<!-- Overall Progress -->
				<div>
					<div class="mb-2 flex items-center justify-between">
						<span class="text-sm font-medium">{$t`Overall Progress`}</span>
						<span class="text-primary text-sm font-medium">
							{conversionProgress.completed.successful + conversionProgress.completed.failed} / {conversionProgress.totalVolumes}
						</span>
					</div>
					<progress class="progress progress-primary w-full" max="100" value={overallProgress}
					></progress>
					<div class="mt-2 flex items-center justify-between text-xs">
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

				<!-- Current Volume Progress -->
				{#if conversionProgress.currentVolume}
					<div class="bg-base-200 rounded-lg p-4">
						<div class="mb-2 flex items-center justify-between">
							<span class="text-sm font-semibold">{$t`Current Volume`}</span>
							<span class="badge badge-primary badge-sm">
								{$t({
									message: 'Volume {vol}',
									values: { vol: conversionProgress.currentVolume.index + 1 },
								})}
							</span>
						</div>
						<h4
							class="text-primary mb-3 truncate font-medium"
							title={conversionProgress.currentVolume.name}
						>
							{conversionProgress.currentVolume.name}
						</h4>
						<progress
							class="progress progress-secondary w-full"
							max="100"
							value={currentVolumeProgress}
						></progress>
						<div class="mt-2 flex justify-between text-xs">
							<span>
								{$t({
									message: '{cur}/{tot} images',
									values: {
										cur: conversionProgress.currentVolume.currentImage,
										tot: conversionProgress.currentVolume.totalImages,
									},
								})}
							</span>
							<span>{currentVolumeProgress.toFixed(1)}%</span>
						</div>
					</div>
				{/if}

				<!-- Volume List -->
				{#if conversionProgress.volumes.length > 0}
					<div>
						<h4 class="mb-2 text-sm font-semibold">{$t`All Volumes`}</h4>
						<div class="border-base-300 max-h-60 space-y-1 overflow-y-auto rounded-lg border p-2">
							{#each conversionProgress.volumes as volume}
								<div
									class="flex items-center justify-between rounded px-3 py-2 text-sm"
									class:bg-base-200={volume.status === 'processing'}
								>
									<div class="flex min-w-0 flex-1 items-center gap-2">
										{#if volume.status === 'completed'}
											<IconCircleCheck size={16} class={getStatusClass(volume.status)} />
										{:else if volume.status === 'failed'}
											<IconCircleX size={16} class={getStatusClass(volume.status)} />
										{:else if volume.status === 'processing'}
											<IconLoader size={16} class="{getStatusClass(volume.status)} animate-spin" />
										{:else}
											<IconCircle size={16} class={getStatusClass(volume.status)} />
										{/if}
										<span class="truncate" title={volume.name}>
											{volume.name || `${$t`Volume`} ${volume.index + 1}`}
										</span>
									</div>
									<div class="flex items-center gap-2">
										{#if volume.status === 'processing' || volume.status === 'completed'}
											<span class="text-xs opacity-70">{volume.progress.toFixed(0)}%</span>
										{/if}
										{#if volume.errorMessage}
											<span title={volume.errorMessage}>
												<IconAlertCircle size={16} class="text-error" />
											</span>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Live Status Messages -->
				{#if conversionProgress.statusMessages.length > 0}
					<div>
						<h4 class="mb-2 text-sm font-semibold">{$t`Live Status`}</h4>
						<div
							bind:this={messageLogElement}
							class="bg-base-200 max-h-40 overflow-y-auto rounded p-2 font-mono text-xs"
						>
							{#each conversionProgress.statusMessages as message}
								<div class="border-base-300 border-b py-1 last:border-b-0">
									{formatStatusMessage(message)}
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Errors Section -->
				{#if conversionProgress.errors.length > 0}
					<div class="alert alert-error">
						<div class="w-full">
							<h4 class="mb-2 flex items-center gap-2 font-semibold">
								<IconAlertCircle size={18} />
								{$t`Errors`}
							</h4>
							<ul class="space-y-1 text-xs">
								{#each conversionProgress.errors as error}
									<li class="flex gap-2">
										<span class="font-medium">{error.volumeName}:</span>
										<span class="opacity-80">{error.error}</span>
									</li>
								{/each}
							</ul>
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
