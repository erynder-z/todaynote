export interface AppSettings {
	notesFolder: string;
	locale: string;
	theme: string;
	rememberWindowSize: boolean;
	notesListLayout: "list" | "masonry";
}

export interface ThemeInfo {
	id: string;
	name: string;
}
