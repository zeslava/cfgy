// Страница входа - реализована по дизайну Figma с использованием токенов
import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "../contexts/useAuth";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import logoImage from "@/assets/logo.svg";
import "@/styles/pages/login.css";

const Login: React.FC = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const handleZidLogin = () => {
    // Инициируем SSO через server -> ZID -> callback (web) -> server sets httpOnly cookie
    window.location.href = "/auth/zid/login?next=/projects";
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      await login(email, password);
      navigate("/projects");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка входа");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-container">
        <div className="login-logo-wrapper">
          <img src={logoImage} alt="CFGY" className="login-logo" />
        </div>

        <div className="login-form-wrapper">
          <div className="login-title">Войдите в аккаунт:</div>

          <form onSubmit={handleSubmit} className="login-form">
            <div className="login-inputs">
              <Input
                type="email"
                placeholder="Email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                disabled={isLoading}
                state={error ? "error" : "default"}
                description={error ? error : undefined}
                className="w-full"
              />

              <div className="login-password-wrapper">
                <Input
                  type="password"
                  placeholder="Пароль"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  disabled={isLoading}
                  minLength={8}
                  showPasswordToggle
                  state={error ? "error" : "default"}
                  className="w-full"
                />
                <Link to="/forgot-password/step1" className="login-forgot-link">
                  Забыл пароль?
                </Link>
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
              onClick={handleZidLogin}
            >
              Войти через ZID
            </Button>
          </form>
        </div>

        <div className="login-signup-wrapper">
          <div className="login-signup-text">Ещё нет аккаунта?</div>
          <div className="login-signup-button-wrapper">
            <div className="login-signup-button-text">Не проблема -</div>
            <Link to="/register" className="login-signup-button-link">
              <Button
                variant="basic-secondary"
                size="small-text"
                className="w-full"
              >
                Зарегистрироваться
              </Button>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Login;
