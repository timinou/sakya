<script lang="ts">
  import '../app.css';
  import { uiState } from '$lib/stores';

  let { children } = $props();

  $effect(() => {
    document.documentElement.setAttribute('data-theme', uiState.effectiveTheme);
  });
</script>

{@render children()}
