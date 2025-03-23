<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import dialogManager, { type Dialog } from '$states/dialog.svelte';
	import { IconX } from '@tabler/icons-svelte';
	import { scale } from 'svelte/transition';
	import { keyboard } from '$lib/keyboard';
	import { onMount } from 'svelte';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';

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

		const unregisterKeyboard = keyboard.smartRegister([
			['escape', handleEscapeKey]
		]);

		const unregisterKeyHint = keyHint.smartAdd([
			['enter', $t`Invoke`],
			['escape', $t`Close`],
			['tab', $t`Navigate between fields`]
		]);

		return () => {
			unregisterKeyboard();
			unregisterKeyHint();
		};
	});
</script>

<button
	class="absolute inset-0 w-screen h-screen border-none bg-background/90 dark:bg-background-dark/90 z-30"
	onclick={handleBackdropClick}
	onkeydown={(e) => e.key === 'Escape' && handleBackdropClick()}
	aria-label={$t`Close dialog`}
	tabIndex="-1"
></button>

<div
	class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-background dark:bg-background-dark card shadow-lg z-50 max-h-[80vh] overflow-y-auto"
	transition:scale={{ duration: 200, start: 0.95 }}
	role="dialog"
	aria-modal="true"
>
	<div class="p-4 flex justify-between items-center">
		<h2 class="text-lg font-semibold text-content-primary">{dialog.title}</h2>
		{#if dialog.showClose !== false}
			<button
				class="btn btn-ghost btn-sm p-1"
				onclick={() => dialogManager.closeDialog(dialog.id)}
				tabIndex="-1"
			>
				<IconX size="20" />
			</button>
		{/if}
	</div>

	<div class="divider mx-4 my-0"></div>

	<div class="p-4">
		{#if typeof dialog.content === 'string'}
			<p class="text-content-tertiary">{dialog.content}</p>
		{:else}
			<dialog.content {...(dialog.contentProps || {})} bind:id={dialog.id} />
		{/if}
	</div>

	{#if dialog.onCancel || dialog.onConfirm}
		<div class="p-4 flex justify-end gap-2">
			{#if dialog.onCancel}
				<button
					class="btn btn-neutral"
					use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
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
					class="btn btn-primary"
					use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
					bind:this={confirmButton}
					onclick={() => {
                dialog.onConfirm?.();
                dialogManager.closeDialog(dialog.id);
              }}
				>
					{dialog.confirmText || $t`Confirm`}
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
    .card {
        width: 50vw;
        padding: 12px;
        min-height: 200px;
        border-radius: 8px;
    }
</style>
