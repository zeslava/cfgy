<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { user, authLoading, login } from "$lib/auth";
  import { setDocumentTitle } from "$lib/documentTitle";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";

  setDocumentTitle("Вход");

  let email = $state("");
  let password = $state("");
  let error = $state("");
  let isLoading = $state(false);

  // Ошибка из query (редирект после неудачного входа через ZID)
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

  function handleZidLogin() {
    window.location.href = "/oauth/zid/login?next=/projects";
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
  <div class="login-page">
    <div class="login-container">
      <div class="login-logo-wrapper">
        <img src="/logo.svg" alt="CFGY" class="login-logo" />
      </div>

      <div class="login-form-wrapper">
        <div class="login-title">Войдите в аккаунт:</div>

        <form onsubmit={handleSubmit} class="login-form">
          <div class="login-inputs">
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

            <div class="login-password-wrapper">
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
              <a href="/forgot-password/step1" class="login-forgot-link">
                Забыл пароль?
              </a>
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
            onclick={handleZidLogin}
          >
            Войти через ZID
          </Button>
        </form>
      </div>

      <div class="login-signup-wrapper">
        <div class="login-signup-text">Ещё нет аккаунта?</div>
        <div class="login-signup-button-wrapper">
          <div class="login-signup-button-text">Не проблема —</div>
          <a href="/register" class="login-signup-button-link">
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
    </div>
  </div>
{/if}
