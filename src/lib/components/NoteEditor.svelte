<script lang="ts">
  /**
   * The main note editor component. Displays a note's content in an editable form.
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
  import { gfm } from '@milkdown/preset-gfm';
  import { keymap } from '@milkdown/prose/keymap';
  import { Selection } from '@milkdown/prose/state';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { untrack } from 'svelte';
  import type { NoteContentResponse, NoteSection } from '$lib/types/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { jumpToSectionInEditor } from '../utils/editor';
  import { useShortcuts } from '../utils/shortcuts';

  // --- Props & State ---

  let {
    noteContent = $bindable(),
    notePath,
    editor,
  } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
    editor: EditorStore;
  }>();

  let editorContainer: HTMLDivElement | null = $state(null);
  let milkdownInstance: Editor | null = $state(null);

  // --- Lifecycle & Reactive Effects ---

  /**
   * 1. Sync props to the internal store before rendering
   */
  $effect.pre(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * 2. Initialize the Milkdown editor instance
   */
  $effect(() => {
    if (!editorContainer || !notePath) return;

    let isDestroyed = false;
    createEditor(editorContainer).then((instance) => {
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

  /**
   * 3. Coordinate reactive updates (content sync and focus)
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
   * Creates the Milkdown editor instance with required plugins and config.
   */
  const createEditor = (container: HTMLDivElement) => {
    return Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, container);
        ctx.set(
          defaultValueCtx,
          untrack(() => editor.content),
        );
        ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
          editor.updateContent(markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .use(customKeymap)
      .create();
  };

  /**
   * Updates the editor's content from a Markdown string and positions the cursor.
   */
  const updateEditorContent = (instance: Editor, markdown: string) => {
    instance.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const parser = ctx.get(parserCtx);
      const doc = parser(markdown);
      if (!doc) return;

      let tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc);

      // Ensure trailing empty line for headings (Milkdown parser workaround)
      if (doc.lastChild?.type.name === 'heading') {
        const paragraph = view.state.schema.nodes.paragraph.create();
        tr = tr.insert(tr.doc.content.size, paragraph);
      }

      // Position cursor at end and focus
      const selection = Selection.atEnd(tr.doc);
      view.dispatch(tr.setSelection(selection).scrollIntoView());
      view.focus();
    });
  };

  /**
   * Focuses the editor instance.
   */
  const focusEditor = (instance: Editor) => {
    instance.action((ctx) => ctx.get(editorViewCtx).focus());
  };

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
      if (sessionState.activePopup !== null) return false;
      if (milkdownInstance) focusEditor(milkdownInstance);
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup !== null) return false;

      const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
      if (idx !== -1 && idx < editor.sections.length) {
        jumpToSectionByIndex(idx);
      }
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

<div bind:this={editorContainer} class="milkdown-editor-wrapper"></div>

<style>
  .milkdown-editor-wrapper :global(.milkdown) {
    width: 100%;
    min-height: 70dvh;
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

  .milkdown-editor-wrapper :global(.milkdown pre) {
    background: var(--code-bg, #f5f5f5);
    border-radius: 0.5rem;
    padding: 1rem;
    overflow-x: auto;
    margin: 1rem 0;
    font-family: var(
      --font-mono,
      'SFMono-Regular',
      Consolas,
      'Liberation Mono',
      Menlo,
      monospace
    );
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .milkdown-editor-wrapper :global(.milkdown blockquote) {
    border-left: 3px solid var(--quote-border, #e2e8f0);
    padding-left: 1rem;
    margin: 1rem 0;
    color: var(--quote-color, inherit);
    font-style: italic;
  }

  .milkdown-editor-wrapper :global(.milkdown code) {
    background: var(--code-bg, #f5f5f5);
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    font-family: var(
      --font-mono,
      'SFMono-Regular',
      Consolas,
      'Liberation Mono',
      Menlo,
      monospace
    );
    font-size: 0.875rem;
  }
</style>
