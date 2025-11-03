// Страница деталей проекта
import { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { projectsApi, environmentsApi } from '../services/api';
import type { Project, Environment } from '../services/api';

const ProjectDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [showEnvForm, setShowEnvForm] = useState(false);
  const [envFormData, setEnvFormData] = useState({ name: '' });
  const [envError, setEnvError] = useState('');
  const [formData, setFormData] = useState({ name: '', description: '' });

  useEffect(() => {
    if (id) {
      loadProject();
      loadEnvironments();
    }
  }, [id]);

  const loadProject = async () => {
    if (!id) return;
    setIsLoading(true);
    setError('');
    try {
      const data = await projectsApi.get(id);
      setProject(data);
      setFormData({ name: data.name, description: data.description || '' });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка загрузки проекта');
    } finally {
      setIsLoading(false);
    }
  };

  const loadEnvironments = async () => {
    if (!id) return;
    try {
      const envs = await environmentsApi.list(id);
      setEnvironments(envs);
    } catch (err) {
      console.error('Ошибка загрузки окружений:', err);
    }
  };

  const handleCreateEnvironment = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    setEnvError('');
    try {
      await environmentsApi.create(id, { name: envFormData.name });
      setEnvFormData({ name: '' });
      setShowEnvForm(false);
      await loadEnvironments();
    } catch (err) {
      setEnvError(err instanceof Error ? err.message : 'Ошибка создания окружения');
    }
  };

  const handleDeleteEnvironment = async (envId: string, envName: string) => {
    if (!id) return;
    if (!confirm(`Удалить окружение "${envName}"? Это действие необратимо и удалит все секреты этого окружения.`)) return;
    try {
      await environmentsApi.delete(id, envId);
      await loadEnvironments();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка удаления окружения');
    }
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    setError('');
    try {
      const updated = await projectsApi.update(id, {
        name: formData.name,
        description: formData.description || undefined,
      });
      setProject(updated);
      setIsEditing(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ошибка обновления проекта');
    }
  };

  const handleDelete = async () => {
    if (!id || !project) return;
    if (!confirm(`Удалить проект "${project.name}"? Это действие необратимо.`)) return;
    try {
      await projectsApi.delete(id);
      navigate('/projects');
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка удаления проекта');
    }
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка...</div>;
  }

  if (error || !project) {
    return <div className="text-red-600">Ошибка: {error || 'Проект не найден'}</div>;
  }

  return (
    <div>
      <div className="mb-6">
        <Link to="/projects" className="text-blue-600 no-underline hover:underline">
          ← Назад к проектам
        </Link>
      </div>
      <div className="bg-white rounded-lg p-6 shadow mb-4">
        {isEditing ? (
          <form onSubmit={handleUpdate}>
            <h1>
              <input
                type="text"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20 text-2xl font-bold mb-2"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
            </h1>
            <div className="mb-4">
              <label className="block mb-2 font-medium text-gray-800">Описание</label>
              <textarea
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
              />
            </div>
            {error && <div className="text-red-600 text-sm mt-1">{error}</div>}
            <div className="flex gap-2 mt-4">
              <button type="submit" className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700">
                Сохранить
              </button>
              <button
                type="button"
                className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                onClick={() => {
                  setIsEditing(false);
                  setFormData({ name: project.name, description: project.description || '' });
                  setError('');
                }}
              >
                Отмена
              </button>
            </div>
          </form>
        ) : (
          <>
            <div className="flex justify-between items-start">
              <div>
                <h1 className="text-2xl font-bold">{project.name}</h1>
                {project.description && (
                  <p className="mt-2 text-gray-500">{project.description}</p>
                )}
              </div>
              <div className="flex gap-2">
                <button
                  className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                  onClick={() => setIsEditing(true)}
                >
                  Редактировать
                </button>
                <button
                  className="bg-red-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-red-700"
                  onClick={handleDelete}
                >
                  Удалить
                </button>
              </div>
            </div>
          </>
        )}
      </div>

      {/* Управление окружениями */}
      <div className="bg-white rounded-lg p-6 shadow mt-8">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold">Окружения</h2>
          <button
            className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
            onClick={() => {
              setShowEnvForm(true);
              setEnvError('');
            }}
          >
            Создать окружение
          </button>
        </div>

        {showEnvForm && (
          <div className="mb-6 p-4 bg-gray-50 rounded">
            <h3 className="mb-2 font-semibold">Создать новое окружение</h3>
            <form onSubmit={handleCreateEnvironment}>
              <div className="mb-4">
                <label htmlFor="env-name" className="block mb-2 font-medium text-gray-800">
                  Название окружения
                </label>
                <input
                  id="env-name"
                  type="text"
                  className="w-full max-w-[300px] px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                  value={envFormData.name}
                  onChange={(e) => setEnvFormData({ name: e.target.value })}
                  required
                  placeholder="Например: production, staging, development"
                />
              </div>
              {envError && <div className="text-red-600 text-sm mt-1">{envError}</div>}
              <div className="flex gap-2">
                <button type="submit" className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700">
                  Создать
                </button>
                <button
                  type="button"
                  className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                  onClick={() => {
                    setShowEnvForm(false);
                    setEnvFormData({ name: '' });
                    setEnvError('');
                  }}
                >
                  Отмена
                </button>
              </div>
            </form>
          </div>
        )}

        {environments.length === 0 ? (
          <p className="text-gray-500">Окружения не найдены. Создайте первое окружение для этого проекта.</p>
        ) : (
          <table className="w-full border-collapse bg-white rounded-lg overflow-hidden shadow">
            <thead>
              <tr>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Название</th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Создано</th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">Действия</th>
              </tr>
            </thead>
            <tbody>
              {environments.map((env) => (
                <tr 
                  key={env.id} 
                  className="hover:bg-gray-50 cursor-pointer"
                  onClick={() => navigate(`/projects/${project.id}/secrets?env=${encodeURIComponent(env.name)}`)}
                >
                  <td className="px-3 py-3 text-left border-b border-gray-200">
                    <strong className="text-blue-600">{env.name}</strong>
                  </td>
                  <td className="px-3 py-3 text-left border-b border-gray-200">{new Date(env.created_at).toLocaleDateString('ru-RU')}</td>
                  <td className="px-3 py-3 text-left border-b border-gray-200" onClick={(e) => e.stopPropagation()}>
                    <div className="flex gap-2">
                      <Link
                        to={`/projects/${project.id}/secrets?env=${encodeURIComponent(env.name)}`}
                        className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600 no-underline"
                        onClick={(e) => e.stopPropagation()}
                      >
                        Секреты
                      </Link>
                      <button
                        className="bg-red-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-red-700"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteEnvironment(env.id, env.name);
                        }}
                      >
                        Удалить
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};

export default ProjectDetail;

