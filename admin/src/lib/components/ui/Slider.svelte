<script module lang="ts">
  // Module-level counter for unique IDs (crypto.randomUUID not available over HTTP)
  let idCounter = 0;
</script>

<script lang="ts">
  interface Props {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    showPercent?: boolean;
    oninput?: (value: number) => void;
  }

  let {
    label,
    value = $bindable(),
    min = 0,
    max = 1,
    step = 0.05,
    disabled = false,
    showPercent = true,
    oninput
  }: Props = $props();

  const id = `slider-${++idCounter}`;

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = parseFloat(target.value);
    oninput?.(value);
  }

  const displayValue = $derived(
    showPercent && max <= 1
      ? `${Math.round(value * 100)}%`
      : `${Math.round(value)}%`
  );
</script>

<div class="slider-group">
  <label class="slider-label" for={id}>{label}</label>
  <input
    {id}
    type="range"
    class="slider"
    {min}
    {max}
    {step}
    {value}
    {disabled}
    oninput={handleInput}
  />
  <span class="slider-value">{displayValue}</span>
</div>

<style>
  .slider-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex: 1;
    min-width: 150px;
  }

  .slider-label {
    font-size: var(--font-size-small);
    opacity: 0.7;
    text-transform: uppercase;
    width: 70px;
    flex-shrink: 0;
  }

  .slider {
    flex: 1;
    height: calc(var(--font-size-base) + var(--spacing-lg) + 2px);
    -webkit-appearance: none;
    appearance: none;
    background: var(--color-slider-track);
    border-radius: 0;
    outline: none;
    border: var(--border);
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: calc(var(--font-size-base) + var(--spacing-lg));
    background: var(--color-primary);
    cursor: pointer;
    border-radius: 0;
  }

  .slider::-moz-range-thumb {
    width: 16px;
    height: calc(var(--font-size-base) + var(--spacing-lg));
    background: var(--color-primary);
    cursor: pointer;
    border: none;
    border-radius: 0;
  }

  .slider-value {
    font-size: var(--font-size-small);
    min-width: 50px;
    text-align: right;
    opacity: 0.7;
  }

  @media (max-width: 768px) {
    .slider-group {
      min-width: auto;
      width: 100%;
    }
  }
</style>
