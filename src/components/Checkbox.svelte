<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    size?: number;
    onchange: () => void;
  }
  let { checked, label, size = 22, onchange }: Props = $props();
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
  <svg viewBox="0 0 22 22" aria-hidden="true">
    <circle class="ring" cx="11" cy="11" r="10" />
    <circle class="fill" cx="11" cy="11" r="10.75" />
    <path class="tick" d="M6.8 11.4 9.7 14.2 15.4 8.2" />
  </svg>
</button>

<style>
  .check {
    width: var(--size);
    height: var(--size);
    flex: none;
    border-radius: 50%;
    color: var(--accent);
    -webkit-tap-highlight-color: transparent;
  }
  svg {
    display: block;
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .ring {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 1.5;
    transition: stroke 0.2s;
  }
  .fill {
    fill: currentColor;
    transform-origin: center;
    transform: scale(0.2);
    opacity: 0;
    transition:
      transform 0.35s var(--spring),
      opacity 0.15s;
  }
  .tick {
    fill: none;
    stroke: #fff;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 14;
    stroke-dashoffset: 14;
    transition: stroke-dashoffset 0.28s var(--ease) 0.08s;
  }
  .check:hover .ring {
    stroke: currentColor;
  }
  .check:active {
    transform: scale(0.9);
  }
  .checked .ring {
    stroke: currentColor;
  }
  .checked .fill {
    transform: scale(1);
    opacity: 1;
  }
  .checked .tick {
    stroke-dashoffset: 0;
  }
</style>
