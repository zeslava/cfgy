// Второй шаг смены пароля - ввод email и кода
import { useState, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import logoImage from "@/assets/logo.svg";
import "@/styles/pages/forgot-password.css";

const ForgotPasswordStep2: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [email, setEmail] = useState(
    (location.state as { email?: string })?.email || "",
  );
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!email) {
      navigate("/forgot-password/step1");
    }
  }, [email, navigate]);

  const handleSubmitCode = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      // TODO: Реализовать API вызов для проверки кода
      // await authApi.verifyResetCode(email, code);
      // Переходим на следующий шаг
      navigate("/forgot-password/step3", { state: { email, code } });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Неверный код");
    } finally {
      setIsLoading(false);
    }
  };

  const handleResendCode = async () => {
    setError("");
    setIsLoading(true);

    try {
      // TODO: Реализовать API вызов для повторной отправки кода
      // await authApi.sendResetCode(email);
      alert("Код отправлен повторно");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка отправки кода");
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
        <div className="forgot-password-form-sections-wrapper">
          <div className="forgot-password-form-section">
            <div className="forgot-password-title">
              Введите почту, на которую зарегистрирован ваш аккаунт
            </div>
            <form onSubmit={handleResendCode} className="forgot-password-form">
              <div className="forgot-password-inputs">
                <Input
                  type="email"
                  placeholder="Email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  disabled={isLoading}
                  className="w-full"
                />
              </div>
              <Button
                type="submit"
                variant="basic-secondary"
                size="big-text"
                disabled={isLoading}
              >
                Отправить код
              </Button>
            </form>
          </div>
          <div className="forgot-password-form-section">
            <div className="forgot-password-title">Введите код из письма:</div>
            <form onSubmit={handleSubmitCode} className="forgot-password-form">
              <div className="forgot-password-inputs">
                <Input
                  type="text"
                  placeholder="Код"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
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
                Изменить пароль
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
    </div>
  );
};

export default ForgotPasswordStep2;
