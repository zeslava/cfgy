<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { user, authLoading } from "$lib/auth";

  // Редирект с /: если авторизован → /projects, иначе → /login
  $effect(() => {
    if ($page.url.pathname !== "/") return;
    if ($authLoading) return;
    if ($user) {
      goto("/projects", { replaceState: true });
    } else {
      goto("/login", { replaceState: true });
    }
  });
</script>

<div class="flex items-center justify-center min-h-screen">Загрузка...</div>
