<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { user, authLoading, login } from "$lib/auth";
  import { setDocumentTitle } from "$lib/documentTitle";
  import { authApi } from "$lib/api";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";

  setDocumentTitle("Вход");

  let email = $state("");
  let password = $state("");
  let error = $state("");
  let isLoading = $state(false);
  let registrationOpen = $state(false);

  authApi
    .registrationStatus()
    .then((s) => (registrationOpen = s.open))
    .catch(() => {});

  // Ошибка из query (редирект после неудачного входа через OIDC)
  $effect(() => {
    const errorFromUrl = page.url.searchParams.get("error");
    if (errorFromUrl) {
      error = decodeURIComponent(errorFromUrl);
      // Очистить query без перезагрузки
      const url = new URL(page.url);
      url.searchParams.delete("error");
      window.history.replaceState({}, "", url.pathname + url.search);
    }
  });

  // Публичная страница: если уже авторизован — редирект на /projects
  $effect(() => {
    if (page.url.pathname !== "/login") return;
    if ($authLoading) return;
    if ($user) goto("/projects", { replaceState: true });
  });

  function handleOidcLogin() {
    window.location.href = "/oauth/oidc/login?next=/projects";
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = "";
    isLoading = true;
    try {
      await login(email, password);
      goto("/projects");
    } catch (err) {
      error = err instanceof Error ? err.message : "Ошибка входа";
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Вход — CFGY</title>
</svelte:head>

{#if $authLoading}
  <div class="flex items-center justify-center min-h-screen">Загрузка...</div>
{:else if $user}
  <div class="flex items-center justify-center min-h-screen">
    Перенаправление...
  </div>
{:else}
  <div class="auth-page">
    <div class="auth-container">
      <div class="auth-logo-wrapper">
        <img src="/logo.svg" alt="CFGY" class="auth-logo" />
      </div>

      <div class="auth-form-wrapper">
        <div class="auth-title">Войдите в аккаунт:</div>

        <form onsubmit={handleSubmit} class="auth-form">
          <div class="auth-inputs">
            <Input
              type="email"
              placeholder="Email"
              bind:value={email}
              required
              disabled={isLoading}
              state={error ? "error" : "default"}
              description={error || undefined}
              className="w-full"
            />

            <div class="auth-password-wrapper">
              <Input
                type="password"
                placeholder="Пароль"
                bind:value={password}
                required
                disabled={isLoading}
                minlength={8}
                showPasswordToggle
                state={error ? "error" : "default"}
                className="w-full"
              />
            </div>
          </div>

          <Button
            type="submit"
            variant="basic-accent"
            size="big-text"
            disabled={isLoading}
          >
            {isLoading ? "Вход..." : "Войти"}
          </Button>

          <Button
            type="button"
            variant="basic-secondary"
            size="big-text"
            disabled={isLoading}
            onclick={handleOidcLogin}
          >
            Войти через SSO
          </Button>
        </form>
      </div>

      {#if registrationOpen}
      <div class="auth-signup-wrapper">
        <div class="auth-signup-text">Ещё нет аккаунта?</div>
        <div class="auth-signup-button-wrapper">
          <div class="auth-signup-button-text">Не проблема —</div>
          <a href="/register" class="auth-signup-button-link">
            <Button
              variant="basic-secondary"
              size="small-text"
              className="w-full"
            >
              Зарегистрироваться
            </Button>
          </a>
        </div>
      </div>
      {/if}
    </div>
  </div>
{/if}
