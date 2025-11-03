// Страница управления секретами
import { useEffect, useState } from 'react';
import { useParams, Link, useSearchParams } from 'react-router-dom';
import { secretsApi, environmentsApi } from '../services/api';
import type { Secrets, Environment } from '../services/api';

const SecretsPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [searchParams] = useSearchParams();
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [selectedEnv, setSelectedEnv] = useState('');
  const [secrets, setSecrets] = useState<Secrets>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showEditForm, setShowEditForm] = useState<string | null>(null);
  const [formData, setFormData] = useState({ key: '', value: '' });
  const [visibleSecrets, setVisibleSecrets] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (id) {
      loadEnvironments();
    }
  }, [id]);

  useEffect(() => {
    if (id && selectedEnv) {
      loadSecrets();
      // Сбрасываем видимость секретов при смене окружения
      setVisibleSecrets(new Set());
    }
  }, [id, selectedEnv]);

  const loadEnvironments = async () => {
    if (!id) return;
    setIsLoading(true);
    try {
      const envs = await environmentsApi.list(id);
      setEnvironments(envs);
      if (envs.length > 0 && !selectedEnv) {
        // Проверяем параметр URL для выбора окружения
        const envParam = searchParams.get('env');
        if (envParam && envs.some(e => e.name === envParam)) {
          setSelectedEnv(envParam);
        } else {
          setSelectedEnv(envs[0].name);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка загрузки окружений');
    } finally {
      setIsLoading(false);
    }
  };

  const loadSecrets = async () => {
    if (!id || !selectedEnv) return;
    setIsLoading(true);
    setError('');
    try {
      const data = await secretsApi.get(id, selectedEnv);
      setSecrets(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка загрузки секретов');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateOrUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id || !selectedEnv) return;
    setError('');
    try {
      const updatedSecrets = { ...secrets };
      updatedSecrets[formData.key] = formData.value;
      await secretsApi.set(id, selectedEnv, updatedSecrets);
      await loadSecrets();
      setShowCreateForm(false);
      setShowEditForm(null);
      setFormData({ key: '', value: '' });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка сохранения секрета');
    }
  };

  const handleDelete = async (key: string) => {
    if (!id || !selectedEnv) return;
    if (!confirm(`Удалить секрет "${key}"?`)) return;
    try {
      await secretsApi.delete(id, selectedEnv, key);
      await loadSecrets();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка удаления');
    }
  };

  const handleEdit = (key: string) => {
    setFormData({ key, value: secrets[key] || '' });
    setShowEditForm(key);
    setShowCreateForm(false);
  };

  const handleCopy = async (value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      alert('Значение скопировано в буфер обмена');
    } catch (err) {
      alert('Ошибка копирования');
    }
  };

  const toggleSecretVisibility = (key: string) => {
    setVisibleSecrets(prev => {
      const newSet = new Set(prev);
      if (newSet.has(key)) {
        newSet.delete(key);
      } else {
        newSet.add(key);
      }
      return newSet;
    });
  };

  if (isLoading && !selectedEnv) {
    return <div className="text-center py-8">Загрузка...</div>;
  }

  if (environments.length === 0) {
    return (
      <div>
        <div className="mb-6">
          <Link to={`/projects/${id}`} className="text-blue-600 no-underline hover:underline">
            ← Назад к проекту
          </Link>
        </div>
        <h1 className="text-2xl font-bold mb-4">Секреты проекта</h1>
        <div className="bg-white rounded-lg p-6 shadow">
          <p>Окружения не найдены. Создайте первое окружение для проекта.</p>
          <Link to={`/projects/${id}`} className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700 no-underline mt-4 inline-block">
            Вернуться к проекту
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="mb-6">
        <Link to={`/projects/${id}`} className="text-blue-600 no-underline hover:underline">
          ← Назад к проекту
        </Link>
      </div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Секреты проекта</h1>
        {selectedEnv && (
          <button
            className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
            onClick={() => {
              setShowCreateForm(true);
              setShowEditForm(null);
              setFormData({ key: '', value: '' });
            }}
          >
            Добавить секрет
          </button>
        )}
      </div>

      {environments.length > 0 && (
        <div className="mb-6">
          <label className="block mb-2 font-medium text-gray-800">Окружение:</label>
          <select
            className="w-auto inline-block ml-2 px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
            value={selectedEnv}
            onChange={(e) => {
              setSelectedEnv(e.target.value);
              setShowCreateForm(false);
              setShowEditForm(null);
              setVisibleSecrets(new Set());
            }}
          >
            {environments.map((env) => (
              <option key={env.id} value={env.name}>
                {env.name}
              </option>
            ))}
          </select>
        </div>
      )}

      {(showCreateForm || showEditForm) && selectedEnv && (
        <div className="bg-white rounded-lg p-6 shadow mb-6">
          <h2 className="text-xl font-semibold mb-4">{showEditForm ? 'Редактировать секрет' : 'Создать новый секрет'}</h2>
          <form onSubmit={handleCreateOrUpdate}>
            <div className="mb-4">
              <label htmlFor="secret-key" className="block mb-2 font-medium text-gray-800">
                Ключ
              </label>
              <input
                id="secret-key"
                type="text"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.key}
                onChange={(e) => setFormData({ ...formData, key: e.target.value })}
                required
                disabled={!!showEditForm}
                placeholder="Например: DATABASE_URL"
              />
            </div>
            <div className="mb-4">
              <label htmlFor="secret-value" className="block mb-2 font-medium text-gray-800">
                Значение
              </label>
              <textarea
                id="secret-value"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.value}
                onChange={(e) => setFormData({ ...formData, value: e.target.value })}
                required
                rows={3}
                placeholder="Значение секрета"
              />
            </div>
            {error && <div className="text-red-600 text-sm mt-1">{error}</div>}
            <div className="flex gap-2">
              <button type="submit" className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700">
                {showEditForm ? 'Сохранить' : 'Создать'}
              </button>
              <button
                type="button"
                className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                onClick={() => {
                  setShowCreateForm(false);
                  setShowEditForm(null);
                  setFormData({ key: '', value: '' });
                  setError('');
                }}
              >
                Отмена
              </button>
            </div>
          </form>
        </div>
      )}

      {error && !showCreateForm && !showEditForm && <div className="text-red-600 mb-4">Ошибка: {error}</div>}

      {selectedEnv && (
        <div>
          <p className="mb-4">
            Секреты для окружения: <strong>{selectedEnv}</strong>
          </p>
          <table className="w-full border-collapse bg-white rounded-lg overflow-hidden shadow">
            <thead>
              <tr>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Ключ</th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Значение</th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Действия</th>
              </tr>
            </thead>
            <tbody>
              {Object.keys(secrets).length === 0 ? (
                <tr>
                  <td colSpan={3} className="px-3 py-3 text-center border-b border-gray-200">Секреты не найдены</td>
                </tr>
              ) : (
                Object.entries(secrets).map(([key, value]) => (
                  <tr key={key} className="hover:bg-gray-50">
                    <td className="px-3 py-3 text-left border-b border-gray-200">{key}</td>
                    <td className="px-3 py-3 text-left border-b border-gray-200">
                      {showEditForm === key ? (
                        <textarea
                          className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                          value={formData.value}
                          onChange={(e) => setFormData({ ...formData, value: e.target.value })}
                          rows={2}
                        />
                      ) : visibleSecrets.has(key) ? (
                        <span className="font-mono break-all">{value}</span>
                      ) : (
                        <span className="font-mono">••••••••</span>
                      )}
                    </td>
                    <td className="px-3 py-3 text-left border-b border-gray-200">
                      <div className="flex flex-wrap gap-2">
                        {showEditForm === key ? (
                          <>
                            <button
                              className="bg-blue-600 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-blue-700"
                              onClick={handleCreateOrUpdate}
                            >
                              Сохранить
                            </button>
                            <button
                              className="bg-gray-500 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-gray-600"
                              onClick={() => {
                                setShowEditForm(null);
                                setFormData({ key: '', value: '' });
                              }}
                            >
                              Отмена
                            </button>
                          </>
                        ) : (
                          <>
                            <button
                              className="bg-blue-600 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-blue-700"
                              onClick={() => toggleSecretVisibility(key)}
                            >
                              {visibleSecrets.has(key) ? 'Скрыть' : 'Показать'}
                            </button>
                            <button
                              className="bg-gray-500 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-gray-600"
                              onClick={() => handleCopy(value)}
                            >
                              Копировать
                            </button>
                            <button
                              className="bg-gray-500 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-gray-600"
                              onClick={() => handleEdit(key)}
                            >
                              Редактировать
                            </button>
                            <button
                              className="bg-red-600 text-white px-3 py-1.5 rounded cursor-pointer text-xs font-medium transition hover:bg-red-700"
                              onClick={() => handleDelete(key)}
                            >
                              Удалить
                            </button>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default SecretsPage;
