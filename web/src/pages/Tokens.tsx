// Страница управления токенами
import { useEffect, useState } from 'react';
import { tokensApi, projectsApi } from '../services/api';
import type { Token, CreateTokenResponse, Project } from '../services/api';

const Tokens: React.FC = () => {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newToken, setNewToken] = useState<CreateTokenResponse | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    project_id: '',
    expires_in_days: '',
  });

  useEffect(() => {
    loadTokens();
    loadProjects();
  }, []);

  const loadTokens = async () => {
    setIsLoading(true);
    setError('');
    try {
      const data = await tokensApi.list();
      setTokens(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка загрузки токенов');
    } finally {
      setIsLoading(false);
    }
  };

  const loadProjects = async () => {
    try {
      const data = await projectsApi.list();
      setProjects(data);
    } catch (err) {
      console.error('Ошибка загрузки проектов:', err);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      const response = await tokensApi.create({
        name: formData.name,
        project_id: formData.project_id || undefined,
        expires_in_days: formData.expires_in_days ? parseInt(formData.expires_in_days) : undefined,
      });
      setNewToken(response);
      setShowCreateForm(false);
      setFormData({ name: '', project_id: '', expires_in_days: '' });
      await loadTokens();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка создания токена');
    }
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`Удалить токен "${name}"?`)) return;
    try {
      await tokensApi.delete(id);
      await loadTokens();
      if (newToken && newToken.token_id === id) {
        setNewToken(null);
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка удаления токена');
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      alert('Скопировано в буфер обмена');
    });
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка токенов...</div>;
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">API Токены</h1>
        <button
          className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
          onClick={() => {
            setShowCreateForm(true);
            setNewToken(null);
          }}
        >
          Создать токен
        </button>
      </div>

      {newToken && (
        <div className="bg-green-50 border-2 border-green-500 rounded-lg p-6 shadow mb-6">
          <h2 className="text-xl font-semibold mb-4">Токен создан!</h2>
          <p className="mb-2 font-bold">Важно: Сохраните этот токен сейчас. Он больше не будет показан.</p>
          <div className="flex gap-2 items-center mb-4">
            <code className="flex-1 p-2 bg-white rounded text-sm break-all">
              {newToken.token}
            </code>
            <button
              className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
              onClick={() => copyToClipboard(newToken.token)}
            >
              Копировать
            </button>
          </div>
          <button
            className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
            onClick={() => setNewToken(null)}
          >
            Закрыть
          </button>
        </div>
      )}

      {showCreateForm && (
        <div className="bg-white rounded-lg p-6 shadow mb-6">
          <h2 className="text-xl font-semibold mb-4">Создать новый токен</h2>
          <form onSubmit={handleCreate}>
            <div className="mb-4">
              <label htmlFor="token-name" className="block mb-2 font-medium text-gray-800">
                Имя токена
              </label>
              <input
                id="token-name"
                type="text"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
                placeholder="Например: Production CI/CD"
              />
            </div>
            <div className="mb-4">
              <label htmlFor="token-project" className="block mb-2 font-medium text-gray-800">
                Проект (необязательно)
              </label>
              <select
                id="token-project"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.project_id}
                onChange={(e) => setFormData({ ...formData, project_id: e.target.value })}
              >
                <option value="">Все проекты</option>
                {projects.map((project) => (
                  <option key={project.id} value={project.id}>
                    {project.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="mb-4">
              <label htmlFor="token-expires" className="block mb-2 font-medium text-gray-800">
                Срок действия (дней, необязательно)
              </label>
              <input
                id="token-expires"
                type="number"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.expires_in_days}
                onChange={(e) => setFormData({ ...formData, expires_in_days: e.target.value })}
                min="1"
                placeholder="Оставить пустым для бессрочного токена"
              />
            </div>
            {error && <div className="text-red-600 text-sm mt-1">{error}</div>}
            <div className="flex gap-2">
              <button type="submit" className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700">
                Создать
              </button>
              <button
                type="button"
                className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                onClick={() => {
                  setShowCreateForm(false);
                  setFormData({ name: '', project_id: '', expires_in_days: '' });
                }}
              >
                Отмена
              </button>
            </div>
          </form>
        </div>
      )}

      {error && !showCreateForm && <div className="text-red-600 mb-4">Ошибка: {error}</div>}

      {tokens.length === 0 ? (
        <div className="bg-white rounded-lg p-6 shadow">
          <p>Токены не найдены. Создайте первый токен для работы с API.</p>
        </div>
      ) : (
        <table className="w-full border-collapse bg-white rounded-lg overflow-hidden shadow">
          <thead>
            <tr>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Имя</th>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Проект</th>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Создан</th>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Срок действия</th>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Последнее использование</th>
              <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Действия</th>
            </tr>
          </thead>
          <tbody>
            {tokens.map((token) => (
              <tr key={token.id} className="hover:bg-gray-50">
                <td className="px-3 py-3 text-left border-b border-gray-200">{token.name}</td>
                <td className="px-3 py-3 text-left border-b border-gray-200">{token.project_id || 'Все проекты'}</td>
                <td className="px-3 py-3 text-left border-b border-gray-200">{new Date(token.created_at).toLocaleDateString('ru-RU')}</td>
                <td className="px-3 py-3 text-left border-b border-gray-200">
                  {token.expires_at
                    ? new Date(token.expires_at).toLocaleDateString('ru-RU')
                    : 'Бессрочно'}
                </td>
                <td className="px-3 py-3 text-left border-b border-gray-200">
                  {token.last_used_at
                    ? new Date(token.last_used_at).toLocaleDateString('ru-RU')
                    : 'Никогда'}
                </td>
                <td className="px-3 py-3 text-left border-b border-gray-200">
                  <button
                    className="bg-red-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-red-700"
                    onClick={() => handleDelete(token.id, token.name)}
                  >
                    Удалить
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};

export default Tokens;

