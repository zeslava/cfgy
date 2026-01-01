// Страница управления параметрами
import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import React from "react";
import { parametersApi, configurationsApi } from "../services/api";
import type { Parameters, Configuration } from "../services/api";
import { ParameterValueMasked } from "../components/domain/parameters/ParameterValueMasked";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { TextArea } from "../components/ui/TextArea";
import { Toggle } from "../components/ui/Toggle";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { Breadcrumbs } from "../components/ui/Breadcrumbs";
import { useToast } from "../contexts/ToastContext";
import { EditIcon, TrashIcon } from "../components/ui/Icons";
import { projectsApi } from "../services/api";
import type { Project } from "../services/api";
import {
  DataTable,
  DataTableHeader,
  DataTableHeaderCell,
  DataTableBody,
  DataTableRow,
  DataTableCell,
  DataTableActions,
  DataTableEmpty,
} from "../components/ui/DataTable";
import "@/styles/pages/parameters.css";

const ParametersPage: React.FC = () => {
  const { id, configId } = useParams<{ id: string; configId: string }>();
  const navigate = useNavigate();
  const [configuration, setConfiguration] = useState<Configuration | null>(
    null
  );
  const [configName, setConfigName] = useState("");
  const [project, setProject] = useState<Project | null>(null);
  const [parameters, setParameters] = useState<Parameters>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showEditForm, setShowEditForm] = useState<string | null>(null);
  const [formData, setFormData] = useState({ key: "", value: "" });
  const [isMasked, setIsMasked] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const { showSuccess, showError } = useToast();

  useEffect(() => {
    if (id && configId) {
      loadProject();
      loadConfiguration();
    }
  }, [id, configId]);

  const loadProject = async () => {
    if (!id) return;
    try {
      const data = await projectsApi.get(id);
      setProject(data);
    } catch (err) {
      console.error("Ошибка загрузки проекта:", err);
    }
  };

  useEffect(() => {
    if (id && configName) {
      loadParameters();
    }
  }, [id, configName]);

  const loadConfiguration = async () => {
    if (!id || !configId) return;
    setIsLoading(true);
    try {
      const configs = await configurationsApi.list(id);
      const config = configs.find((c) => c.id === configId);
      if (config) {
        setConfiguration(config);
        setConfigName(config.name);
      }
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Ошибка загрузки конфигурации"
      );
    } finally {
      setIsLoading(false);
    }
  };

  const loadParameters = async () => {
    if (!id || !configName) return;
    setIsLoading(true);
    setError("");
    try {
      const data = await parametersApi.get(id, configName);
      setParameters(data);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Ошибка загрузки параметров"
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateOrUpdate = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!id || !configName) return;
    setError("");
    try {
      // Находим индекс параметра для обновления или добавляем новый
      const existingIndex = parameters.findIndex((p) => p.key === formData.key);
      const updatedParameters = [...parameters];

      if (existingIndex >= 0) {
        // Обновляем существующий параметр
        updatedParameters[existingIndex] = {
          key: formData.key,
          value: formData.value,
        };
      } else {
        // Добавляем новый параметр и сортируем
        updatedParameters.push({ key: formData.key, value: formData.value });
        updatedParameters.sort((a, b) =>
          a.key.localeCompare(b.key, "ru", { sensitivity: "base" })
        );
      }

      // Обновляем локальное состояние сразу, чтобы сохранить сортировку
      setParameters(updatedParameters);

      await parametersApi.set(id, configName, updatedParameters);

      // Перезагружаем данные с сервера для синхронизации
      await loadParameters();

      showSuccess(
        showEditForm
          ? `Параметр "${formData.key}" обновлён`
          : `Параметр "${formData.key}" создан`
      );
      setShowCreateForm(false);
      setShowEditForm(null);
      setFormData({ key: "", value: "" });
      setIsMasked(false);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Ошибка сохранения параметра";
      setError(message);
      showError(message);
      // В случае ошибки перезагружаем данные с сервера
      await loadParameters();
    }
  };

  const handleDelete = async (key: string) => {
    setDeleteConfirm(key);
  };

  const confirmDelete = async () => {
    if (!id || !configName || !deleteConfirm) return;
    try {
      // Удаляем из локального состояния сразу
      const updatedParameters = parameters.filter(
        (p) => p.key !== deleteConfirm
      );
      setParameters(updatedParameters);

      await parametersApi.delete(id, configName, deleteConfirm);
      showSuccess(`Параметр "${deleteConfirm}" удалён`);

      // Перезагружаем данные с сервера для синхронизации
      await loadParameters();
    } catch (err) {
      showError(err instanceof Error ? err.message : "Ошибка удаления");
      // В случае ошибки перезагружаем данные с сервера
      await loadParameters();
    } finally {
      setDeleteConfirm(null);
    }
  };

  const handleEdit = (key: string) => {
    const param = parameters.find((p) => p.key === key);
    setFormData({ key, value: param?.value || "" });
    setShowEditForm(key);
    setShowCreateForm(false);
  };

  const handleCopy = async (value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      showSuccess("Значение скопировано в буфер обмена");
    } catch (err) {
      showError("Ошибка копирования");
    }
  };

  if (isLoading && !configName) {
    return <div className="text-center py-8">Загрузка...</div>;
  }

  if (!configuration) {
    return (
      <div>
        <Breadcrumbs
          items={[
            { label: "Мои проекты", to: "/projects" },
            ...(project
              ? [{ label: project.name, to: `/projects/${id}` }]
              : []),
          ]}
        />
        <div className="mt-4">
          <div className="text-2xl font-bold mb-4">Параметры</div>
          <div className="bg-white rounded-lg p-6 shadow">
            <p>Конфигурация не найдена.</p>
            <Button
              variant="basic-accent"
              size="small-text"
              onClick={() => navigate(`/projects/${id}`)}
              className="mt-4"
            >
              Вернуться к проекту
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="parameters-page">
        <div className="parameters-page-header">
          <Breadcrumbs
            items={[
              { label: "Мои проекты", to: "/projects" },
              ...(project
                ? [{ label: project.name, to: `/projects/${id}` }]
                : []),
              { label: configName },
            ]}
          />
        </div>
        <div className="parameters-page-content">
          <div className="parameters-title-section">
            <div className="parameters-title-wrapper">
              <div className="parameters-title">{configName}</div>
              <p className="parameters-description">Описание</p>
            </div>
            <Button
              variant="basic-secondary"
              size="small-icon"
              onClick={() => {}}
            >
              <EditIcon className="size-6" />
            </Button>
          </div>
          <div className="data-table-section">
            <div className="data-table-section-header">
              <div className="data-table-section-title">Параметры конфига:</div>
              <Button
                variant="basic-secondary"
                size="small-text"
                onClick={() => {
                  setShowCreateForm(true);
                  setShowEditForm(null);
                  setFormData({ key: "", value: "" });
                  setIsMasked(false);
                }}
              >
                Добавить параметр
              </Button>
            </div>

            {error && !showCreateForm && (
              <div className="text-states-error text-regular-16 mb-4">
                Ошибка: {error}
              </div>
            )}

            <DataTable>
              <DataTableHeader>
                <DataTableHeaderCell>Имя</DataTableHeaderCell>
                <DataTableHeaderCell>Значение</DataTableHeaderCell>
                <DataTableHeaderCell align="right">
                  Действия
                </DataTableHeaderCell>
              </DataTableHeader>
              {parameters.length === 0 && !showCreateForm ? (
                <DataTableEmpty>Параметры не найдены</DataTableEmpty>
              ) : (
                <DataTableBody>
                  {showCreateForm && !showEditForm && (
                    <DataTableRow>
                      <DataTableCell>
                        <Input
                          value={formData.key}
                          onChange={(e) =>
                            setFormData({ ...formData, key: e.target.value })
                          }
                          required
                          placeholder="name"
                        />
                      </DataTableCell>
                      <DataTableCell>
                        <div
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: "20px",
                            width: "100%",
                          }}
                        >
                          <div style={{ flex: 1, minWidth: 0 }}>
                            <Input
                              value={formData.value}
                              onChange={(e) =>
                                setFormData({
                                  ...formData,
                                  value: e.target.value,
                                })
                              }
                              required
                              placeholder="value"
                            />
                          </div>
                          <Toggle
                            checked={isMasked}
                            onChange={setIsMasked}
                            aria-label="Маскировать значение"
                          />
                        </div>
                      </DataTableCell>
                      <DataTableActions>
                        <Button
                          size="sm"
                          variant="primary"
                          onClick={(e) => {
                            e.preventDefault();
                            handleCreateOrUpdate();
                          }}
                        >
                          Сохранить
                        </Button>
                      </DataTableActions>
                    </DataTableRow>
                  )}
                  {parameters.map((param) => (
                    <DataTableRow
                      key={param.key}
                      isEditing={showEditForm === param.key}
                    >
                      <DataTableCell>{param.key}</DataTableCell>
                      <DataTableCell>
                        {showEditForm === param.key ? (
                          <TextArea
                            value={
                              showEditForm === param.key &&
                              formData.key === param.key
                                ? formData.value
                                : param.value
                            }
                            onChange={(e) =>
                              setFormData({
                                key: param.key,
                                value: e.target.value,
                              })
                            }
                            rows={2}
                          />
                        ) : (
                          <ParameterValueMasked
                            value={param.value}
                            parameterKey={param.key}
                            onCopy={() => handleCopy(param.value)}
                          />
                        )}
                      </DataTableCell>
                      <DataTableActions>
                        {showEditForm === param.key ? (
                          <Button
                            size="sm"
                            variant="primary"
                            onClick={() => {
                              if (formData.key === param.key) {
                                handleCreateOrUpdate();
                              }
                            }}
                          >
                            Сохранить
                          </Button>
                        ) : (
                          <>
                            <Button
                              variant="basic-secondary"
                              size="small-icon"
                              onClick={() => handleEdit(param.key)}
                            >
                              <EditIcon className="size-6" />
                            </Button>
                            <Button
                              variant="basic-secondary"
                              size="small-icon"
                              onClick={() => handleDelete(param.key)}
                            >
                              <TrashIcon className="size-6" />
                            </Button>
                          </>
                        )}
                      </DataTableActions>
                    </DataTableRow>
                  ))}
                </DataTableBody>
              )}
            </DataTable>
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={!!deleteConfirm}
        title="Удаление параметра"
        message={`Вы уверены, что хотите удалить параметр "${deleteConfirm}"?`}
        confirmLabel="Удалить"
        cancelLabel="Отмена"
        variant="danger"
        onConfirm={confirmDelete}
        onCancel={() => setDeleteConfirm(null)}
      />
    </>
  );
};

export default ParametersPage;
