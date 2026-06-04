<script lang="ts">
  import { settings } from '$lib/stores/settings.svelte';
  import IdentIconDotmatrix from './IdentIcon_dotmatrix.svelte';
  import IdentIconRound from './IdentIcon_round.svelte';

  export let title = '';
  export let size = 32;
  export let style: 'dotmatrix' | 'round' | 'none' = 'dotmatrix';

  const effectiveStyle =
    style === 'dotmatrix' && settings.identiconStyle !== 'dotmatrix'
      ? settings.identiconStyle
      : style;
</script>

{#if effectiveStyle === 'dotmatrix'}
  <IdentIconDotmatrix {title} {size} />
{:else if effectiveStyle === 'round'}
  <IdentIconRound {title} {size} />
{:else if effectiveStyle === 'none'}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    height={size}
    viewBox="0 -960 960 960"
    width={size}
    fill="currentColor"
    ><path
      d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600Z"
    /></svg
  >
{/if}
