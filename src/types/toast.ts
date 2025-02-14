export default interface Toast {
	id: number;
	message: string;
	type: "success" | "error" | "warning" | "info";
	timeout: number | null;
}
