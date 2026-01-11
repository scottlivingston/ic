<script lang="ts">
  import { audioVolume } from '../stores';
  import { sendVolume } from '../api';
  import Section from './ui/Section.svelte';
  import Slider from './ui/Slider.svelte';

  let collapsed = $state(false);

  function handleVolumeChange(value: number) {
    audioVolume.set(value);
    sendVolume(value);
  }
</script>

<Section title="Audio" bind:collapsed>
  <div class="audio-controls">
    <Slider
      label="Volume"
      value={$audioVolume}
      min={0}
      max={1}
      step={0.05}
      oninput={handleVolumeChange}
    />
  </div>
</Section>

<style>
  .audio-controls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }
</style>
