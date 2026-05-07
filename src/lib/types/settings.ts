export type AppSettings = {
	notesFolder: string;
	locale: string;
	theme: string;
	rememberWindowSize: boolean;
	notesListLayout: "list" | "masonry";
};

export type ThemeInfo = {
	id: string;
	name: string;
};
