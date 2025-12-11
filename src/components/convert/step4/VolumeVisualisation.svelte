<script lang="ts">
	import convState from '$states/converter.svelte';
	import { t, plural } from 'svelte-i18n-lingui';
	import {
		getChaptersPercentage,
		getTotalChapters,
		step4State,
	} from '$components/convert/step4/utils.svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { VStack, HStack } from 'waku/layout';
	import { Badge } from 'waku/components';

	let volVisFocused = $state(false);
</script>

<div
	class="glass-subtle flex h-full flex-col overflow-hidden rounded-xl"
	class:ring-2={volVisFocused}
	class:ring-accent-500={volVisFocused}
>
	<div
		class="flex flex-col overflow-y-auto p-4"
		tabindex="0"
		role="tabpanel"
		onfocus={() => (volVisFocused = true)}
		onblur={() => (volVisFocused = false)}
		use:handleKeyHint={{
			keys: [
				['arrowup', $t`Scroll up`],
				['arrowdown', $t`Scroll down`],
			],
		}}
	>
		<h3 class="mb-3 text-base font-semibold">{$t`Volume Distribution`}</h3>

		{#if convState.chapterSizes.length > 0}
			<VStack gap="sm" class="flex-1">
				{#each convState.chapterSizes as chapters, i}
					<div class="bg-surface-1 hover:bg-surface-2 rounded-lg p-3 transition-all">
						<HStack justify="between" align="center" class="mb-3">
							<span class="font-medium"
								>{$t({ message: 'Volume {vol}', values: { vol: i + 1 } })}</span
							>
							<Badge variant="secondary">
								{$plural(chapters, {
									one: '# chapter',
									other: '# chapters',
								})}
							</Badge>
						</HStack>

						<div class="flex gap-1">
							{#each Array(chapters) as _, j}
								<div
									class="bg-accent-500 h-4 flex-1 rounded-sm opacity-80 transition-all hover:scale-105 hover:opacity-100"
									style="--index: {j}; animation-delay: calc(var(--index) * 30ms);"
									title={$t({
										message: 'Chapter {chap} of Volume {vol}',
										values: { chap: j + 1, vol: i + 1 },
									})}
								></div>
							{/each}
						</div>
					</div>
				{/each}
			</VStack>
		{:else}
			<div
				class="border-waku-border/50 flex h-24 items-center justify-center rounded-lg border border-dashed p-4"
			>
				<p class="text-muted text-center text-sm">
					{$t`No volumes detected. Try running the bundler again.`}
				</p>
			</div>
		{/if}

		{#if step4State.result && step4State.result.total_chapters > 0}
			<div class="border-waku-border/50 mt-4 border-t pt-3">
				<HStack justify="between" align="center" class="mb-2">
					<span class="text-sm font-medium">{$t`Total Chapter Usage`}</span>
					<span class="text-sm">{getTotalChapters()}/{step4State.result.total_chapters}</span>
				</HStack>
				<div class="bg-surface-2 h-2 w-full overflow-hidden rounded-full">
					<div
						class="h-full rounded-full transition-all duration-300 ease-out"
						class:bg-success={getTotalChapters() === step4State.result.total_chapters}
						class:bg-warning={getTotalChapters() < step4State.result.total_chapters}
						class:bg-danger={getTotalChapters() > step4State.result.total_chapters}
						style="width: {getChaptersPercentage()}%"
					></div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	/* Ensure the scrollable container has smooth scrolling */
	[role='tabpanel'] {
		scroll-behavior: smooth;
		-webkit-overflow-scrolling: touch;
	}

	/* Remove focus outline */
	[role='tabpanel']:focus {
		outline: none;
	}
</style>
