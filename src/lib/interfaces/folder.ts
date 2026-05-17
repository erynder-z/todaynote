export interface FolderValidation {
	isValid: boolean;
	isWritable: boolean;
	exists: boolean;
	noteCount: number;
	error: string | null;
}
