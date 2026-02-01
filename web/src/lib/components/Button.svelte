<script lang="ts">
  export type ButtonVariant =
    | "primary"
    | "secondary"
    | "danger"
    | "basic-secondary"
    | "basic-accent";
  export type ButtonSize = "big-text" | "small-text" | "small-icon";

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    fullWidth?: boolean;
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    className?: string;
    children?: import("svelte").Snippet;
    icon?: import("svelte").Snippet;
    [attr: string]: unknown;
  }

  let {
    variant = "primary",
    size = "big-text",
    fullWidth = false,
    disabled = false,
    type = "button",
    className = "",
    children,
    icon,
    ...rest
  }: Props = $props();

  const variantClass = $derived(
    disabled
      ? variant === "primary" || variant === "basic-accent"
        ? "button-primary-disabled"
        : variant === "danger"
          ? "button-danger-default"
          : "button-secondary-disabled"
      : variant === "primary" || variant === "basic-accent"
        ? "button-primary-default"
        : variant === "danger"
          ? "button-danger-default"
          : "button-secondary-default"
  );
  const sizeClass = $derived(
    size === "small-icon"
      ? "button-small-icon"
      : size === "small-text"
        ? "button-small-text"
        : "button-big-text"
  );
</script>

<button
  {type}
  class="button {variantClass} {sizeClass} {fullWidth
    ? 'button-full-width'
    : ''} {className}"
  {disabled}
  {...rest}
>
  {#if size === "small-icon"}
    {#if icon}
      {@render icon()}
    {:else if children}
      {@render children()}
    {/if}
  {:else}
    {#if icon}
      <span class="shrink-0">{@render icon()}</span>
    {/if}
    {#if children}
      {@render children()}
    {/if}
  {/if}
</button>
