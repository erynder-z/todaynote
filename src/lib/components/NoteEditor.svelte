<script lang="ts">
  /**
   * The main note editor component. Coordinates the Milkdown instance with the EditorStore.
   * Handles high-level actions like shortcuts and section navigation.
   */
  import type { Editor } from '@milkdown/core';
  import { keymap } from '@milkdown/prose/keymap';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { untrack } from 'svelte';
  import type { NoteContentResponse, NoteSection } from '$lib/types/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import type { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import {
    navigateToLastAvailable,
    navigateToOffset,
  } from '../utils/dailyNote';
  import {
    focusEditor,
    jumpToSectionInEditor,
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
   * Main entry point for jumping to a section.
   */
  const handleJump = async (name: string) => {
    const instance = milkdownInstance;
    if (!instance) return;

    const exists = editor.sections.some((s: NoteSection) => s.name === name);
    if (exists) {
      jumpToSectionInEditor(instance, name);
    } else {
      await editor.ensureSectionExists(name);
      // Wait for the next tick/update to ensure section is rendered before jumping
      setTimeout(() => {
        if (milkdownInstance) jumpToSectionInEditor(milkdownInstance, name);
      }, 100);
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
      if (sessionState.activePopup !== null) return false;
      if (milkdownInstance) focusEditor(milkdownInstance);
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup !== null) return false;

      const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
      if (idx !== -1 && idx < editor.sections.length) jumpToSectionByIndex(idx);
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
    editor.jumpToSection = (name: string) => {
      const instance = milkdownInstance;
      if (instance) jumpToSectionInEditor(instance, name);
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
