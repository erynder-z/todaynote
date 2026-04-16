<script lang="ts">
  /**
   * The main note display component. Displays a note's content in an editable form.
   */
  import {
    defaultValueCtx,
    Editor,
    editorViewCtx,
    rootCtx,
  } from '@milkdown/core';
  import { listener, listenerCtx } from '@milkdown/plugin-listener';
  import { commonmark } from '@milkdown/preset-commonmark';
  import { Selection, TextSelection } from '@milkdown/prose/state';
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
  let editorContainer: HTMLDivElement | null = $state(null);
  let milkdownInstance: Editor | null = $state(null);

  /**
   * Applies the cursor position from the store to the Milkdown editor.
   */
  $effect(() => {
    const pos = editor.cursorPosition;
    const instance = milkdownInstance;

    if (instance && pos !== null) {
      setTimeout(() => {
        instance.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          view.focus();

          try {
            const tr = view.state.tr;

            const pmPosition = pos + 1;

            const safePosition = Math.min(
              pmPosition,
              view.state.doc.content.size - 1,
            );

            const resolvedPos = view.state.doc.resolve(safePosition);

            const selection = Selection.near(resolvedPos);

            view.dispatch(tr.setSelection(selection));
            view.dispatch(view.state.tr.scrollIntoView());
          } catch (e) {
            console.warn('Could not set cursor position', e);
          }
        });
        editor.cursorPosition = null;
      }, 10);
    }
  });

  /**
   * Updates the UI state after a jump.
   */
  const handleJumpResult = (updated: NoteContentResponse) => {
    noteContent = updated;
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
      const instance = milkdownInstance;
      if (sessionState.activePopup === null && instance) {
        instance.action((ctx) => {
          ctx.get(editorViewCtx).focus();
        });
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
    const instance = milkdownInstance;
    if (sessionState.activePopup === null && notePath && instance) {
      untrack(() => {
        instance.action((ctx) => ctx.get(editorViewCtx).focus());
      });
    }
  });

  /**
   * Mount and manage the Milkdown editor lifecycle
   */
  $effect(() => {
    if (!editorContainer) return;

    let isDestroyed = false;

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, editorContainer);

        ctx.set(
          defaultValueCtx,
          untrack(() => editor.content),
        );

        ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
          editor.updateContent(markdown);
        });
      })
      .use(commonmark)
      .use(listener)
      .create()
      .then((instance) => {
        if (isDestroyed) {
          instance.destroy();
        } else {
          milkdownInstance = instance;
        }
      });

    return () => {
      isDestroyed = true;
      milkdownInstance?.destroy();
      milkdownInstance = null;
    };
  });
</script>

<div class="note-container">
  <NoteHeader {noteContent} />

  <div bind:this={editorContainer} class="milkdown-editor-wrapper"></div>
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

  .milkdown-editor-wrapper :global(.milkdown) {
    width: 100%;
    min-height: 60dvh;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
  }

  .milkdown-editor-wrapper :global(.milkdown .editor) {
    outline: none;
  }

  .milkdown-editor-wrapper :global(.milkdown ul),
  .milkdown-editor-wrapper :global(.milkdown ol) {
    padding-left: 3rem;
    margin: 1rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown li) {
    margin: 0.25rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown ul) {
    list-style-type: disc;
  }

  .milkdown-editor-wrapper :global(.milkdown ol) {
    list-style-type: decimal;
  }

  @media (max-width: 480px) {
    .note-container {
      min-height: 100dvh;
    }
  }
</style>
