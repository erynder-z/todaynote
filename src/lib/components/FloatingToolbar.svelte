<script lang="ts">
  /**
   * Orchestrates the reusable selection floating anchor with the note formatter.
   * Keeps backward compatibility for NoteEditor.svelte imports.
   */
  import type { Editor } from '@milkdown/core';
  import NoteFormatter from './NoteFormatter.svelte';
  import SelectionFloatingAnchor from './SelectionFloatingAnchor.svelte';

  let { editorInstance } = $props<{ editorInstance: Editor | null }>();
</script>

<SelectionFloatingAnchor>
  {#snippet children()}
    <NoteFormatter {editorInstance} />
  {/snippet}
</SelectionFloatingAnchor>
