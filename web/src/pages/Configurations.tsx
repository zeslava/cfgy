// Страница конфигураций проекта
import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { projectsApi, configurationsApi } from "../services/api";
import type { Project, Configuration } from "../services/api";
import { EditIcon, TrashIcon } from "../components/ui/Icons";
import { Button } from "../components/ui/Button";
import { Breadcrumbs } from "../components/ui/Breadcrumbs";
import { List } from "../components/ui/List";
import { ListItem } from "../components/ui/ListItem";
import { sortByString } from "../utils/sort";
import "@/styles/pages/configurations.css";
import "@/styles/components/data-table.css";

const Configurations: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [configurations, setConfigurations] = useState<Configuration[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [showConfigForm, setShowConfigForm] = useState(false);
  const [configFormData, setConfigFormData] = useState({ name: "" });
  const [configError, setConfigError] = useState("");
  const [formData, setFormData] = useState({ name: "", description: "" });

  useEffect(() => {
    if (id) {
      loadProject();
      loadConfigurations();
    }
  }, [id]);

  const loadProject = async () => {
    if (!id) return;
    setIsLoading(true);
    setError("");
    try {
      const data = await projectsApi.get(id);
      setProject(data);
      setFormData({ name: data.name, description: data.description || "" });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка загрузки проекта");
    } finally {
      setIsLoading(false);
    }
  };

  const loadConfigurations = async () => {
    if (!id) return;
    try {
      const configs = await configurationsApi.list(id);
      setConfigurations(configs);
    } catch (err) {
      console.error("Ошибка загрузки конфигураций:", err);
    }
  };

  const handleCreateConfiguration = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    setConfigError("");
    try {
      await configurationsApi.create(id, { name: configFormData.name });
      setConfigFormData({ name: "" });
      setShowConfigForm(false);
      await loadConfigurations();
    } catch (err) {
      setConfigError(
        err instanceof Error ? err.message : "Ошибка создания конфигурации"
      );
    }
  };

  const handleDeleteConfiguration = async (
    configId: string,
    configName: string
  ) => {
    if (!id) return;
    if (
      !confirm(
        `Удалить конфигурацию "${configName}"? Это действие необратимо и удалит все параметры этой конфигурации.`
      )
    )
      return;
    try {
      await configurationsApi.delete(id, configId);
      await loadConfigurations();
    } catch (err) {
      alert(
        err instanceof Error ? err.message : "Ошибка удаления конфигурации"
      );
    }
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id) return;
    setError("");
    try {
      const updated = await projectsApi.update(id, {
        name: formData.name,
        description: formData.description || undefined,
      });
      setProject(updated);
      setIsEditing(false);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Ошибка обновления проекта"
      );
    }
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка...</div>;
  }

  if (error || !project) {
    return (
      <div className="text-states-error text-regular-16">
        Ошибка: {error || "Проект не найден"}
      </div>
    );
  }

  return (
    <div className="configurations-page">
      <div className="configurations-header">
        <Breadcrumbs
          items={[
            { label: "Мои проекты", to: "/projects" },
            { label: project.name },
          ]}
        />
        <div className="configurations-title-section">
          <div className="configurations-title-wrapper">
            {isEditing ? (
              <form onSubmit={handleUpdate} className="configurations-form">
                <input
                  type="text"
                  className="configurations-form-input"
                  value={formData.name}
                  onChange={(e) =>
                    setFormData({ ...formData, name: e.target.value })
                  }
                  required
                />
                <textarea
                  className="configurations-form-textarea"
                  value={formData.description || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, description: e.target.value })
                  }
                  rows={1}
                  placeholder="Описание"
                />
                {error && (
                  <div className="text-states-error text-regular-16 mt-1">
                    {error}
                  </div>
                )}
                <div className="configurations-form-actions">
                  <Button type="submit" variant="primary" size="sm">
                    Сохранить
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    onClick={() => {
                      setIsEditing(false);
                      setFormData({
                        name: project.name,
                        description: project.description || "",
                      });
                      setError("");
                    }}
                  >
                    Отмена
                  </Button>
                </div>
              </form>
            ) : (
              <>
                <div className="configurations-title-section">
                  <div className="configurations-title-wrapper">
                    <div className="configurations-title">{project.name}</div>
                    <p className="configurations-description">
                      {project.description || "Описание"}
                    </p>
                  </div>
                  <Button
                    variant="basic-secondary"
                    size="small-icon"
                    onClick={() => setIsEditing(true)}
                  >
                    <EditIcon className="size-6" />
                  </Button>
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Управление конфигурациями */}
      <div className="configurations-content">
        <div className="data-table-section-header">
          <div className="data-table-section-title">Конфиги проекта:</div>
          <Button
            variant="basic-secondary"
            size="small-text"
            onClick={() => {
              setShowConfigForm(true);
              setConfigError("");
            }}
          >
            Добавить конфиг
          </Button>
        </div>

        {showConfigForm && (
          <div className="config-create-wrapper">
            <form
              onSubmit={handleCreateConfiguration}
              className="config-create-form"
            >
              <div className="config-create-input-row">
                <input
                  type="text"
                  value={configFormData.name}
                  onChange={(e) => setConfigFormData({ name: e.target.value })}
                  placeholder="Название конфига"
                  className="config-create-input"
                  autoFocus
                  required
                />
                <Button type="submit" variant="basic-accent" size="small-text">
                  Сохранить
                </Button>
              </div>
              {configError && (
                <div className="text-states-error text-regular-16 mt-2">
                  {configError}
                </div>
              )}
            </form>
            <div className="config-create-divider" />
          </div>
        )}

        {configurations.length === 0 ? (
          <p className="data-table-empty">
            Конфигурации не найдены. Создайте первую конфигурацию для этого
            проекта.
          </p>
        ) : (
          <List>
            {sortByString(configurations, (c) => c.name).map((config) => (
              <ListItem key={config.id}>
                <Link
                  to={`/projects/${project.id}/configurations/${config.id}/parameters`}
                  className="configurations-item-link text-style-regular-16"
                >
                  {config.name}
                </Link>
                <Button
                  variant="basic-secondary"
                  size="small-icon"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeleteConfiguration(config.id, config.name);
                  }}
                >
                  <TrashIcon className="size-6" />
                </Button>
              </ListItem>
            ))}
          </List>
        )}
      </div>
    </div>
  );
};

export default Configurations;
