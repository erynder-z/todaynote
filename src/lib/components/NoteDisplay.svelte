<script lang="ts">
  import { untrack } from 'svelte';
  import { NoteLine } from '$lib';
  import type { NoteLineData } from '$lib/types/notes';
  import { saveNoteContent } from '$lib/utils/notes';

  let { noteContent, notePath } = $props<{
    noteContent: string;
    notePath: string | null;
  }>();

  // Initial state
  let lines = $state<NoteLineData[]>([]);
  let activeIndex = $state<number | null>(null);
  let lastLoadedPath = $state<string | null>(null);

  // Derived markdown for comparison and saving
  const currentMarkdown = $derived(lines.map((l) => l.markdown).join('\n'));

  const syncProps = () => {
    const isNewPath = notePath !== lastLoadedPath;
    const isExternalUpdate = noteContent !== untrack(() => currentMarkdown);

    if (isNewPath || isExternalUpdate) {
      lines = (noteContent || '')
        .split('\n')
        .map((m: string) => ({ markdown: m, html: '' }));
      lastLoadedPath = notePath;
      if (isNewPath) activeIndex = null;
    }
  };

  const debounceSave = () => {
    if (!notePath || lines.length === 0) return;

    const contentToSave = currentMarkdown;
    const timeout = setTimeout(() => {
      untrack(() => saveNoteContent(notePath, contentToSave));
    }, 500);

    return () => clearTimeout(timeout);
  };

  const insertLine = (e: KeyboardEvent, i: number) => {
    e.preventDefault();
    lines.splice(i + 1, 0, { markdown: '', html: '' });
    activeIndex = i + 1;
  };

  const deleteLine = (e: KeyboardEvent, i: number) => {
    if (lines[i].markdown === '' && lines.length > 1) {
      e.preventDefault();
      lines.splice(i, 1);
      activeIndex = Math.max(0, i - 1);
    }
  };

  const navigateLines = (e: KeyboardEvent, i: number) => {
    const direction = e.key === 'ArrowUp' ? -1 : 1;
    const newIndex = i + direction;

    if (newIndex >= 0 && newIndex < lines.length) {
      e.preventDefault();
      activeIndex = newIndex;
    }
  };

  const handleKeyDown = (e: KeyboardEvent, i: number) => {
    if (e.key === 'Enter') {
      insertLine(e, i);
    } else if (e.key === 'Backspace') {
      deleteLine(e, i);
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      navigateLines(e, i);
    }
  };

  $effect.pre(() => syncProps());

  $effect(() => debounceSave());
</script>

<div class="note-container">
  {#each lines as line, i (i)}
    <NoteLine
      bind:markdown={line.markdown}
      isActive={activeIndex === i}
      onActivate={() => (activeIndex = i)}
      onDeactivate={(e: FocusEvent) => {
        const target = e.relatedTarget as HTMLElement;
        if (!target?.closest('.note-container')) activeIndex = null;
      }}
      onChange={(_, html) => (line.html = html)}
      onKeyDown={(e) => handleKeyDown(e, i)}
    />
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
