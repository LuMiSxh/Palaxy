<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onDestroy, onMount } from 'svelte';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';
	import { addToast } from '$states/toast.svelte';
	import {
		IconClock,
		IconFiles,
		IconPlayerPause,
		IconPlayerPlay,
		IconRefresh
	} from '@tabler/icons-svelte';
	import { commands, type LogPath, type SyncStatus } from '$types';
	import { wrapper } from '$lib/utils';

	let { id = $bindable(null) } = $props();

	// Logs section
	let logsPath: LogPath = $state({ directory: '', file: '' });

	// Sync section
	let syncStatus: SyncStatus | null = $state(null);

	let keyHintDestroy: () => void;

	onMount(async () => {
		await fetchLogsPath();
		await fetchSyncStatus();

		keyHintDestroy = keyHint.smartAdd([
			['arrowup', $t`Scroll up`],
			['arrowdown', $t`Scroll down`]
		]);
	});

	onDestroy(() => {
		if (keyHintDestroy) {
			keyHintDestroy();
		}
	});

	async function fetchLogsPath() {
		const response = await wrapper(commands.mgmtGetLogsPath());

		if (response && response.payload) {
			logsPath = response.payload;
		}
	}

	async function fetchSyncStatus() {
		const response = await wrapper(commands.mgmtSyncStatus());

		if (response) {
			syncStatus = response.payload;
		}
	}

	async function startSync() {
		const response = await wrapper(commands.mgmtSyncStart(null));

		if (response) {
			addToast($t`Sync started`, 'success');

			await fetchSyncStatus();
		}
	}

	async function stopSync() {
		const response = await wrapper(commands.mgmtSyncStop());
		if (response) {
			addToast($t`Sync stopped`, 'success');
			await fetchSyncStatus();
		}
	}

	function formatTimeRemaining(seconds: number): string {
		if (seconds <= 0) return $t`Imminent`;

		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		const remainingSeconds = seconds % 60;

		if (hours > 0) {
			return `${hours}h ${minutes}m ${remainingSeconds}s`;
		} else if (minutes > 0) {
			return `${minutes}m ${remainingSeconds}s`;
		} else {
			return `${remainingSeconds}s`;
		}
	}
</script>

<div class="">
	<!-- Tab Content -->
	<div class="tab-content">
		<div class="grid gap-5">
			<div class="flex items-center">
				<IconClock class="text-primary mr-2" size={20} />
				<h3 class="text-lg font-semibold">{$t`Sync Status`}</h3>
			</div>

			<div class="">
				<div class="mb-2 flex items-center justify-between">
					<span class="text-base">{$t`Next sync in:`}</span>
					<span class="text-primary text-base font-semibold">
						{formatTimeRemaining(syncStatus?.minutes_until_next_sync ?? -1)}
					</span>
				</div>
			</div>

			<div class="flex justify-end gap-3">
				<button
					class="btn btn-soft"
					onclick={fetchSyncStatus}
					use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
				>
					<IconRefresh size={20} />
					{$t`Refresh`}
				</button>
				<button
					class="btn btn-outline"
					onclick={stopSync}
					use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
				>
					<IconPlayerPause size={20} />
					{$t`Stop Sync`}
				</button>
				<button
					class="btn btn-primary"
					onclick={startSync}
					use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
				>
					<IconPlayerPlay size={20} />
					{$t`Start Sync`}
				</button>
			</div>
		</div>
		<div class="divider"></div>
		<div class="grid gap-5">
			<div class="flex items-center">
				<IconFiles class="text-primary mr-2" size={20} />
				<h3 class="text-lg font-semibold">{$t`Application Logs`}</h3>
			</div>

			<div class="">
				<p class="mb-2 text-base">
					{$t`Application logs are stored on your device. You can access them through your file explorer.`}
				</p>

				{#if logsPath.directory}
					<div class="mb-2">
						<p class="text-base font-medium">{$t`Log Directory:`}</p>
						<p class="text-content-secondary dark:text-content-dark-secondary text-base break-all">
							{logsPath.directory}
						</p>
					</div>
				{/if}

				{#if logsPath.file}
					<div class="">
						<p class="text-base font-medium">{$t`Log File:`}</p>
						<p class="text-content-secondary dark:text-content-dark-secondary text-base break-all">
							{logsPath.file}
						</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
