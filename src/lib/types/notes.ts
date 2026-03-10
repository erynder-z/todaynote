export type FormattedNote = {
	filename: string;
	formattedName: string;
};

export type NoteLineData = {
	markdown: string;
	html: string;
};

export type NoteContentResponse = {
	lines: string[];
	metadata: Record<string, string>;
	metadataRange: [number, number] | null;
};

export type NoteLineProps = {
	markdown: string;
	isActive: boolean;
	onActivate: () => void;
	onDeactivate: (e: FocusEvent) => void;
	onChange: (markdown: string, html: string) => void;
	onKeyDown: (e: KeyboardEvent) => void;
};
