// Страница регистрации
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/useAuth";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import logoImage from "@/assets/logo.svg";
import "@/styles/pages/register.css";

const Register: React.FC = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { register } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (password !== confirmPassword) {
      setError("Пароли не совпадают");
      return;
    }

    if (password.length < 8) {
      setError("Пароль должен быть не менее 8 символов");
      return;
    }

    setIsLoading(true);

    try {
      await register(email, password);
      navigate("/projects");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка регистрации");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="register-page">
      <div className="register-container">
        {/* Logo */}
        <div className="register-logo-wrapper">
          <img src={logoImage} alt="CFGY" className="register-logo" />
        </div>

        <div className="register-form-wrapper">
          {/* Title */}
          <div className="register-title">Регистрация:</div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="register-form">
            {/* Fields */}
            <div className="register-inputs">
              <Input
                type="email"
                placeholder="Email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                disabled={isLoading}
                state={error ? "error" : "default"}
                className="w-full"
              />
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
              <Input
                type="password"
                placeholder="Повторите пароль"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                required
                disabled={isLoading}
                minLength={8}
                showPasswordToggle
                state={error ? "error" : "default"}
                className="w-full"
              />
            </div>

            {/* Submit button */}
            <Button
              type="submit"
              variant="basic-secondary"
              size="big-text"
              disabled={isLoading}
            >
              {isLoading ? "Регистрация..." : "Зарегистрироваться"}
            </Button>
          </form>

          {/* Error message */}
          {error && (
            <p className="text-states-error text-regular-16 text-center w-full">
              {error}
            </p>
          )}
        </div>
      </div>
    </div>
  );
};

export default Register;
