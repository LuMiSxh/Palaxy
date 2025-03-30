<script lang="ts">
	import { onDestroy, onMount, tick, untrack } from 'svelte';
	import { IconCheck, IconClock } from '@tabler/icons-svelte';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { commands } from '$types';
	import { stepState } from '$states/converter.svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import Confetti from '$components/Confetti.svelte';

	let unregisterKey: () => void;
	let loading = $state(true);
	let duration = $state(0);
	let showConfetti = $state(false);

	function formatTime(seconds: number): string {
		if (seconds < 60) {
			return `${seconds.toFixed(1)} ${$t`seconds`}`;
		} else {
			const minutes = Math.floor(seconds / 60);
			const remainingSeconds = seconds % 60;
			return `${minutes} ${minutes === 1 ? $t`minute` : $t`minutes`} ${remainingSeconds.toFixed(0)} ${$t`seconds`}`;
		}
	}

	function keyHinter(add = true) {
		if (add) {
			keyHint.addKey('enter', $t`Convert another manga`);
		} else {
			keyHint.removeKey('enter');
		}
	}

	onMount(async () => {
		stepState.disablePrev = true;
		stepState.disableNext = true;
		// Setup keyhint and keyboard
		unregisterKey = keyboard.smartRegister([
			[
				'enter',
				() => {
					if (!loading) stepState.reset();
				}
			]
		]);

		const result = await wrapper(commands.convConvert());
		loading = false;

		if (result != null) {
			duration = result.duration;
			// Trigger confetti animation after loading completes
			await tick();
			showConfetti = true;
		}
	});

	onDestroy(() => {
		// Cleanup keyboard
		if (unregisterKey) unregisterKey();
	});

	$effect(() => {
		const shouldAdd = !loading;
		untrack(() => keyHinter(shouldAdd));
	});
</script>

{#if loading}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<LoadingSpinner text={$t`Converting...`} />
	</div>
{:else}
	{#if showConfetti}
		<!-- eslint-disable @typescript-eslint/no-unused-vars -->
		<Confetti duration={null} />
	{/if}

	<div
		class="flex h-full w-full flex-col items-center justify-center overflow-y-auto p-4"
		style="max-height: calc(100vh - 8rem)"
	>
		<div class="card w-full max-w-lg shadow-lg" style="animation: var(--animate-fade-in)">
			<div class="card-header flex justify-center">
				<h3 class="text-lg font-semibold">
					{$t`Conversion Complete!`}
				</h3>
			</div>

			<div class="card-body">
				<p class="mb-4 text-center text-lg">
					{$t`The conversion process has completed successfully`}
				</p>

				<div class="my-6 flex w-full flex-col items-center gap-2">
					<div class="flex w-full items-center justify-center gap-3">
						<IconClock size={24} class="text-primary" />
						<span class="text-lg font-medium">{$t`Processing Time`}</span>
					</div>
					<div class="flex flex-col items-center justify-center">
						<span
							class="from-primary via-secondary to-primary bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent"
						>
							{formatTime(duration)}
						</span>
					</div>
				</div>

				<div class="mb-4 flex justify-center">
					<button
						class="btn btn-lg btn-success gap-2 transition-all hover:scale-105"
						onclick={() => stepState.reset()}
					>
						<IconCheck size={20} />
						{$t`Convert another manga`}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
