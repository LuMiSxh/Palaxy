export default interface Command {
	name: string;
	description: string;
	icon: any;
	action?: () => void | any;
	hidden?: boolean;
	subcommands?: Command[];
}
