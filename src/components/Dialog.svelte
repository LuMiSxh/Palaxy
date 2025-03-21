<script lang="ts">
  import dialogManager from '$states/dialog.svelte';
  import DialogContent from '$components/DialogContent.svelte';
  // Use a reactive statement to ensure we're only showing the latest dialog
	let activeDialog = $derived.by(() => dialogManager.dialogs.length > 0 ? dialogManager.dialogs[dialogManager.dialogs.length - 1] : null);
</script>

<div class="fixed inset-0 z-50 w-[1px] h-[1px]">
  {#if activeDialog}
    <DialogContent dialog={activeDialog} />
  {/if}
</div>
