<script lang="ts">
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { platform, arch, version } from '@tauri-apps/plugin-os';

	let appName = $state('');
	let appVersion = $state('');
	let tauriVersion = $state('');
	let osName = $state('');
	let osArch = $state('');
	let osVersion = $state('');
	let loading = $state(true);
	let error = $state<Error | null>(null);

	onMount(async () => {
		try {
			// Fetch all system information in parallel
			const [name, ver, tauri, plat, architecture, osVer] = await Promise.all([
				getName(),
				getVersion(),
				getTauriVersion(),
				platform(),
				arch(),
				version()
			]);

			appName = name;
			appVersion = ver;
			tauriVersion = tauri;
			osName = plat;
			osArch = architecture;
			osVersion = osVer;
		} catch (e) {
			error = e as Error;
		} finally {
			loading = false;
		}
	});
</script>

<div class="p-6 max-w-3xl mx-auto">
	{#if loading}
		<div class="flex items-center justify-center p-6">
			<div class="animate-pulse">{$t`Loading system information...`}</div>
		</div>
	{:else if error}
		<div class="alert alert-error">{error.message}</div>
	{:else}
		<div class="w-full h-full flex gap-2">
			<!-- Application Information -->
			<div class="px-2">
				<h3 class="text-lg font-medium mb-1">{$t`Application Information`}</h3>
				<div class="overflow-x-auto h-full pb-2">
					<table class="table w-full h-full">
						<tbody>
						<tr>
							<td>
								{$t`Application Name`}
							</td>
							<td>
								{appName}
							</td>
						</tr>
						<tr>
							<td>
								{$t`Application Version`}
							</td>
							<td>
								{appVersion}
							</td>
						</tr>
						<tr>
							<td>
								{$t`Tauri Version`}
							</td>
							<td>
								{tauriVersion}
							</td>
						</tr>
						</tbody>
					</table>
				</div>
			</div>

			<!-- System Information -->
			<div class="px-2">
				<h3 class="text-lg font-medium mb-1">{$t`System Information`}</h3>
				<div class="overflow-x-auto h-full pb-2">
					<table class="table w-full h-full">
					    <tbody>
					    <tr>
					        <td>{$t`Operating System`}</td>
					        <td>{osName}</td>
					    </tr>
					    <tr>
					        <td>{$t`Architecture`}</td>
					        <td>{osArch}</td>
					    </tr>
					    <tr>
					        <td>{$t`OS Version`}</td>
					        <td>{osVersion}</td>
					    </tr>
					    </tbody>
					</table>
				</div>
			</div>
		</div>
	{/if}
</div>
