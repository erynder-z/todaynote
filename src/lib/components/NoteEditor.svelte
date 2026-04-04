<script lang="ts">
  /**
   * The main note display component. Displays a note's content in an editable form.
   */
  import { untrack } from 'svelte';
  import { NoteLine, sessionState, useShortcuts } from '$lib';
  import { EditorStore } from '$lib/stores/editor.svelte';
  import type { NoteContentResponse } from '$lib/types/notes';
  import { jumpToSection } from '$lib/utils/notes';
  import NoteHeader from './NoteHeader.svelte';

  let { noteContent = $bindable(), notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  // Initialize the central logic store
  const editor = new EditorStore();

  /**
   * Updates the UI state after a jump or slash command.
   */
  const handleJumpResult = (updated: NoteContentResponse) => {
    noteContent = updated;

    if (updated.targetIndex !== undefined) {
      setTimeout(() => (editor.activeIndex = updated.targetIndex ?? null), 10);
    } else {
      setTimeout(() => {
        editor.activeIndex = null;
        editor.focusLastLine();
      }, 10);
    }
  };

  // Connect the store's jump event back to the component's bindable props
  editor.onJump = handleJumpResult;

  /**
   * Triggers a jump to a named section and updates the editor state.
   */
  const handleJump = async (name: string) => {
    const updated = await jumpToSection(name);
    if (updated) handleJumpResult(updated);
  };

  /**
   * Jumps to a section based on its index (0-8).
   */
  const jumpToSectionByIndex = (idx: number) => {
    if (noteContent?.sections?.[idx]?.name)
      handleJump(noteContent.sections[idx].name);
  };

  useShortcuts({
    focusLastLine: () => {
      if (sessionState.activePopup === null) editor.focusLastLine();
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup === null) {
        // Use e.code (e.code.match(/Digit(\d)/)) for reliable number key detection
        const match = e.code.match(/Digit(\d)/);
        if (match) {
          const num = parseInt(match[1], 10);
          if (num > 0 && num <= 9) jumpToSectionByIndex(num - 1);
        }
      }
    },
  });

  /**
   * Sync props to the store
   */
  $effect.pre(() => editor.sync(noteContent, notePath));

  /**
   * Handle flushing changes when switching lines
   */
  $effect(() => editor.handleLineSwitch(editor.activeIndex));

  /**
   * Returns focus to the editor when a popup is closed.
   */
  $effect(() => {
    if (sessionState.activePopup === null && notePath)
      untrack(() => editor.focusLastLine());
  });
</script>

<div class="note-container">
  <NoteHeader {noteContent} />
  {#each editor.lines as line, i (i)}
    <NoteLine
      bind:markdown={line.markdown}
      sectionShortcut={line.sectionShortcut}
      isActive={editor.activeIndex === i}
      onActivate={() => (editor.activeIndex = i)}
      onDeactivate={(e: FocusEvent) => {
        const target = e.relatedTarget as HTMLElement;
        if (!target?.closest('.note-container')) editor.activeIndex = null;
      }}
      onChange={(markdown) => editor.updateLine(i, markdown)}
      onKeyDown={async (e) => {
        if (await editor.handleKeyDown(e, i)) e.preventDefault();
      }}
    />
  {/each}
</div>

<style>
  .note-container {
    width: clamp(20rem, 90%, 70ch);
    min-height: 85dvh;
    height: auto;
    flex-shrink: 0;
    padding: 3rem 1rem;
    background-color: var(--bg-surface);
    border: 0.0625rem solid var(--border);
    color: var(--text-main);
    box-shadow: 0 0.125rem 0.25rem rgba(0, 0, 0, 0.05);
  }

  .note-container :global(.rendered-line) {
    line-height: 1.6;
    font-size: 1rem;
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
    background-color: var(--bg-base);
    padding: 0.2rem 0.4rem;
    border-radius: 0.3rem;
    font-family: monospace;
  }

  .note-container :global(.rendered-line pre) {
    background-color: var(--bg-base);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    white-space: pre-wrap;
  }

  .note-container :global(.rendered-line table) {
    width: 100%;
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

  @media (max-width: 480px) {
    .note-container {
      min-height: 100dvh;
    }
  }
</style>
