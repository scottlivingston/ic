<script lang="ts">
  import { PRESET_PHRASES, type PresetPhrase } from '../stores';
  import { sendSpeak } from '../api';
  import Section from './ui/Section.svelte';
  import Button from './ui/Button.svelte';
  import TextInput from './ui/TextInput.svelte';

  let presetsCollapsed = $state(false);
  let customCollapsed = $state(false);
  let customMessage = $state('');

  function handlePresetClick(phrase: PresetPhrase) {
    sendSpeak(phrase.text, phrase.face);
  }

  function handleCustomSubmit() {
    const msg = customMessage.trim();
    if (msg) {
      sendSpeak(msg);
      customMessage = '';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleCustomSubmit();
    }
  }
</script>

<Section title="Preset Phrases" bind:collapsed={presetsCollapsed}>
  <div class="preset-buttons">
    {#each PRESET_PHRASES as phrase}
      <Button variant="block" onclick={() => handlePresetClick(phrase)}>
        {phrase.text}
      </Button>
    {/each}
  </div>
</Section>

<Section title="Custom Message" bind:collapsed={customCollapsed}>
  <div class="free-form">
    <TextInput
      bind:value={customMessage}
      placeholder="Enter text to speak..."
      onkeydown={handleKeydown}
    />
    <Button onclick={handleCustomSubmit}>Send</Button>
  </div>
</Section>

<style>
  .preset-buttons {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .free-form {
    display: flex;
    gap: var(--spacing-sm);
  }

  @media (max-width: 768px) {
    .free-form {
      flex-direction: column;
    }
  }
</style>
