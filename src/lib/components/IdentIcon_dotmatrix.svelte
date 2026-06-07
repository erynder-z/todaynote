<script lang="ts">
  let { title = '', size = '2rem' } = $props<{
    title?: string;
    size?: string; // relative units
  }>();

  /**
   * High-distribution, non-cryptographic string hashing (djb2).
   * Operates entirely in memory with no external dependencies.
   */
  const hashString = (str: string): number => {
    let hash = 5381;

    const cleanStr = str.trim() || 'default-thread-token';

    for (let i = 0; i < cleanStr.length; i++)
      hash = (hash << 5) + hash + cleanStr.charCodeAt(i);

    return Math.abs(hash);
  };

  const hash = $derived(hashString(title));
  const gridSize = 5;
  const cellSize = 100 / gridSize;
  const cellPadding = cellSize * 0.2;
  const dotSize = cellSize - cellPadding;

  // Generate dot-matrix pattern
  const generateDotMatrix = () => {
    const dots = [];

    const color = generateColor(0);

    for (let row = 0; row < gridSize; row++) {
      for (let col = 0; col < gridSize; col++) {
        // Use hash bits to determine if dot should be visible

        const bitPosition = row * gridSize + col;
        const shouldShow = (hash & (1 << bitPosition)) !== 0;

        if (shouldShow) {
          const x = col * cellSize + cellSize / 2;
          const y = row * cellSize + cellSize / 2;

          dots.push({
            cx: x,
            cy: y,
            fill: color,
          });
        }
      }
    }

    return dots;
  };

  const THEME_COLOR_VARIABLES = [
    '--accent',
    '--success',
    '--error',
    '--md-h1',
    '--md-h2',
    '--md-h3',
    '--md-h4',
    '--md-bold',
    '--md-italic',
    '--md-list-marker',
    '--md-link',
    '--md-link-hover',
    '--md-quote-border',
    '--text-selection',
    '--accent',
    '--success',
    '--error',
    '--md-h1',
    '--md-h2',
    '--md-h3',
  ];

  // Generate color from theme variables
  const generateColor = (index: number): string => {
    const colorHash = hash + index * 997;
    const colorIndex =
      ((colorHash % THEME_COLOR_VARIABLES.length) +
        THEME_COLOR_VARIABLES.length) %
      THEME_COLOR_VARIABLES.length;
    return `var(${THEME_COLOR_VARIABLES[colorIndex]})`;
  };

  const dots = $derived(generateDotMatrix());
</script>

<svg width={size} height={size} viewBox="0 0 100 100" class="thread-identicon">
  <!-- Dot-matrix background -->
  <rect x="0" y="0" width="100" height="100" fill="transparent" />
  <!-- Dot-matrix pattern -->
  {#each dots as dot}
    <circle cx={dot.cx} cy={dot.cy} r={dotSize / 2} fill={dot.fill} />
  {/each}
</svg>

<style>
  .thread-identicon {
    display: inline-block;
    vertical-align: middle;
    user-select: none;
    transform: translateZ(0);
    overflow: visible;
  }
</style>
