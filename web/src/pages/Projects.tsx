// Страница списка проектов
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { projectsApi } from "../services/api";
import type { Project } from "../services/api";

const Projects: React.FC = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [formData, setFormData] = useState({ name: "", description: "" });
  const navigate = useNavigate();

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    setIsLoading(true);
    setError("");
    try {
      const data = await projectsApi.list();
      setProjects(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка загрузки проектов");
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    try {
      const project = await projectsApi.create({
        name: formData.name,
        description: formData.description || undefined,
      });
      setShowCreateForm(false);
      setFormData({ name: "", description: "" });
      await loadProjects();
      navigate(`/projects/${project.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка создания проекта");
    }
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`Удалить проект "${name}"? Это действие необратимо.`)) return;
    try {
      await projectsApi.delete(id);
      await loadProjects();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Ошибка удаления проекта");
    }
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка проектов...</div>;
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Проекты</h1>
        <button
          className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
          onClick={() => setShowCreateForm(true)}
        >
          Создать проект
        </button>
      </div>

      {showCreateForm && (
        <div className="bg-white rounded-lg p-6 shadow mb-6">
          <h2 className="text-xl font-semibold mb-4">Создать новый проект</h2>
          <form onSubmit={handleCreate}>
            <div className="mb-4">
              <label
                htmlFor="project-name"
                className="block mb-2 font-medium text-gray-800"
              >
                Название
              </label>
              <input
                id="project-name"
                type="text"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                required
                placeholder="Название проекта"
              />
            </div>
            <div className="mb-4">
              <label
                htmlFor="project-description"
                className="block mb-2 font-medium text-gray-800"
              >
                Описание (необязательно)
              </label>
              <textarea
                id="project-description"
                className="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600/20"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                rows={3}
                placeholder="Описание проекта"
              />
            </div>
            {error && <div className="text-red-600 text-sm mt-1">{error}</div>}
            <div className="flex gap-2">
              <button
                type="submit"
                className="bg-blue-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-blue-700"
              >
                Создать
              </button>
              <button
                type="button"
                className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600"
                onClick={() => {
                  setShowCreateForm(false);
                  setFormData({ name: "", description: "" });
                  setError("");
                }}
              >
                Отмена
              </button>
            </div>
          </form>
        </div>
      )}

      {error && !showCreateForm && (
        <div className="text-red-600 mb-4">Ошибка: {error}</div>
      )}

      {projects.length === 0 ? (
        <div className="bg-white rounded-lg p-6 shadow">
          <p>Проекты не найдены. Создайте первый проект.</p>
        </div>
      ) : (
        <div className="overflow-hidden">
          <table className="w-full border-collapse bg-white rounded-lg overflow-hidden shadow">
            <thead>
              <tr>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">
                  Название
                </th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">
                  Описание
                </th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">
                  Создан
                </th>
                <th className="px-3 py-3 text-left border-b border-gray-200 bg-gray-50 font-semibold text-gray-800">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody>
              {projects.map((project) => (
                <tr
                  key={project.id}
                  className="hover:bg-gray-50 cursor-pointer"
                  onClick={() => navigate(`/projects/${project.id}`)}
                >
                  <td className="px-3 py-3 text-left border-b border-gray-200">
                    <span className="text-blue-600 hover:underline">
                      {project.name}
                    </span>
                  </td>
                  <td className="px-3 py-3 text-left border-b border-gray-200">
                    {project.description || "-"}
                  </td>
                  <td className="px-3 py-3 text-left border-b border-gray-200">
                    {new Date(project.created_at).toLocaleDateString("ru-RU")}
                  </td>
                  <td
                    className="px-3 py-3 text-left border-b border-gray-200"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex gap-2">
                      <Link
                        to={`/projects/${project.id}/secrets`}
                        className="bg-gray-500 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-gray-600 no-underline"
                        onClick={(e) => e.stopPropagation()}
                      >
                        Секреты
                      </Link>
                      <button
                        className="bg-red-600 text-white px-4 py-2 rounded cursor-pointer text-sm font-medium transition hover:bg-red-700"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(project.id, project.name);
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
        </div>
      )}
    </div>
  );
};

export default Projects;
