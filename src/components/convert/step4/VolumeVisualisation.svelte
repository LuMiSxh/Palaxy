<script lang="ts">
	import { IconFileZip } from '@tabler/icons-svelte';
	import convState from '$states/converter.svelte';
	import { t, plural } from 'svelte-i18n-lingui';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import { VStack, HStack } from 'waku/layout';
	import { Badge } from 'waku/components';

	interface Props {
		scrollContainer?: HTMLDivElement | null;
	}

	let { scrollContainer = $bindable(null) }: Props = $props();
</script>

<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
	<IconFileZip size={18} />
	<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Distribution`}</span>
</HStack>

{#if convState.chapterSizes.length > 0}
	<div bind:this={scrollContainer} class="custom-scrollbar flex-1 space-y-3 overflow-y-auto pr-2">
		{#each convState.chapterSizes as chapters, i}
			{@const totalChapters = step4State.result?.total_chapters || 1}
			{@const previousChapters = convState.chapterSizes.slice(0, i).reduce((sum, c) => sum + c, 0)}
			{@const currentPercentage = (chapters / totalChapters) * 100}

			<div class="bg-surface-2 group hover:bg-surface-1 rounded-lg p-4 transition-all">
				<HStack justify="between" align="center" class="mb-3">
					<HStack gap="sm" align="center">
						<div
							class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {i % 2 === 0
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
		class="border-waku-border/50 flex flex-1 items-center justify-center rounded-lg border border-dashed p-8"
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

<style>
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
		transition: background-color 0.2s;
	}

	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background-color: var(--waku-surface-1);
	}
</style>
