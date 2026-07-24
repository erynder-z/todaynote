<script lang="ts">
  /**
   * Orchestrates the reusable selection floating anchor with the note formatter.
   * Keeps backward compatibility for NoteEditor.svelte imports.
   */
  import type { Editor } from '@milkdown/core';
  import NoteFormatter from './NoteFormatter.svelte';
  import SelectionFloatingAnchor from './SelectionFloatingAnchor.svelte';

  let { editorInstance } = $props<{ editorInstance: Editor | null }>();

  // Track if link input is active to prevent toolbar from hiding
  let linkInputActive = $state(false);

  const handleLinkInputActive = (active: boolean) => {
    linkInputActive = active;
  };
</script>

<SelectionFloatingAnchor {linkInputActive}>
  {#snippet children()}
    <NoteFormatter {editorInstance} onLinkInputActive={handleLinkInputActive} />
  {/snippet}
</SelectionFloatingAnchor>
