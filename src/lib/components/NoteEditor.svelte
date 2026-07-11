<script lang="ts">
  /**
   * The main note editor component. Coordinates the Milkdown instance with the EditorStore.
   * Handles high-level actions like shortcuts and thread navigation.
   */
  import type { Editor } from '@milkdown/core';
  import { keymap } from '@milkdown/prose/keymap';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { tick, untrack } from 'svelte';
  import type { NoteContentResponse, NoteThread } from '$lib/interfaces/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import type { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import {
    navigateToLastAvailable,
    navigateToOffset,
  } from '../utils/dailyNote';
  import { EditorService } from '../utils/editor';
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
  let editorService: EditorService | null = $state(null);

  /**
   * Clean up Milkdown editor instance when component is destroyed
   * to prevent memory leaks
   */
  $effect(() => {
    return () => {
      if (milkdownInstance) milkdownInstance.destroy();
    };
  });

  /**
   * Update editor service when milkdown instance changes
   */
  $effect(() => {
    editorService = milkdownInstance
      ? new EditorService(milkdownInstance)
      : null;
  });

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
    const service = editorService;
    if (!instance || !service) return;

    if (editor.pendingExternalUpdate) {
      service.updateContent(editor.content);
      editor.pendingExternalUpdate = false;
    }

    if (sessionState.activePopup === null && notePath)
      untrack(() => service.focus());
  });

  /**
   * Main entry point for jumping to a thread.
   */
  const handleJump = async (threadId: string) => {
    const instance = milkdownInstance;
    if (!instance || !editorService) return;

    const threadIndex = editor.threads.findIndex(
      (nt: NoteThread) => nt.id === threadId,
    );
    if (threadIndex !== -1) {
      editorService.jumpToThreadByIndex(threadIndex);
    } else {
      tick().then(() => {
        if (milkdownInstance && editorService) {
          const newThreadIndex = editor.threads.findIndex(
            (s: NoteThread) => s.id === threadId,
          );
          if (newThreadIndex !== -1) {
            editorService.jumpToThreadByIndex(newThreadIndex);
          }
        }
      });
    }
  };

  /**
   * When in Navigation Mode: Jumps to a thread based on its index.
   * When in Thread Options Mode: Opens the thread options popup.
   */
  const jumpToThreadByIndex = async (idx: number) => {
    const thread = editor.threads[idx];
    if (thread?.id) {
      if (sessionState.threadShortcutsMode === 'navigation') {
        await handleJump(thread.id);
      } else {
        sessionState.selectedThreadForOptions = thread;
        sessionState.activePopup = 'threadOptions';
      }
    }
  };

  /**
   * Shortcut handler: Focus the last line of the editor
   */
  const handleFocusLastLine = (): boolean => {
    if (sessionState.activePopup !== null) return false;
    if (milkdownInstance && editorService) {
      editorService.focusEnd();
      return true;
    }
    return false;
  };

  /**
   * Shortcut handler: Jump to thread by number shortcut
   */
  const handleJumpByNumber = (e: KeyboardEvent): boolean => {
    if (sessionState.activePopup !== null) return false;

    const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
    if (idx !== -1 && idx < editor.threads.length) {
      jumpToThreadByIndex(idx);
      return true;
    }

    return false;
  };

  /**
   * Shortcut handler: Navigate to yesterday's note
   */
  const handleNavigateYesterday = async (e: Event) => {
    if (sessionState.activePopup !== null) {
      e.preventDefault();
      return;
    }

    await navigateToOffset(-1);
  };

  /**
   * Shortcut handler: Navigate to last available note
   */
  const handleNavigateLastAvailable = async (e: Event) => {
    if (sessionState.activePopup !== null) {
      e.preventDefault();
      return;
    }

    await navigateToLastAvailable();
  };

  /**
   * Shortcut handler: Navigate to today's note
   */
  const handleNavigateToday = async (e: Event) => {
    if (sessionState.activePopup !== null) {
      e.preventDefault();
      return;
    }
    await navigateToOffset(0);
  };

  useShortcuts({
    focusLastLine: handleFocusLastLine,
    jumpByNumber: handleJumpByNumber,
    navigateYesterday: handleNavigateYesterday,
    navigateLastAvailable: handleNavigateLastAvailable,
    navigateToday: handleNavigateToday,
  });

  const customKeymap = prosePlugin(() =>
    keymap({
      'Mod-1': () => true,
      'Mod-2': () => true,
    }),
  );

  /**
   * Connect the store's sync back to the component's bindable props
   */
  $effect(() => {
    editor.onContentUpdate = (updated: NoteContentResponse) =>
      (noteContent = updated);
  });

  /**
   * Expose jump functionality to parent components
   */
  $effect(() => {
    editor.jumpToThread = (threadId: string) => {
      const instance = milkdownInstance;
      if (instance && editorService) {
        const threadIndex = editor.threads.findIndex(
          (nt: NoteThread) => nt.id === threadId,
        );
        if (threadIndex !== -1) editorService.jumpToThreadByIndex(threadIndex);
      }
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
