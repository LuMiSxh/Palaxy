import type { PageLoad } from './$types';
import { bridge } from "$lib/functions"
import type { BaseResult } from "$lib/types"

type agent = {name: string, icon: string | null, url: string, tags: string[]}

interface AgentListResult extends BaseResult {
	agents: agent[]
}

export const load: PageLoad = async ({ params }) => {
	console.log(params);
	return {
		agents: (await bridge<AgentListResult>("get_agent_list"))?.agents ?? []
	};
};
