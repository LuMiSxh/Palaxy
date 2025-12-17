<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { BentoGrid, BentoItem, VStack, HStack } from 'waku/layout';
	import { Button, Badge } from 'waku/components';
	import {
		IconTransform,
		IconCommand,
		IconSettings,
		IconBook,
		IconSparkles,
	} from '@tabler/icons-svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { getVersion, getTauriVersion } from '@tauri-apps/api/app';

	let versions = $state({ app: '', tauri: '', loading: true });

	onMount(() => {
		keyHint.clear();
		keyHint.addKey('space', $t`ActionHub`);

		Promise.all([getVersion(), getTauriVersion()])
			.then(([appVer, tauriVer]) => {
				versions = { app: appVer, tauri: tauriVer, loading: false };
			})
			.catch(() => {
				versions = { app: 'N/A', tauri: 'N/A', loading: false };
			});

		return keyHint.register([['tab', $t`Navigate`]]);
	});

	function openActionHub() {
		window.dispatchEvent(new KeyboardEvent('keydown', { key: ' ' }));
		window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ' }));
	}
</script>

<div class="h-full w-full overflow-y-auto p-3">
	<VStack gap="md" class="mx-auto h-full max-w-6xl">
		<!-- Header -->
		<VStack gap="xs">
			<HStack gap="sm" align="center">
				<h1
					class="from-accent-500 to-accent-300 bg-linear-to-r bg-clip-text text-3xl font-bold text-transparent"
				>
					Palaxy
				</h1>
				<Badge variant="primary" class="text-xs">v{versions.app || '...'}</Badge>
			</HStack>
			<p class="text-muted text-sm">{$t`Convert and organize your manga collection with ease`}</p>
		</VStack>

		<!-- Main Grid -->
		<BentoGrid cols={3} density="comfortable" rows="2fr auto" class="flex-1">
			<!-- Primary Action: Convert -->
			<BentoItem
				colspan={2}
				rowspan={2}
				glass
				class="group relative min-h-0 overflow-hidden outline-none"
				onclick={() => goto('/convert')}
				data-keyhint={`enter;${$t`Start conversion`}`}
			>
				<div
					class="from-accent-500/10 absolute inset-0 bg-linear-to-br to-transparent opacity-0 transition-opacity duration-500 group-focus-within:opacity-100 group-hover:opacity-100"
				></div>

				<div class="relative z-10 flex h-full flex-col justify-between">
					<VStack gap="md">
						<HStack gap="sm" align="center">
							<div
								class="bg-accent-500/10 flex h-12 w-12 items-center justify-center rounded-full transition-all group-focus-within:scale-110 group-hover:scale-110"
							>
								<IconTransform size={24} class="text-accent-500" />
							</div>
							<VStack gap="none">
								<h2 class="text-xl font-bold">{$t`Manga Converter`}</h2>
								<Badge variant="success" class="mt-1 self-start text-xs">
									<IconSparkles size={12} />
									{$t`Ready`}
								</Badge>
							</VStack>
						</HStack>

						<p class="text-muted max-w-md text-sm leading-relaxed">
							{$t`Transform your manga images into professional digital formats. Supports CBZ and EPUB with automatic volume detection, image optimization, and batch processing.`}
						</p>
					</VStack>

					<Button
						size="lg"
						variant="primary"
						onclick={(e: MouseEvent) => {
							e.stopPropagation();
							goto('/convert');
						}}
						style="seamless"
						class="self-start"
						tabindex={-1}
					>
						<IconTransform size={20} />
						{$t`Start Converting`}
					</Button>
				</div>

				<!-- Character Art Decoration with Rotation Effect -->
				<img
					src="/holo/watame.png"
					alt="Character"
					class="pointer-events-none absolute -right-6 -bottom-10 h-[125%] object-contain opacity-15 grayscale transition-all
                           duration-500 group-focus-within:scale-105 group-focus-within:rotate-3 group-focus-within:opacity-50
                           group-focus-within:grayscale-0 group-hover:scale-105 group-hover:rotate-3 group-hover:opacity-50
                           group-hover:grayscale-0"
				/>
			</BentoItem>

			<!-- Secondary Actions Column -->
			<div class="flex flex-col gap-3">
				<!-- Action Hub -->
				<BentoItem
					glass
					onclick={openActionHub}
					data-keyhint={`space;${$t`Open`}`}
					class="group hover:border-accent-500/50 focus:border-accent-500/50 flex-1 cursor-pointer transition-all outline-none"
				>
					<div class="flex h-full flex-col justify-between">
						<VStack gap="xs">
							<HStack gap="sm" align="center">
								<div
									class="bg-accent-500/10 flex h-10 w-10 items-center justify-center rounded-lg transition-all duration-300 group-focus-within:scale-110 group-hover:scale-110"
								>
									<IconCommand size={20} class="text-accent-500" />
								</div>
								<VStack gap="none" class="flex-1">
									<h3 class="text-sm font-semibold">{$t`Action Hub`}</h3>
									<span class="text-muted text-xs">{$t`Quick commands`}</span>
								</VStack>
							</HStack>
						</VStack>
						<div
							class="bg-surface-2 group-hover:bg-accent-500/10 mt-3 rounded-lg p-2 text-center transition-colors"
						>
							<Badge variant="neutral" class="text-xs">
								<kbd class="font-mono">Space</kbd>
							</Badge>
						</div>
					</div>
				</BentoItem>

				<BentoItem
					glass
					onclick={() => goto('/settings')}
					data-keyhint={`enter;${$t`Open settings`}`}
					class="group hover:border-accent-500/50 focus:border-accent-500/50 flex-1 cursor-pointer transition-all outline-none"
				>
					<div class="flex h-full flex-col justify-between">
						<VStack gap="xs">
							<HStack gap="sm" align="center">
								<div
									class="bg-accent-500/10 flex h-10 w-10 items-center justify-center rounded-lg transition-all duration-300 group-focus-within:scale-110 group-hover:scale-110"
								>
									<IconSettings size={20} class="text-accent-500" />
								</div>
								<VStack gap="none" class="flex-1">
									<h3 class="text-sm font-semibold">{$t`Settings`}</h3>
									<span class="text-muted text-xs">{$t`Preferences`}</span>
								</VStack>
							</HStack>
						</VStack>
						<p class="text-muted mt-2 text-xs leading-relaxed">
							{$t`Configure theme, language, and automation presets`}
						</p>
					</div>
				</BentoItem>
			</div>

			<!-- Bottom Row: Features & System Info -->
			<BentoItem colspan={2} glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconBook size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Features`}</span>
				</HStack>

				<div class="grid grid-cols-3 gap-4">
					<VStack gap="xs">
						<div class="bg-accent-500/20 flex h-8 w-8 items-center justify-center rounded-lg">
							<IconTransform size={16} class="text-accent-500" />
						</div>
						<VStack gap="none">
							<span class="text-sm font-medium">{$t`Format Conversion`}</span>
							<span class="text-muted text-xs">{$t`CBZ & EPUB support`}</span>
						</VStack>
					</VStack>

					<VStack gap="xs">
						<div class="bg-accent-500/20 flex h-8 w-8 items-center justify-center rounded-lg">
							<IconSparkles size={16} class="text-accent-500" />
						</div>
						<VStack gap="none">
							<span class="text-sm font-medium">{$t`Auto Detection`}</span>
							<span class="text-muted text-xs">{$t`Volume & chapter bundling`}</span>
						</VStack>
					</VStack>

					<VStack gap="xs">
						<div class="bg-accent-500/20 flex h-8 w-8 items-center justify-center rounded-lg">
							<IconBook size={16} class="text-accent-500" />
						</div>
						<VStack gap="none">
							<span class="text-sm font-medium">{$t`Batch Processing`}</span>
							<span class="text-muted text-xs">{$t`Multiple volumes at once`}</span>
						</VStack>
					</VStack>
				</div>
			</BentoItem>

			<!-- System Status -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconSettings size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`System`}</span>
					<div
						class="bg-success ml-auto h-2 w-2 animate-pulse rounded-full shadow-[0_0_8px_rgba(34,197,94,0.6)]"
					></div>
				</HStack>

				{#if !versions.loading}
					<VStack gap="sm">
						<div class="bg-surface-2 rounded-lg p-3">
							<VStack gap="xs">
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Application`}</span>
									<span class="font-mono font-medium">v{versions.app}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">Tauri</span>
									<span class="font-mono font-medium">v{versions.tauri}</span>
								</HStack>
							</VStack>
						</div>
					</VStack>
				{/if}
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
