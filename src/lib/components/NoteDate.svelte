<script lang="ts">
  /**
   * Component for displaying a note's formatted date.
   * Extracts the localized date from the provided note content.
   */
  import type { NoteContentResponse } from '$lib/interfaces/notes';

  let { noteContent } = $props<{
    noteContent: NoteContentResponse | null;
  }>();

  let date = $derived(noteContent?.metadata.formattedDate);
</script>

{#if date}
  <div class="date">{date}</div>
{/if}

<style>
  .date {
    font-size: 1.1rem;
    color: var(--text-main);
    font-weight: 700;
    letter-spacing: -0.01em;
    text-align: center;
    width: 100%;
  }
</style>
