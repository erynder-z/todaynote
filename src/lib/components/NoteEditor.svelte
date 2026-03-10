<script lang="ts">
  /**
   * The main note display component. Displays a note's content in an editable form.
   */
  import { untrack } from 'svelte';
  import { NoteLine } from '$lib';
  import type { NoteContentResponse, NoteLineData } from '$lib/types/notes';
  import {
    deleteNoteLine,
    insertNoteLine,
    updateNoteLine,
  } from '$lib/utils/notes';

  let { noteContent, notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  /** Reactive array of line objects representing the note's content. */
  let lines = $state<NoteLineData[]>([]);
  /** Index of the line currently being edited. */
  let activeIndex = $state<number | null>(null);
  /** Tracked path to detect when a different note is loaded. */
  let lastLoadedPath = $state<string | null>(null);
  /** Index of the line that has unsaved changes. */
  let changedLineIndex = $state<number | null>(null);

  /**
   * Persists the content of a specific line to the backend.
   */
  const flush = async (index: number) => {
    if (lines[index]) {
      const content = lines[index].markdown;
      if (changedLineIndex === index) changedLineIndex = null;
      await updateNoteLine(index, content);
    }
  };

  /**
   * Parses the raw note content into an array of line objects.
   */
  const loadLines = () => {
    if (!noteContent) {
      lines = [];
      return;
    }
    lines = noteContent.lines.map((m: string) => ({ markdown: m, html: '' }));
  };

  /**
   * Checks if a line index is within the metadata/frontmatter range.
   */
  const isMetadataLine = (index: number) => {
    if (!noteContent?.metadataRange) return false;
    const [start, end] = noteContent.metadataRange;
    return index >= start && index <= end;
  };

  /**
   * Appends an empty line to the end of the note if it's missing or if
   * the last line contains text. This ensures the user always has a
   * clean starting point for new input.
   */
  const ensureTrailingEmptyLine = () => {
    const lastLine = lines[lines.length - 1];
    if (lines.length === 0 || lastLine.markdown.trim() !== '') {
      lines.push({ markdown: '', html: '' });
      insertNoteLine(lines.length - 1, '');
    }
  };

  /**
   * Synchronizes the component state when the active note changes.
   * This handles initial loading and prepares the UI for editing.
   */
  const syncProps = () => {
    if (notePath !== lastLoadedPath) {
      loadLines();
      ensureTrailingEmptyLine();

      lastLoadedPath = notePath;
      // Focus the last line (which we ensured is empty if needed)
      activeIndex = lines.length - 1;
      changedLineIndex = null;
    }
  };

  /**
   * Ensures unsaved changes are flushed when the user moves to a different line.
   */
  const autoFlushOnLineSwitch = () => {
    const current = activeIndex;
    untrack(() => {
      if (changedLineIndex !== null && changedLineIndex !== current) {
        flush(changedLineIndex);
      }
    });
  };

  /**
   * Automatically saves changes after a short delay of inactivity.
   */
  const debouncedAutoSave = () => {
    if (changedLineIndex === null || !lines[changedLineIndex]) return;

    const index = changedLineIndex;
    // Access markdown here so Svelte tracks it and resets the timer on every keystroke
    const _content = lines[index].markdown;

    const timeout = setTimeout(() => {
      untrack(() => {
        if (changedLineIndex === index) flush(index);
      });
    }, 500);

    return () => clearTimeout(timeout);
  };

  /**
   * Adds a new empty line immediately after the specified index.
   */
  const insertLine = async (i: number) => {
    lines.splice(i + 1, 0, { markdown: '', html: '' });
    activeIndex = i + 1;
    await insertNoteLine(i + 1, '');
  };

  /**
   * Removes the line at the specified index and shifts focus.
   */
  const deleteLine = async (i: number) => {
    lines.splice(i, 1);
    // Find previous visible line
    let nextIndex = i - 1;
    while (nextIndex >= 0 && isMetadataLine(nextIndex)) {
      nextIndex--;
    }
    activeIndex = Math.max(0, nextIndex);
    await deleteNoteLine(i);
  };

  /**
   * Moves the active line focus up or down, skipping hidden lines.
   */
  const navigateLines = (i: number, direction: 'up' | 'down') => {
    let nextIndex = direction === 'up' ? i - 1 : i + 1;

    while (
      nextIndex >= 0 &&
      nextIndex < lines.length &&
      isMetadataLine(nextIndex)
    ) {
      nextIndex = direction === 'up' ? nextIndex - 1 : nextIndex + 1;
    }

    if (nextIndex >= 0 && nextIndex < lines.length) {
      activeIndex = nextIndex;
    }
  };

  /**
   * Coordinates keyboard shortcuts for line editing and navigation.
   */
  const handleKeyDown = async (e: KeyboardEvent, i: number) => {
    switch (e.key) {
      case 'Enter':
        e.preventDefault();
        await insertLine(i);
        break;
      case 'Backspace':
        if (lines[i].markdown === '' && lines.length > 1) {
          e.preventDefault();
          await deleteLine(i);
        }
        break;
      case 'ArrowUp':
        e.preventDefault();
        navigateLines(i, 'up');
        break;
      case 'ArrowDown':
        e.preventDefault();
        navigateLines(i, 'down');
        break;
    }
  };

  /**
   * Updates the internal state of a line and marks it as changed.
   */
  const handleLineChange = (i: number, markdown: string) => {
    if (lines[i]) {
      lines[i].markdown = markdown;
      changedLineIndex = i;
    }
  };

  $effect.pre(() => syncProps());
  $effect(() => autoFlushOnLineSwitch());
  $effect(() => debouncedAutoSave());
</script>

<div class="note-container">
  {#each lines as line, i (i)}
    {#if !isMetadataLine(i)}
      <NoteLine
        bind:markdown={line.markdown}
        isActive={activeIndex === i}
        onActivate={() => (activeIndex = i)}
        onDeactivate={(e: FocusEvent) => {
          const target = e.relatedTarget as HTMLElement;
          if (!target?.closest('.note-container')) activeIndex = null;
        }}
        onChange={(markdown) => handleLineChange(i, markdown)}
        onKeyDown={(e) => handleKeyDown(e, i)}
      />
    {/if}
  {/each}
</div>

<style>
  .note-container {
    padding: 2rem;
    background-color: var(--bg-surface);
    border: 0.0625rem solid var(--border);
    color: var(--text-main);
    border-radius: 0.75rem;
    margin: 1rem 0;
    max-height: 70vh;
    overflow-y: auto;
  }

  .note-container :global(.rendered-line p) {
    margin: 0;
  }
  .note-container :global(.rendered-line h1) {
    font-size: 1.5rem;
  }
  .note-container :global(.rendered-line h2) {
    font-size: 1.3rem;
  }
  .note-container :global(.rendered-line h3) {
    font-size: 1.2rem;
  }

  .note-container :global(.rendered-line h1),
  .note-container :global(.rendered-line h2),
  .note-container :global(.rendered-line h3) {
    margin-top: 0.5rem;
    margin-bottom: 0.2rem;
    font-weight: 600;
  }

  .note-container :global(.rendered-line ul),
  .note-container :global(.rendered-line ol) {
    margin: 0;
    padding-left: 1.5rem;
  }

  .note-container :global(.rendered-line code) {
    background-color: var(--bg-main);
    padding: 0.2rem 0.4rem;
    border-radius: 0.3rem;
    font-family: monospace;
  }

  .note-container :global(.rendered-line pre) {
    background-color: var(--bg-main);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
  }

  .note-container :global(.rendered-line blockquote) {
    border-left: 0.25rem solid var(--border);
    margin: 0;
    padding-left: 1rem;
    color: var(--text-muted);
  }

  .note-container :global(.rendered-line a) {
    color: var(--accent);
    text-decoration: none;
  }
  .note-container :global(.rendered-line table) {
    border-collapse: collapse;
    width: 100%;
  }

  .note-container :global(.rendered-line th),
  .note-container :global(.rendered-line td) {
    border: 0.0625rem solid var(--border);
    padding: 0.5rem;
  }
</style>
