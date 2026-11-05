<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    size?: number;
    onchange: () => void;
  }
  let { checked, label, size = 20, onchange }: Props = $props();
</script>

<button
  class="check"
  class:checked
  style="--size:{size}px"
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  title={label}
  onclick={(e) => {
    e.stopPropagation();
    onchange();
  }}
>
  <svg viewBox="0 0 20 20" aria-hidden="true">
    <circle class="ring" cx="10" cy="10" r="8.6" />
    <circle class="fill" cx="10" cy="10" r="9.4" />
    <path class="tick" d="M6 10.4 8.7 13 14 7.4" />
  </svg>
</button>

<style>
  .check {
    width: var(--size);
    height: var(--size);
    flex: none;
    border-radius: 50%;
    color: var(--accent);
  }
  svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .ring {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 1.5;
    transition: stroke 0.15s;
  }
  .fill {
    fill: currentColor;
    transform-origin: center;
    transform: scale(0);
    transition: transform 0.28s var(--ease);
  }
  .tick {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 14;
    stroke-dashoffset: 14;
    opacity: 0;
    transition:
      stroke-dashoffset 0.25s var(--ease),
      opacity 0.15s;
  }
  .check:hover .ring {
    stroke: currentColor;
  }
  .check:hover .tick {
    opacity: 0.7;
    stroke-dashoffset: 0;
  }
  .checked .ring {
    stroke: currentColor;
  }
  .checked .fill {
    transform: scale(1);
  }
  .checked .tick,
  .checked:hover .tick {
    stroke: #fff;
    opacity: 1;
    stroke-dashoffset: 0;
    transition-delay: 0.08s;
  }
  .checked {
    animation: pop 0.35s var(--ease);
  }
  @keyframes pop {
    40% {
      transform: scale(1.2);
    }
  }
</style>
