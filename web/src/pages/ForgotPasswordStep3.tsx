// Третий шаг смены пароля - создание нового пароля
import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { Input } from '../components/ui/Input';
import { Button } from '../components/ui/Button';
import logoImage from '@/assets/logo.svg';
import '@/styles/pages/forgot-password.css';

const ForgotPasswordStep3: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [email] = useState((location.state as { email?: string })?.email || '');
  const [code] = useState((location.state as { code?: string })?.code || '');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!email || !code) {
      navigate('/forgot-password/step1');
    }
  }, [email, code, navigate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (newPassword !== confirmPassword) {
      setError('Пароли не совпадают');
      return;
    }

    if (newPassword.length < 8) {
      setError('Пароль должен быть не менее 8 символов');
      return;
    }

    setIsLoading(true);

    try {
      // TODO: Реализовать API вызов для смены пароля
      // await authApi.resetPassword(email, code, newPassword);
      // После успешной смены пароля переходим на страницу входа
      navigate('/login', { state: { message: 'Пароль успешно изменен' } });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка смены пароля');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="forgot-password-page">
      <div className="forgot-password-container">
        <div className="forgot-password-logo-wrapper">
          <img
            src={logoImage}
            alt="CFGY"
            className="forgot-password-logo"
          />
        </div>
        <div className="forgot-password-form-wrapper">
          <div className="forgot-password-title">
            Создайте новый пароль:
          </div>
          <form onSubmit={handleSubmit} className="forgot-password-form">
            <div className="forgot-password-password-fields">
              <Input
                type="password"
                placeholder="Новый пароль"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
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
              {isLoading ? 'Смена пароля...' : 'Сменить пароль'}
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
};

export default ForgotPasswordStep3;

