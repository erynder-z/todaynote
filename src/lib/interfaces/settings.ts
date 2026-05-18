export interface AppSettings {
	notesFolder: string;
	locale: string;
	theme: string;
	rememberAppLayout: boolean;
	notesListLayout: "list" | "masonry";
	rememberSettings: boolean;
	searchMode: "notes" | "threads" | "tags";
	searchIsFuzzy: boolean;
	searchSelectedTag: string | null;
	controlCenterWidth: number;
}

export interface ThemeInfo {
	id: string;
	name: string;
}
