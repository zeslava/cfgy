// Первый шаг смены пароля - ввод email
// Страница восстановления пароля - шаг 1
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import logoImage from "@/assets/logo.svg";
import "@/styles/pages/forgot-password.css";

const ForgotPasswordStep1: React.FC = () => {
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      // Здесь будет вызов API для отправки кода на email
      console.log("Отправка кода на email:", email);

      // Переход к следующему шагу
      navigate("/forgot-password/step2", { state: { email } });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="forgot-password-page">
      <div className="forgot-password-container">
        <div className="forgot-password-logo-wrapper">
          <img src={logoImage} alt="CFGY" className="forgot-password-logo" />
        </div>
        <div className="forgot-password-form-wrapper">
          <div className="forgot-password-title">
            Введите почту, на которую зарегистрирован ваш аккаунт
          </div>
          <form onSubmit={handleSubmit} className="forgot-password-form">
            <div className="forgot-password-inputs">
              <Input
                type="email"
                placeholder="Email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                disabled={isLoading}
                state={error ? "error" : "default"}
                description={error || undefined}
                className="w-full"
              />
            </div>
            <Button
              type="submit"
              variant="basic-secondary"
              size="big-text"
              disabled={isLoading}
            >
              {isLoading ? "Отправка..." : "Отправить код"}
            </Button>
          </form>
          {error && (
            <div className="text-states-error text-regular-16 text-center w-full">
              {error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ForgotPasswordStep1;
