<script lang="ts">
  import Sortable from 'sortablejs';
  import { presetPhrases, phrasesLoaded, phrasesEditMode, availableFaces, type PresetPhrase } from '../stores';
  import { sendSpeak, fetchPhrases, savePhrases, fetchFaces } from '../api';
  import Section from './ui/Section.svelte';
  import Button from './ui/Button.svelte';
  import TextInput from './ui/TextInput.svelte';
  import FaceSelect from './ui/FaceSelect.svelte';

  // Editable phrase with unique ID for Svelte keying
  type EditablePhrase = PresetPhrase & { _id: number };
  let nextId = 0;

  let presetsCollapsed = $state(false);
  let customCollapsed = $state(false);
  let customMessage = $state('');
  let customFace = $state('default');

  // Edit mode state
  let editingPhrases = $state<EditablePhrase[]>([]);
  let newPhraseText = $state('');
  let newPhraseFace = $state('default');
  let wasInEditMode = $state(false);

  // Reference to the phrase list for SortableJS
  let phraseListEl = $state<HTMLDivElement | null>(null);
  let sortableInstance: Sortable | null = null;

  // Load phrases and faces on mount if not already loaded
  $effect(() => {
    if (!$phrasesLoaded) {
      phrasesLoaded.set(true);
      Promise.all([fetchPhrases(), fetchFaces()]).then(([phrases, faces]) => {
        presetPhrases.set(phrases);
        availableFaces.set(faces);
      });
    }
  });

  // Handle edit mode enter/exit
  $effect(() => {
    if ($phrasesEditMode && !wasInEditMode) {
      // Entering edit mode - copy phrases with unique IDs
      editingPhrases = $presetPhrases.map(p => ({ ...p, _id: nextId++ }));
    } else if (!$phrasesEditMode && wasInEditMode) {
      // Exiting edit mode - save changes (strip _id before saving)
      const phrasesToSave: PresetPhrase[] = editingPhrases.map(({ _id, ...rest }) => rest);
      savePhrases(phrasesToSave).then((result) => {
        if (result.success) {
          presetPhrases.set(phrasesToSave);
        } else {
          console.error('Failed to save phrases:', result.error);
        }
      });
    }
    wasInEditMode = $phrasesEditMode;
  });

  // Initialize/destroy SortableJS when edit mode changes
  $effect(() => {
    if ($phrasesEditMode && phraseListEl) {
      // Small delay to ensure DOM is ready
      setTimeout(() => {
        sortableInstance = Sortable.create(phraseListEl, {
          handle: '.drag-handle',
          animation: 150,
          onEnd: (evt) => {
            if (evt.oldIndex !== undefined && evt.newIndex !== undefined) {
              const newPhrases = [...editingPhrases];
              const [moved] = newPhrases.splice(evt.oldIndex, 1);
              newPhrases.splice(evt.newIndex, 0, moved);
              editingPhrases = newPhrases;
            }
          }
        });
      }, 0);
    } else if (sortableInstance) {
      sortableInstance.destroy();
      sortableInstance = null;
    }
  });

  function handlePresetClick(phrase: PresetPhrase) {
    sendSpeak(phrase.text, phrase.face);
  }

  function handleCustomSubmit() {
    const msg = customMessage.trim();
    if (msg) {
      sendSpeak(msg, customFace !== 'default' ? customFace : undefined);
      customMessage = '';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleCustomSubmit();
    }
  }

  // Edit functions
  function addPhrase() {
    const text = newPhraseText.trim();
    if (!text) return;

    const phrase: EditablePhrase = { text, _id: nextId++ };
    if (newPhraseFace && newPhraseFace !== 'default') {
      phrase.face = newPhraseFace;
    }

    editingPhrases = [...editingPhrases, phrase];
    newPhraseText = '';
    newPhraseFace = 'default';
  }

  function removePhrase(index: number) {
    editingPhrases = editingPhrases.filter((_, i) => i !== index);
  }

  function updatePhraseText(index: number, text: string) {
    const newPhrases = [...editingPhrases];
    newPhrases[index] = { ...newPhrases[index], text };
    editingPhrases = newPhrases;
  }

  function updatePhraseFace(index: number, face: string) {
    const newPhrases = [...editingPhrases];
    if (face === 'default') {
      // Remove face property if default
      const { face: _, ...rest } = newPhrases[index];
      newPhrases[index] = rest as PresetPhrase;
    } else {
      newPhrases[index] = { ...newPhrases[index], face };
    }
    editingPhrases = newPhrases;
  }

</script>

