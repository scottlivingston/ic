<script lang="ts">
  import { effectsState } from '../stores';
  import { sendEffects } from '../api';
  import type { EffectsState } from '../types';
  import Section from './ui/Section.svelte';
  import Button from './ui/Button.svelte';
  import Slider from './ui/Slider.svelte';

  let collapsed = $state(false);

  function toggleEffect(key: keyof EffectsState) {
    effectsState.update(state => {
      const newState = { ...state, [key]: !state[key] };
      sendEffects(newState);
      return newState;
    });
  }

  function updateSlider(key: keyof EffectsState, value: number) {
    effectsState.update(state => {
      const newState = { ...state, [key]: value };
      sendEffects(newState);
      return newState;
    });
  }
</script>

<Section title="CRT Effects" bind:collapsed>
  <div class="effects-controls">
    <div class="effect-row">
      <Button active={$effectsState.scanlines} onclick={() => toggleEffect('scanlines')}>
        Scanlines
      </Button>
      <Slider
        label="Opacity"
        value={$effectsState.scanline_opacity}
        min={0}
        max={1}
        step={0.05}
        oninput={(v) => updateSlider('scanline_opacity', v)}
      />
    </div>

    <div class="effect-row">
      <Button active={$effectsState.curvature} onclick={() => toggleEffect('curvature')}>
        Curvature
      </Button>
      <Slider
        label="Amount"
        value={$effectsState.curvature_amount}
        min={0}
        max={100}
        step={5}
        showPercent={false}
        oninput={(v) => updateSlider('curvature_amount', v)}
      />
    </div>

    <div class="effect-row">
      <Button active={$effectsState.grid} onclick={() => toggleEffect('grid')}>
        Grid
      </Button>
    </div>
  </div>
</Section>

<style>
  .effects-controls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .effect-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .effect-row :global(.btn) {
    min-width: 120px;
  }

  @media (max-width: 768px) {
    .effect-row {
      flex-direction: column;
      align-items: stretch;
      gap: var(--spacing-sm);
    }

    .effect-row :global(.btn) {
      min-width: auto;
      width: 100%;
    }
  }
</style>
