<script lang="ts">
  /**
   * Component for displaying a note's formatted date.
   * Extracts and formats the localized date from the provided note content.
   */
  import type { NoteContentResponse } from '$lib/interfaces/notes';
  import { locale } from '$lib/utils/i18n';
  import { formatNoteName } from '$lib/utils/notes';

  let { noteContent } = $props<{
    noteContent: NoteContentResponse | null;
  }>();

  let date = $derived(
    noteContent?.path
      ? formatNoteName(noteContent.path.split(/[/\\]/).pop() || '', $locale)
      : '',
  );
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
