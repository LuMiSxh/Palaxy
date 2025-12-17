<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import dialogManager, { type Dialog } from '$states/dialog.svelte';
	import { IconX } from '@tabler/icons-svelte';
	import { scale } from 'svelte/transition';
	import { keyboard } from '$lib/keyboard';
	import { onMount } from 'svelte';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';

	// Waku Imports
	import { VStack, HStack, Separator } from 'waku/layout';
	import { Button } from 'waku/components';

	let { dialog }: { dialog: Dialog } = $props();
	let confirmButton: HTMLButtonElement | null = $state(null);
	let cancelButton: HTMLButtonElement | null = $state(null);

	function handleBackdropClick(event: MouseEvent | undefined = undefined) {
		event?.stopPropagation();
		event?.preventDefault();

		dialog.onCancel?.();
		dialogManager.closeDialog(dialog.id);
	}

	function handleEscapeKey(event: KeyboardEvent) {
		if (event.key === 'Escape' && dialogManager.dialogs.length > 0) {
			const currentDialog = dialogManager.dialogs[dialogManager.dialogs.length - 1];
			currentDialog.onCancel?.();
			dialogManager.closeDialog(currentDialog.id);
		}
	}

	onMount(() => {
		// Find all tabbable elements in the dialog
		const dialogElement = document.querySelector('[role="dialog"]');
		const tabbableElements = dialogElement?.querySelectorAll(
			'button:not([tabindex="-1"]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
		);

		// Focus the first tabbable element or confirm button
		if (tabbableElements && tabbableElements.length > 0) {
			(tabbableElements[0] as HTMLElement).focus();
		} else if (confirmButton) {
			confirmButton.focus();
		} else if (cancelButton) {
			cancelButton.focus();
		}

		// Focus the confirm-button by default
		if (confirmButton) {
			confirmButton.focus();
		} else if (cancelButton) {
			cancelButton.focus();
		}

		const unregisterKeyboard = keyboard.smartRegister([['escape', handleEscapeKey]]);

		const unregisterKeyHint = keyHint.register([
			['escape', $t`Close`],
			['tab', $t`Navigate fields`],
		]);

		return () => {
			unregisterKeyboard();
			unregisterKeyHint();
		};
	});
</script>

<button
	class="bg-background/90 dark:bg-background-dark/90 absolute inset-0 z-30 h-screen w-screen border-none"
	onclick={handleBackdropClick}
	onkeydown={(e) => e.key === 'Escape' && handleBackdropClick()}
	aria-label={$t`Close dialog`}
	tabIndex="-1"
></button>

<div
	class="bg-waku-surface backdrop-blur-glass border-waku-border fixed top-1/2 left-1/2 z-50 max-h-[88vh] min-h-[10vh] min-w-[50vw] -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-xl border shadow-2xl"
	transition:scale={{ duration: 200, start: 0.95 }}
	role="dialog"
	aria-modal="true"
>
	<VStack gap="none" class="h-full">
		<!-- Header -->
		<HStack align="center" justify="between" class="p-4">
			<h2 class="text-lg font-semibold">{dialog.title}</h2>
			{#if dialog.showClose !== false}
				<Button
					variant="ghost"
					size="sm"
					onclick={() => dialogManager.closeDialog(dialog.id)}
					tabIndex="-1"
					class="h-8 w-8 p-0"
				>
					<IconX size={20} />
				</Button>
			{/if}
		</HStack>

		<Separator class="my-0!" />

		<!-- Content -->
		<div class="max-h-[70vh] flex-1 overflow-y-auto p-4">
			{#if typeof dialog.content === 'string'}
				<p class="text-muted">{dialog.content}</p>
			{:else}
				<dialog.content {...dialog.contentProps || {}} bind:id={dialog.id} />
			{/if}
		</div>

		<!-- Actions -->
		{#if dialog.onCancel || dialog.onConfirm}
			<div class="border-waku-border border-t p-4">
				<HStack align="center" justify="end" gap="sm">
					{#if dialog.onCancel}
						<button
							class="waku-button waku-button--ghost"
							use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
							bind:this={cancelButton}
							onclick={() => {
								dialog.onCancel?.();
								dialogManager.closeDialog(dialog.id);
							}}
						>
							{dialog.cancelText || $t`Cancel`}
						</button>
					{/if}

					{#if dialog.onConfirm}
						<button
							class="waku-button waku-button--accent"
							use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
							bind:this={confirmButton}
							onclick={() => {
								dialog.onConfirm?.();
								dialogManager.closeDialog(dialog.id);
							}}
						>
							{dialog.confirmText || $t`Confirm`}
						</button>
					{/if}
				</HStack>
			</div>
		{/if}
	</VStack>
</div>

<style>
	.backdrop-blur-glass {
		backdrop-filter: blur(20px);
		background: rgba(var(--waku-surface-rgb, 255, 255, 255), 0.95);
	}

	:global(.dark) .backdrop-blur-glass {
		background: rgba(var(--waku-surface-rgb, 0, 0, 0), 0.95);
	}
</style>
