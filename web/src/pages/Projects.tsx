// Страница списка проектов
import { useEffect, useState, useRef } from "react";
import { useNavigate, Link } from "react-router-dom";
import { projectsApi } from "../services/api";
import type { Project } from "../services/api";
import { Button } from "../components/ui/Button";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { useToast } from "../contexts/ToastContext";
import { TrashIcon } from "../components/ui/Icons";
import {
  DataTable,
  DataTableBody,
  DataTableRow,
  DataTableCell,
  DataTableActions,
  DataTableEmpty,
} from "../components/ui/DataTable";
import { sortByString } from "../utils/sort";
import "@/styles/pages/projects.css";
import "@/styles/components/data-table.css";

const Projects: React.FC = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [projectName, setProjectName] = useState("");
  const [deleteConfirm, setDeleteConfirm] = useState<{
    id: string;
    name: string;
  } | null>(null);
  const navigate = useNavigate();
  const { showSuccess, showError } = useToast();
  const formRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadProjects();
  }, []);

  // Закрытие формы при клике вне компонента
  useEffect(() => {
    if (!showCreateForm) return;

    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;

      // Не закрываем, если клик был на форме или на кнопке "Добавить проект"
      if (
        formRef.current?.contains(target) ||
        target.closest("[data-add-project-button]")
      ) {
        return;
      }

      // Закрываем форму и очищаем поле
      setShowCreateForm(false);
      setProjectName("");
      setError("");
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [showCreateForm]);

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
    if (!projectName.trim()) {
      setError("Название проекта обязательно");
      return;
    }
    try {
      const project = await projectsApi.create({
        name: projectName.trim(),
      });
      setShowCreateForm(false);
      setProjectName("");
      showSuccess(`Проект "${project.name}" создан`);
      await loadProjects();
      navigate(`/projects/${project.id}`);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Ошибка создания проекта";
      setError(message);
      showError(message);
    }
  };

  const handleDelete = async (id: string, name: string) => {
    setDeleteConfirm({ id, name });
  };

  const confirmDelete = async () => {
    if (!deleteConfirm) return;
    try {
      await projectsApi.delete(deleteConfirm.id);
      showSuccess(`Проект "${deleteConfirm.name}" удалён`);
      await loadProjects();
    } catch (err) {
      showError(err instanceof Error ? err.message : "Ошибка удаления проекта");
    } finally {
      setDeleteConfirm(null);
    }
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка проектов...</div>;
  }

  return (
    <div className="projects-page">
      <div className="projects-header">
        <div className="projects-title">Мои проекты</div>
        <Button
          variant="basic-secondary"
          size="medium-text"
          className="projects-add-button"
          data-add-project-button
          onClick={() => {
            setShowCreateForm(true);
            setError("");
          }}
        >
          Добавить проект
        </Button>
      </div>

      {error && !showCreateForm && (
        <div className="projects-error">Ошибка: {error}</div>
      )}

      {showCreateForm && (
        <div ref={formRef} className="projects-create-wrapper">
          <form onSubmit={handleCreate} className="projects-form">
            <div className="projects-input-wrapper">
              <input
                type="text"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                placeholder="Название проекта"
                className="projects-input"
                autoFocus
                required
              />
              <Button type="submit" variant="basic-accent" size="small-text">
                Сохранить
              </Button>
            </div>
          </form>
          <div className="projects-divider" />
        </div>
      )}

      <DataTable>
        <DataTableBody>
          {projects.length === 0 ? (
            <DataTableEmpty>
              Проекты не найдены. Создайте первый проект.
            </DataTableEmpty>
          ) : (
            sortByString(projects, (p) => p.name).map((project) => (
              <DataTableRow
                key={project.id}
                onClick={() => navigate(`/projects/${project.id}`)}
              >
                <DataTableCell>
                  <Link
                    to={`/projects/${project.id}`}
                    className="projects-item-link text-style-regular-16"
                    onClick={(e) => e.stopPropagation()}
                  >
                    {project.name}
                  </Link>
                </DataTableCell>
                <DataTableActions>
                  <Button
                    variant="basic-secondary"
                    size="small-icon"
                    className="projects-delete-button"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(project.id, project.name);
                    }}
                  >
                    <TrashIcon className="size-6" />
                  </Button>
                </DataTableActions>
              </DataTableRow>
            ))
          )}
        </DataTableBody>
      </DataTable>

      <ConfirmDialog
        open={!!deleteConfirm}
        title="Удаление проекта"
        message={`Вы уверены, что хотите удалить проект "${deleteConfirm?.name}"? Это действие необратимо.`}
        confirmLabel="Удалить"
        cancelLabel="Отмена"
        variant="danger"
        onConfirm={confirmDelete}
        onCancel={() => setDeleteConfirm(null)}
      />
    </div>
  );
};

export default Projects;
