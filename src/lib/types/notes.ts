export type FormattedNote = {
	filename: string;
	formattedName: string;
};

export type NoteLineData = {
	markdown: string;
	html: string;
	sectionShortcut?: string;
};

export type NoteMetadata = {
	formattedDate: string;
	tags: string[];
	raw: Record<string, string>;
};

export type NoteSection = {
	name: string;
	startLine: number;
	endLine: number;
	level: number;
	shortcut?: string;
};

export type NoteContentResponse = {
	lines: string[];
	metadata: NoteMetadata;
	sections: NoteSection[];
	targetIndex?: number;
};

export type NoteLineProps = {
	markdown: string;
	sectionShortcut?: string;
	isActive: boolean;
	onActivate: () => void;
	onDeactivate: (e: FocusEvent) => void;
	onChange: (markdown: string, html: string) => void;
	onKeyDown: (e: KeyboardEvent) => void;
};
