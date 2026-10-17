<script lang="ts">
  /**
   * The main note editor component. Coordinates the Milkdown instance with
   * the EditorStore. Handles reactive content syncing, focus, transitions,
   * and wires up editor shortcuts and the custom keymap.
   */
  import type { Editor } from '@milkdown/kit/core';
  import { untrack } from 'svelte';
  import type { NoteContentResponse, NoteThread } from '#lib/interfaces/notes.js';
  import { customKeymapPlugin } from '../plugins/customKeymapPlugin';
  import { linkOpenerPlugin } from '../plugins/linkOpenerPlugin';
  import type { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { EditorService } from '../utils/editor';
  import { createEditorShortcuts } from '../utils/editorShortcuts';
  import { useShortcuts } from '../utils/shortcuts';
  import FloatingToolbar from './FloatingToolbar.svelte';
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

  let lastNotePath = $state<string | null>(null);
  let isTransitioning = $state<boolean>(false);
  let transitionTimeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    return () => {
      if (transitionTimeout) clearTimeout(transitionTimeout);
    };
  });

  const plugins = $derived([customKeymapPlugin, linkOpenerPlugin]);

  // Build shortcut handlers once; getters read the reactive instance/service at event time.
  const shortcuts = createEditorShortcuts({
    editor: untrack(() => editor),
    getInstance: () => milkdownInstance,
    getService: () => editorService,
  });

  useShortcuts(shortcuts.actions);

  /** Create the editor service whenever the Milkdown instance becomes ready. */
  $effect(() => {
    editorService = milkdownInstance
      ? new EditorService(milkdownInstance)
      : null;
  });

  /** Sync props to the internal store before rendering. */
  $effect.pre(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * Apply pending external content updates to the editor.
   * On a note path change (when there was a previous note) we delay the
   * update to match the CSS fade transition; otherwise we apply immediately.
   */
  $effect(() => {
    const instance = milkdownInstance;
    const service = editorService;
    const hasPendingUpdate = editor.pendingExternalUpdate;
    const pendingJump = sessionState.pendingThreadJump;
    const currentPath = notePath;

    if (!instance || !service || !hasPendingUpdate) return;

    editor.pendingExternalUpdate = false;

    const isPathChange = currentPath !== lastNotePath;

    const performUpdate = () => {
      if (isPathChange) {
        const editorElement = document.querySelector('.editor-main');
        if (editorElement) editorElement.scrollTop = 0;
      }

      const onComplete = () => {
        if (isPathChange) isTransitioning = false;
      };

      if (pendingJump) {
        sessionState.pendingThreadJump = null;
        service.updateContent(editor.content, () => {
          editor.jumpToThread(pendingJump);
          onComplete();
        });
      } else {
        service.updateContent(editor.content, onComplete);
      }
      lastNotePath = currentPath;
    };

    if (isPathChange && lastNotePath !== null) {
      isTransitioning = true;

      if (transitionTimeout) clearTimeout(transitionTimeout);
      transitionTimeout = setTimeout(
        performUpdate,
        sessionState.isLinux ? 0 : 150,
      );
    } else {
      performUpdate();
    }
  });

  /**
   * Auto-focus the editor when no popup is active and we have a note path.
   * Blur when a popup opens so keystrokes don't reach ProseMirror behind the modal overlay.
   */
  $effect(() => {
    const instance = milkdownInstance;
    const service = editorService;
    if (!instance || !service) return;

    if (sessionState.activePopup === null && notePath) service.focus();
    else if (sessionState.activePopup !== null) service.blur();
  });

  /** Sync the store's content updates back to the bindable prop. */
  $effect(() => {
    editor.onContentUpdate = (updated: NoteContentResponse) =>
      (noteContent = updated);
  });

  /** Expose jump functionality to parent components. */
  $effect(() => {
    editor.jumpToThread = shortcuts.jumpToThread;
  });
</script>

{#if notePath}
  <div
    class="editor-transition-container"
    class:transitioning={isTransitioning}
    class:no-transition={sessionState.isLinux}
  >
    <MilkdownEditor
      content={editor.content}
      onReady={(inst) => {
        milkdownInstance = inst;
      }}
      onUpdate={(markdown: string) => editor.updateContent(markdown)}
      {plugins}
      activeThreadIndex={sessionState.activePopup === 'threadOptions'
        ? editor.threads.findIndex(
            (t: NoteThread) =>
              t.id === sessionState.selectedThreadForOptions?.id,
          )
        : -1}
    />
    {#if settings.floatingToolbarEnabled}
      <FloatingToolbar editorInstance={milkdownInstance} />
    {/if}
  </div>
{/if}

<style>
  .editor-transition-container {
    opacity: 1;
    transition: opacity 150ms cubic-bezier(0.4, 0, 0.2, 1);
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .editor-transition-container.transitioning {
    opacity: 0;
    overflow: hidden;
  }

  /* Transitions cause flickering on Linux; apply changes instantly instead. */
  .editor-transition-container.no-transition {
    transition: none;
  }
</style>
