<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState from '$states/converter.svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { IconFolder, IconFolderOpen } from '@tabler/icons-svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Button, Badge } from 'waku/components';

	let btn: HTMLButtonElement | null = $state(null);
	let unregisterKeyboard: () => void;

	onMount(async () => {
		// Reset
		await wrapper(commands.convStateReset());
		converter.reset();

		// Set Keyboard etc.
		btn?.focus();
		unregisterKeyboard = keyboard.smartRegister([['enter', select]]);
	});

	onDestroy(() => {
		if (unregisterKeyboard) unregisterKeyboard();
	});

	async function select(evt: KeyboardEvent | MouseEvent | undefined = undefined) {
		evt?.stopPropagation();
		evt?.preventDefault();

		convState.source =
			(await open({
				directory: true,
				multiple: false,
			})) ?? '';

		if (converter.source !== null) {
			await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
		}
	}
</script>

<div class="w-full p-3 pb-3">
	<VStack gap="md" class="mx-auto max-w-4xl">
		<BentoGrid cols={1} density="compact">
			<BentoItem>
				<VStack gap="sm" class="w-full">
					<HStack gap="sm" align="center" class="text-muted mb-2">
						{#if !convState.source}
							<IconFolder size={18} />
						{:else}
							<IconFolderOpen size={18} />
						{/if}
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Source Selection`}</span>
					</HStack>

					<!-- Selection Button -->
					<button
						bind:this={btn}
						class="bg-surface-2 hover:bg-surface-1 hover:border-accent-500/50 group focus:border-accent-500 focus:ring-accent-500/20 relative w-full overflow-hidden rounded-lg border border-transparent p-4 text-left transition-all duration-200 focus:ring-2 focus:outline-none"
						use:handleKeyHint={{ keys: [['enter', $t`Select location`]] }}
						onclick={select}
					>
						{#if !convState.source}
							<HStack gap="md" align="center">
								<div
									class="bg-surface-0 group-hover:bg-accent-500/10 flex h-12 w-12 shrink-0 items-center justify-center rounded-lg transition-colors duration-200"
								>
									<IconFolder
										size={24}
										class="text-muted group-hover:text-accent-500 transition-colors"
									/>
								</div>
								<VStack gap="xs" class="flex-1">
									<span class="text-base font-medium">{$t`Browse for folder`}</span>
									<span class="text-muted text-sm">
										{$t`Choose the directory containing your manga images`}
									</span>
								</VStack>
								<Badge variant="secondary" class="text-xs">{$t`Enter`}</Badge>
							</HStack>
						{:else}
							<VStack gap="sm" class="w-full">
								<HStack gap="sm" align="center">
									<div
										class="bg-accent-500/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg"
									>
										<IconFolderOpen size={20} class="text-accent-500" />
									</div>
									<VStack gap="xs" class="flex-1 overflow-hidden">
										<span class="text-sm font-semibold">{$t`Selected folder`}</span>
										<div
											class="bg-surface-0 w-full truncate rounded px-3 py-1.5 font-mono text-xs"
											title={convState.source}
										>
											{truncatePath(convState.source, 70)}
										</div>
									</VStack>
								</HStack>
								<span class="text-muted text-xs">{$t`Click to change selection`}</span>
							</VStack>
						{/if}
					</button>

					<!-- Ready indicator -->
					{#if convState.source}
						<div class="bg-success/10 border-success/30 rounded-lg border p-3">
							<HStack gap="sm" align="start">
								<div class="bg-success/20 mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full"></div>
								<VStack gap="xs">
									<span class="text-sm font-medium">{$t`Ready to proceed`}</span>
									<span class="text-muted text-xs">
										{$t`Click Next to analyze the source folder contents`}
									</span>
								</VStack>
							</HStack>
						</div>
					{/if}
				</VStack>
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
