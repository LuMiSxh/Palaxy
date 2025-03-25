<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { IconBookmark, IconFileZip } from '@tabler/icons-svelte';
	import convState, { stepState } from '$states/converter.svelte';
	import { commands } from '$types';
	import { addToast } from '$states/toast.svelte';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import LoadingSpinner from '$components/LoadingSpinner.svelte';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import VolumeVisualisation from '$components/convert/step4/VolumeVisualisation.svelte';
	import ManualBundling from '$components/convert/step4/ManualBundling.svelte';

	// FIXME Rework manual and image bundling
	let unregisterKeyHint: () => void;

	async function runBundler() {
		step4State.loading = true;
		stepState.disablePrev = true;
		stepState.disableNext = true;

		// But since we are too fast, we need to wait for the backend to update its state first.
		await tick();
		const result = await wrapper(commands.convBundle(step4State.sensibility));

		step4State.loading = false;

		if (result !== null && result.payload !== null) {
			step4State.result = result.payload;
			convState.chapterSizes = result.payload.chapter_sizes ?? [];

			// Only update the UI if we have valid data
			stepState.disableNext = convState.chapterSizes.length <= 0;
		} else {
			// Handle the error case
			addToast($t`Failed to run bundler. Please try again.`, 'error');
			stepState.disableNext = true;
		}

		stepState.disablePrev = false;
	}

	onMount(async () => {
		unregisterKeyHint = keyHint.smartAdd([['tab', $t`Navigate between fields`]]);

		// Run bundler on mount
		await runBundler();
	});

	onDestroy(async () => {
		if (unregisterKeyHint) unregisterKeyHint();

		// Save chapter sizes
		await wrapper(commands.convStateSet({ VolumeSizes: convState.chapterSizes }));
	});
</script>

<div class="flex h-full w-full flex-col" style="height: calc(100vh - 8rem)">
	{#if step4State.loading}
		<div class="flex h-full items-center justify-center">
			<LoadingSpinner text={$t`Running Bundler...`} />
		</div>
	{:else if convState.bundle === "MANUAL"}
		<ManualBundling />
	{:else}
		<div class="grid h-full grid-cols-[1fr_2fr] gap-6 p-4">
			<!-- Left side: Info panel -->
			<div class="flex flex-col gap-4">
				<div class="card">
					<div class="card-body">
						<h3 class="mb-4 text-xl font-semibold">{$t`Detection Result`}</h3>

						<div class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="flex items-center">
									<IconBookmark class="mr-2 text-primary" size={20} />
									{$t`Detected Chapters`}
								</span>
								<span class="badge badge-primary">{step4State.result?.total_chapters ?? 0}</span>
							</div>

							<div class="flex items-center justify-between">
								<span class="flex items-center">
									<IconFileZip class="mr-2 text-primary" size={20} />
									{$t`Detected Volumes`}
								</span>
								<span class="badge badge-primary">{step4State.result?.total_volumes ?? 0}</span>
							</div>
						</div>
					</div>
				</div>

				<div class="card">
					<div class="card-body">
						<h3 class="mb-4 text-xl font-semibold">{$t`Bundle Options`}</h3>

						<button
							class="btn btn-outline w-full mb-2"
							use:handleKeyHint={{keys: [['enter', $t`Invoke`]]}}
							onclick={() => {
								convState.bundle = "MANUAL"
							}}
						>
							{$t`Switch to Manual Bundling`}
						</button>

						<button
							class="btn btn-primary w-full"
							use:handleKeyHint={{keys: [['enter', $t`Invoke`]]}}
							onclick={runBundler}
						>
							{$t`Rerun Bundler`}
						</button>
					</div>
				</div>
			</div>

			<!-- Right side: Volumes visualization -->
			<VolumeVisualisation />
			<!--{#if convState.bundle === "IMAGE"}-->
			<!--	<div class="h-full w-full p-2">-->
			<!--		<table class="table">-->
			<!--			<thead>-->
			<!--			<tr>-->
			<!--				<th>{$t`Image Detection`}</th>-->
			<!--			</tr>-->
			<!--			</thead>-->
			<!--			<tbody>-->
			<!--			<tr>-->
			<!--				<td>-->
			<!--					<div class="w-full">-->
			<!--						<div class="flex items-center justify-between mb-2">-->
			<!--							<label for="range-slider" class="text-sm">-->
			<!--								{$t`Grayscale Sensibility`}:-->
			<!--								<code class="text-primary">-->
			<!--									{sensibility}%-->
			<!--								</code>-->
			<!--							</label>-->
			<!--						</div>-->
			<!--						<input-->
			<!--							id="range-slider"-->
			<!--							type="range"-->
			<!--							min="0"-->
			<!--							max="100"-->
			<!--							step="5"-->
			<!--							class="w-full h-2 bg-background-tertiary dark:bg-background-dark-tertiary rounded-lg appearance-none cursor-pointer"-->
			<!--							bind:value={sensibility}-->
			<!--							use:handleKeyHint={{-->
			<!--									keys: [-->
			<!--										['arrowleft', $t`Decrease`],-->
			<!--										['arrowright', $t`Increase`]-->
			<!--									],-->
			<!--									reset: true-->
			<!--								}}-->
			<!--						/>-->
			<!--						<div class="flex w-full justify-between text-xs mt-1">-->
			<!--							<span>0%</span>-->
			<!--							<span>25%</span>-->
			<!--							<span>50%</span>-->
			<!--							<span>75%</span>-->
			<!--							<span>100%</span>-->
			<!--						</div>-->
			<!--					</div>-->
			<!--				</td>-->
			<!--			</tr>-->
			<!--			<tr>-->
			<!--				<td class="flex h-full w-full items-center justify-around pt-4">-->
			<!--					<button-->
			<!--						class="btn bg-primary hover:bg-primary-hover active:bg-primary-active"-->
			<!--						onclick={runBundler}-->
			<!--					>-->
			<!--						{$t`Rerun Bundler`}-->
			<!--					</button>-->
			<!--					<button-->
			<!--						class="btn bg-secondary hover:bg-secondary-hover active:bg-secondary-active"-->
			<!--						onclick={() => {-->
			<!--								convState.bundle = "MANUAL"-->
			<!--							}}-->
			<!--					>-->
			<!--						{$t`Manual Bundling`}-->
			<!--					</button>-->
			<!--				</td>-->
			<!--			</tr>-->
			<!--			</tbody>-->
			<!--		</table>-->
			<!--	</div>-->
			<!--{/if}-->
		</div>
	{/if}
</div>
