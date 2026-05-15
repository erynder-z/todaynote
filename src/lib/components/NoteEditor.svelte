<script lang="ts">
  /**
   * The main note editor component. Coordinates the Milkdown instance with the EditorStore.
   * Handles high-level actions like shortcuts and thread navigation.
   */
  import type { Editor } from '@milkdown/core';
  import { keymap } from '@milkdown/prose/keymap';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { untrack } from 'svelte';
  import type { NoteContentResponse, NoteThread } from '$lib/types/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import type { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import {
    navigateToLastAvailable,
    navigateToOffset,
  } from '../utils/dailyNote';
  import {
    focusEditor,
    jumpToThreadInEditor,
    updateEditorContent,
  } from '../utils/editor';
  import { useShortcuts } from '../utils/shortcuts';
  import MilkdownEditor from './MilkdownEditor.svelte';

  let {
    noteContent = $bindable(),
    notePath,
    editor,
  } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
    editor: EditorStore;
  }>();

  let milkdownInstance: Editor | null = $state(null);

  // --- Lifecycle & Reactive Effects ---

  /**
   * 1. Sync props to the internal store before rendering
   */
  $effect.pre(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * 2. Coordinate reactive updates (content sync and focus)
   */
  $effect(() => {
    const instance = milkdownInstance;
    if (!instance) return;

    if (editor.pendingExternalUpdate) {
      updateEditorContent(instance, editor.content);
      editor.pendingExternalUpdate = false;
    }

    if (sessionState.activePopup === null && notePath)
      untrack(() => focusEditor(instance));
  });

  // --- Actions & Helpers ---

  /**
   * Main entry point for jumping to a thread.
   */
  const handleJump = async (name: string) => {
    const instance = milkdownInstance;
    if (!instance) return;

    const exists = editor.threads.some((s: NoteThread) => s.name === name);
    if (exists) {
      jumpToThreadInEditor(instance, name);
    } else {
      await editor.ensureThreadExists(name);
      // Wait for the next tick/update to ensure thread is rendered before jumping
      setTimeout(() => {
        if (milkdownInstance) jumpToThreadInEditor(milkdownInstance, name);
      }, 100);
    }
  };

  /**
   * Jumps to a thread based on its index (0-8).
   */
  const jumpToThreadByIndex = async (idx: number) => {
    const thread = editor.threads[idx];
    if (thread?.name) await handleJump(thread.name);
  };

  useShortcuts({
    focusLastLine: () => {
      if (sessionState.activePopup !== null) return false;
      if (milkdownInstance) focusEditor(milkdownInstance);
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup !== null) return false;

      const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
      if (idx !== -1 && idx < editor.threads.length) jumpToThreadByIndex(idx);
    },
    navigateYesterday: async (e) => {
      if (sessionState.activePopup !== null) {
        e.preventDefault();
        return;
      }
      await navigateToOffset(-1);
    },
    navigateLastAvailable: async (e) => {
      if (sessionState.activePopup !== null) {
        e.preventDefault();
        return;
      }
      await navigateToLastAvailable();
    },
    navigateToday: async (e) => {
      if (sessionState.activePopup !== null) {
        e.preventDefault();
        return;
      }
      await navigateToOffset(0);
    },
  });

  const customKeymap = prosePlugin(() =>
    keymap({
      'Mod-1': () => true,
      'Mod-2': () => true,
    }),
  );

  // Connect the store's sync back to the component's bindable props
  $effect(() => {
    editor.onJump = (updated: NoteContentResponse) => (noteContent = updated);
  });

  // Expose jump functionality to parent components
  $effect(() => {
    editor.jumpToThread = (name: string) => {
      const instance = milkdownInstance;
      if (instance) jumpToThreadInEditor(instance, name);
    };
  });
</script>

{#if notePath}
  <MilkdownEditor
    content={editor.content}
    bind:instance={milkdownInstance}
    onUpdate={(markdown) => editor.updateContent(markdown)}
    plugins={[customKeymap]}
  />
{/if}
