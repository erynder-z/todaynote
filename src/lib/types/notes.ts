export type FormattedNote = {
	filename: string;
	formatted_name: string;
};

export type NoteLineData = {
	markdown: string;
	html: string;
};

export type NoteLineProps = {
	markdown: string;
	isActive: boolean;
	onActivate: () => void;
	onDeactivate: (e: FocusEvent) => void;
	onChange: (markdown: string, html: string) => void;
	onKeyDown: (e: KeyboardEvent) => void;
};