<Section title="Preset Phrases" bind:collapsed={presetsCollapsed}>
  {#if $phrasesEditMode}
    <!-- Edit Mode -->
    <div class="edit-mode">
      <div class="phrase-list" bind:this={phraseListEl}>
        {#each editingPhrases as phrase, index (phrase._id)}
          <div class="phrase-item">
            <FaceSelect
              value={phrase.face || 'default'}
              options={$availableFaces}
              onchange={(face) => updatePhraseFace(index, face)}
            />
            <TextInput
              value={phrase.text}
              placeholder="Phrase text..."
              oninput={(text) => updatePhraseText(index, text)}
            />
            <Button variant="small" onclick={() => removePhrase(index)}>X</Button>
            <span class="drag-handle" title="Drag to reorder"></span>
          </div>
        {/each}
      </div>

      <div class="add-phrase">
        <FaceSelect
          bind:value={newPhraseFace}
          options={$availableFaces}
        />
        <TextInput bind:value={newPhraseText} placeholder="New phrase text..." />
        <Button onclick={addPhrase} disabled={!newPhraseText.trim()}>Add</Button>
      </div>
    </div>
  {:else}
    <!-- Normal Mode -->
    <div class="preset-buttons">
      {#if $presetPhrases.length === 0}
        <div class="loading">Loading phrases...</div>
      {:else}
        {#each $presetPhrases as phrase}
          <Button variant="block" onclick={() => handlePresetClick(phrase)}>
            <img
              class="face-preview"
              src="/api/faces/{phrase.face || 'default'}.png"
              alt=""
            />
            {phrase.text}
          </Button>
        {/each}
      {/if}
    </div>
  {/if}
</Section>

<Section title="Custom Message" bind:collapsed={customCollapsed}>
  <div class="free-form">
    <FaceSelect
      bind:value={customFace}
      options={$availableFaces}
    />
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

  .preset-buttons :global(.btn) {
    display: flex;
    align-items: center;
    padding-top: 0;
    padding-bottom: 0;
    line-height: 42px;
  }

  .face-preview {
    width: 40px;
    height: 30px;
    flex-shrink: 0;
    margin-right: var(--spacing-sm);
    /* Tint white face to cyan/green, keep black background */
    filter: sepia(1) saturate(5) hue-rotate(120deg);
  }

  .loading {
    opacity: 0.5;
    text-align: center;
    padding: var(--spacing-md);
  }

  .free-form {
    display: flex;
    gap: var(--spacing-sm);
  }

  .free-form :global(.face-select) {
    flex-shrink: 0;
  }

  .free-form :global(.text-input) {
    flex: 1;
  }

  /* Edit mode styles */
  .edit-mode {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .phrase-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .phrase-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    border: var(--border);
    background: var(--color-bg);
  }

  .phrase-item :global(.text-input) {
    flex: 1;
    height: 40px;
    box-sizing: border-box;
  }

  .phrase-item :global(.btn) {
    height: 40px;
    width: 40px;
    padding: 0;
    box-sizing: border-box;
  }

  .phrase-item :global(.face-select) {
    flex-shrink: 0;
  }

  .drag-handle {
    width: 40px;
    height: 40px;
    padding: var(--spacing-xs);
    cursor: grab;
    opacity: 0.5;
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .drag-handle::before {
    content: '\2630';
    font-size: 16px;
    color: var(--color-primary);
  }

  .drag-handle:hover {
    opacity: 1;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .add-phrase {
    display: flex;
    gap: var(--spacing-sm);
  }

  .add-phrase :global(.text-input) {
    flex: 1;
    height: 40px;
    box-sizing: border-box;
  }

  .add-phrase :global(.btn) {
    height: 40px;
    box-sizing: border-box;
  }

  .add-phrase :global(.face-select) {
    flex-shrink: 0;
  }

  @media (max-width: 768px) {
    .free-form {
      flex-direction: row;
      flex-wrap: wrap;
    }

    .free-form :global(.text-input) {
      flex: none;
      width: 100%;
      order: 1;
      margin-top: var(--spacing-xs);
    }

    .free-form :global(.face-select) {
      flex: 1;
    }

    .phrase-item {
      flex-wrap: wrap;
    }

    .phrase-item :global(.text-input) {
      flex: none;
      width: 100%;
      order: 1;
      margin-top: var(--spacing-xs);
    }

    .phrase-item :global(.face-select) {
      flex: 1;
    }

    .add-phrase {
      flex-wrap: wrap;
    }

    .add-phrase :global(.text-input) {
      flex: none;
      width: 100%;
      order: 1;
      margin-top: var(--spacing-xs);
    }

    .add-phrase :global(.face-select) {
      flex: 1;
    }
  }
</style>
