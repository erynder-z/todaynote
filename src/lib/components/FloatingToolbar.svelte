<script lang="ts">
  /**
   * Orchestrates the reusable selection floating anchor with the note formatter.
   * Keeps backward compatibility for NoteEditor.svelte imports.
   */
  import type { FloatingToolbarProps } from '#lib/interfaces/editor.ts';
  import type { SelectionFloatingAnchorInstance } from '#lib/types/other.ts';
  import NoteFormatter from './NoteFormatter.svelte';
  import SelectionFloatingAnchor from './SelectionFloatingAnchor.svelte';

  let { editorInstance }: FloatingToolbarProps = $props();

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
