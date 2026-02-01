<script lang="ts">
  export type InputState = "default" | "focus" | "disabled" | "error";

  interface Props {
    type?: string;
    value?: string;
    state?: InputState;
    label?: string;
    description?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    className?: string;
    showPasswordToggle?: boolean;
    oninput?: (e: Event) => void;
    onchange?: (e: Event) => void;
    onfocus?: (e: FocusEvent) => void;
    onblur?: (e: FocusEvent) => void;
    [attr: string]: unknown;
  }

  let {
    type = "text",
    value = $bindable(""),
    state: inputStateProp = "default",
    label,
    description,
    placeholder,
    disabled = false,
    required = false,
    className = "",
    showPasswordToggle = false,
    oninput,
    onchange,
    onfocus,
    onblur,
    ...rest
  }: Props = $props();

  let isFocused = $state(false);
  let showPasswordVisible = $state(false);
  const inputState = $derived(inputStateProp);
  const isPassword = $derived(type === "password");
  const inputType = $derived(isPassword && showPasswordVisible ? "text" : type);
  const isDisabled = $derived(disabled || inputState === "disabled");

  const containerClass = $derived(
    inputState === "error"
      ? "input-container-error"
      : inputState === "disabled" || isDisabled
        ? "input-container-disabled"
        : isFocused
          ? "input-container-focus"
          : "input-container-default"
  );
  const fieldClass = $derived(
    isDisabled ? "input-field-disabled" : "input-field-default"
  );
  const descClass = $derived(
    inputState === "error"
      ? "input-description-error"
      : "input-description-default"
  );
  const iconClass = $derived(
    inputState === "error" ? "input-icon-error" : "input-icon-default"
  );
</script>

<div class="input-wrapper {className}">
  {#if label}
    <label class="input-label" for={rest.id}>{label}</label>
  {/if}
  <div class="input-container {containerClass}">
    <input
      id={rest.id}
      class="input-field {fieldClass}"
      {inputType}
      bind:value
      {placeholder}
      {required}
      disabled={isDisabled}
      {oninput}
      {onchange}
      onfocus={(e) => {
        isFocused = true;
        onfocus?.(e);
      }}
      onblur={(e) => {
        isFocused = false;
        onblur?.(e);
      }}
      {...rest}
    />
    {#if showPasswordToggle && isPassword}
      <button
        type="button"
        class="input-toggle-button"
        disabled={isDisabled}
        onclick={() => (showPasswordVisible = !showPasswordVisible)}
        aria-label={showPasswordVisible ? "Скрыть пароль" : "Показать пароль"}
      >
        {#if showPasswordVisible}
          <svg
            class="input-icon {iconClass}"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"
              fill="currentColor"
            />
          </svg>
        {:else}
          <svg
            class="input-icon {iconClass}"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5z"
              fill="currentColor"
            />
            <path
              d="M2 2l20 20"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        {/if}
      </button>
    {/if}
  </div>
  {#if description}
    <p class="input-description {descClass}">{description}</p>
  {/if}
</div>
