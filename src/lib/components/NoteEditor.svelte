<script lang="ts">
  /**
   * The main note display component. Displays a note's content in an editable form.
   */
  import { untrack } from 'svelte';
  import { sessionState, useShortcuts } from '$lib';
  import { EditorStore } from '$lib/stores/editor.svelte';
  import type { NoteContentResponse } from '$lib/types/notes';
  import NoteHeader from './NoteHeader.svelte';

  let { noteContent = $bindable(), notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  // Initialize the central logic store
  const editor = new EditorStore();

  let textarea: HTMLTextAreaElement | null = $state(null);

  /**
   * Updates the UI state after a jump.
   */
  const handleJumpResult = (updated: NoteContentResponse) => {
    noteContent = updated;

    // Set cursor position after content update
    setTimeout(() => {
      if (textarea && editor.cursorPosition !== null) {
        textarea.focus();
        textarea.setSelectionRange(
          editor.cursorPosition,
          editor.cursorPosition,
        );
        editor.cursorPosition = null;
      }
    }, 10);
  };

  // Connect the store's jump event back to the component's bindable props
  editor.onJump = handleJumpResult;

  /**
   * Triggers a jump to a named section and updates the editor state.
   */
  const handleJump = async (name: string) => {
    await editor.jumpToSection(name);
  };

  /**
   * Jumps to a section based on its index (0-8).
   */
  const jumpToSectionByIndex = async (idx: number) => {
    const section = editor.sections[idx];
    if (section?.name) await handleJump(section.name);
  };

  useShortcuts({
    focusLastLine: () => {
      if (sessionState.activePopup === null) {
        textarea?.focus();
      }
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup === null) {
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
   * Returns focus to the editor when a popup is closed.
   */
  $effect(() => {
    if (sessionState.activePopup === null && notePath)
      untrack(() => textarea?.focus());
  });

  /**
   * Auto-resize textarea to fit content
   */
  const autoResize = () => {
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = `${textarea.scrollHeight}px`;
    }
  };

  $effect(() => {
    autoResize();
  });
</script>

<div class="note-container">
  <NoteHeader {noteContent} />
  <textarea
    bind:this={textarea}
    value={editor.content}
    oninput={(e) => {
      editor.updateContent(e.currentTarget.value);
      autoResize();
    }}
    spellcheck="false"
    placeholder="Start writing..."
  ></textarea>
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

  textarea {
    width: 100%;
    min-height: 60dvh;
    background: transparent;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
    resize: none;
    padding: 0;
    margin: 0;
    outline: none;
    overflow: hidden;
    display: block;
  }

  textarea::placeholder {
    color: var(--text-muted);
    opacity: 0.5;
  }

  @media (max-width: 480px) {
    .note-container {
      min-height: 100dvh;
    }
  }
</style>
