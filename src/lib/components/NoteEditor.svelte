<script lang="ts">
  /**
   * The main note display component. Displays a note's content in an editable form.
   */
  import {
    defaultValueCtx,
    Editor,
    editorViewCtx,
    parserCtx,
    rootCtx,
  } from '@milkdown/core';
  import { listener, listenerCtx } from '@milkdown/plugin-listener';
  import { commonmark } from '@milkdown/preset-commonmark';
  import { keymap } from '@milkdown/prose/keymap';
  import { Selection } from '@milkdown/prose/state';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { untrack } from 'svelte';
  import { jumpToSectionInEditor, sessionState, useShortcuts } from '$lib';
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
   * Syncs store content to editor if it changed from outside (e.g. backend load)
   */
  $effect(() => {
    const content = editor.content;
    const instance = milkdownInstance;

    if (instance && editor.pendingExternalUpdate) {
      instance.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const parser = ctx.get(parserCtx);
        const doc = parser(content);
        if (doc) {
          const tr = view.state.tr.replaceWith(
            0,
            view.state.doc.content.size,
            doc,
          );
          view.dispatch(tr);
        }
      });
      editor.pendingExternalUpdate = false;
    }
  });

  /**
   * Main entry point for jumping to a section.
   */
  const handleJump = async (name: string) => {
    const instance = milkdownInstance;
    if (!instance) return;

    // Check if section already exists in our current list
    const exists = editor.sections.some((s) => s.name === name);

    if (exists) {
      jumpToSectionInEditor(instance, name);
    } else {
      // Create it via backend, wait for sync, then jump
      await editor.ensureSectionExists(name);
      // Increased delay for reliability during document reconstruction
      setTimeout(() => jumpToSectionInEditor(instance, name), 100);
    }
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

  const customKeymap = prosePlugin(() =>
    keymap({
      'Mod-1': () => true,
      'Mod-2': () => true,
    }),
  );

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
    if (!editorContainer || !notePath) return;

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
      .use(customKeymap)
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

  // Connect the store's sync back to the component's bindable props
  editor.onJump = (updated) => {
    noteContent = updated;
  };
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
