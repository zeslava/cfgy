// Страница входа
import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

const Login: React.FC = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    try {
      await login(email, password);
      navigate('/projects');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка входа');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen p-4 bg-gray-50">
      <div className="bg-white rounded-lg p-8 shadow-md w-full max-w-md">
        <h1 className="mb-6 text-center text-gray-800 text-2xl font-bold">Вход в CFGY</h1>
        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label htmlFor="email" className="block mb-2 font-medium text-gray-800">
              Email
            </label>
            <input
              id="email"
              type="email"
              className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              disabled={isLoading}
            />
          </div>
          <div className="mb-4">
            <label htmlFor="password" className="block mb-2 font-medium text-gray-800">
              Пароль
            </label>
            <input
              id="password"
              type="password"
              className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              disabled={isLoading}
              minLength={8}
            />
          </div>
          {error && <div className="text-red-600 text-sm mt-1">{error}</div>}
          <button
            type="submit"
            className="w-full mt-4 bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={isLoading}
          >
            {isLoading ? 'Вход...' : 'Войти'}
          </button>
        </form>
        <p className="mt-4 text-center">
          Нет аккаунта? <Link to="/register" className="text-blue-600 no-underline hover:underline">Зарегистрироваться</Link>
        </p>
      </div>
    </div>
  );
};

export default Login;

