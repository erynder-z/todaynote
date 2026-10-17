<script lang="ts">
  /**
   * Orchestrates the reusable selection floating anchor with the note formatter.
   * Keeps backward compatibility for NoteEditor.svelte imports.
   */
  import type { Editor } from '@milkdown/kit/core';
  import type { SelectionFloatingAnchorInstance } from '#lib/types/other.js';
  import NoteFormatter from './NoteFormatter.svelte';
  import SelectionFloatingAnchor from './SelectionFloatingAnchor.svelte';

  interface Props {
    editorInstance: Editor | null;
  }

  let { editorInstance }: Props = $props();

  let linkInputActive = $state(false);
  let anchor: SelectionFloatingAnchorInstance | null = $state(null);

  const handleLinkInputActive = (active: boolean) => {
    linkInputActive = active;
  };
</script>

<SelectionFloatingAnchor bind:this={anchor} {linkInputActive} {editorInstance}>
  {#snippet children(visible: boolean)}
    <NoteFormatter
      {editorInstance}
      onLinkInputActive={handleLinkInputActive}
      {anchor}
      {visible}
    />
  {/snippet}
</SelectionFloatingAnchor>
