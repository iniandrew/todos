<script lang="ts">
  import { Star } from "@lucide/svelte";

  interface Props {
    active: boolean;
    label: string;
    onchange: () => void;
  }
  let { active, label, onchange }: Props = $props();
</script>

<button
  class="star"
  class:active
  aria-pressed={active}
  aria-label={label}
  title={label}
  onclick={(e) => {
    e.stopPropagation();
    onchange();
  }}
>
  <Star size={18} strokeWidth={1.8} fill={active ? "currentColor" : "none"} />
</button>

<style>
  .star {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 50%;
    color: var(--text-3);
    transition:
      color 0.15s,
      transform 0.2s var(--ease);
  }
  .star:hover {
    color: var(--accent);
  }
  .star:active {
    transform: scale(0.85);
  }
  .active {
    color: var(--accent);
    animation: spin 0.45s var(--ease);
  }
  @keyframes spin {
    50% {
      transform: scale(1.25) rotate(36deg);
    }
  }
</style>
