<script lang="ts">
  let { title = '', size = 32 } = $props();

  /**
   * High-distribution, non-cryptographic string hashing (djb2).
   */
  const hashString = (str: string): number => {
    let hash = 5381;
    const cleanStr = str.trim() || 'default-thread-token';
    for (let i = 0; i < cleanStr.length; i++)
      hash = (hash << 5) + hash + cleanStr.charCodeAt(i);

    return Math.abs(hash);
  };

  // Reactive derivations driven by the input thread title
  const hash = $derived(hashString(title));

  const shapeCount = 4;
  const center = 50;
  const diameter = 100;

  // Simple pseudo-random number generator based on hash
  const pseudoRandom = (): number => {
    const value = Math.sin(hash * 12.9898) * 43758.5453;
    return value - Math.floor(value);
  };

  // Use theme-specific identicon color palette
  const IDENTICON_COLOR_VARIABLES = [
    '--identicon-1',
    '--identicon-2',
    '--identicon-3',
    '--identicon-4',
    '--identicon-5',
    '--identicon-6',
    '--identicon-7',
    '--identicon-8',
    '--identicon-9',
    '--identicon-10',
  ];

  // Generate color from theme-specific identicon palette
  const generateColor = (index: number): string => {
    const colorHash = hash + index * 997;
    const colorIndex =
      ((colorHash % IDENTICON_COLOR_VARIABLES.length) +
        IDENTICON_COLOR_VARIABLES.length) %
      IDENTICON_COLOR_VARIABLES.length;
    return `var(${IDENTICON_COLOR_VARIABLES[colorIndex]})`;
  };

  // Generate shapes in jazzicon style
  const generateShapes = () => {
    const shapes = [];
    const remainingColors = [];

    // Generate 4 distinct vibrant colors
    for (let i = 0; i < shapeCount; i++) remainingColors.push(generateColor(i));

    // Generate shapes (rectangles with random transformations)
    for (let i = 0; i < shapeCount; i++) {
      const firstRot = pseudoRandom();
      const angle = Math.PI * 2 * firstRot;
      const velocity =
        (diameter / shapeCount) * pseudoRandom() + (i * diameter) / shapeCount;
      const tx = Math.cos(angle) * velocity;
      const ty = Math.sin(angle) * velocity;
      const secondRot = pseudoRandom();
      const rot = firstRot * 360 + secondRot * 180;
      const transform = `translate(${tx} ${ty}) rotate(${rot.toFixed(1)} ${center} ${center})`;
      const fill = remainingColors[i % remainingColors.length];

      shapes.push({
        transform,
        fill,
      });
    }

    return shapes;
  };

  const shapes = $derived(generateShapes());
</script>

<svg width={size} height={size} viewBox="0 0 100 100" class="thread-identicon">
  <!-- Circular background (like jazzicon) -->

  <circle cx="50" cy="50" r="50" fill="var(--local-bg, #202020)" />

  <!-- Jazzicon-style (https://github.com/danfinlay/jazzicon) shapes - overlapping colored rectangles that fill the entire space -->

  <g clip-path="url(#circle-clip)">
    {#each shapes as shape}
      <rect
        x="-25"
        y="-25"
        width="150"
        height="150"
        transform={shape.transform}
        fill={shape.fill}
      />
    {/each}
  </g>

  <!-- Clip to circle -->
  <defs>
    <clipPath id="circle-clip">
      <circle cx="50" cy="50" r="50" />
    </clipPath>
  </defs>
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
