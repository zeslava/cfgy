// Страница профиля
import { useEffect, useState } from "react";
import { tokensApi } from "../services/api";
import type { Token, CreateTokenResponse } from "../services/api";
import { Avatar } from "../components/ui/Avatar";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { TrashIcon, CopyIcon } from "../components/ui/Icons";
import { useAuth } from "../contexts/useAuth";
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
import { getInitialsFromEmail, getDisplayName } from "@/utils/avatar";
import { sortByString } from "../utils/sort";
import { useToast } from "../contexts/ToastContext";
import "@/styles/pages/profile.css";

const Profile: React.FC = () => {
  const { user } = useAuth();
  const { showSuccess, showError } = useToast();
  const [tokens, setTokens] = useState<Token[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newToken, setNewToken] = useState<CreateTokenResponse | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    expires_at: "",
  });

  useEffect(() => {
    loadTokens();
  }, []);

  const loadTokens = async () => {
    setIsLoading(true);
    setError("");
    try {
      const data = await tokensApi.list();
      setTokens(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ошибка загрузки токенов");
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreate = async () => {
    if (!formData.name.trim()) {
      showError("Введите имя токена");
      return;
    }

    setError("");
    try {
      // Вычисляем количество дней до истечения
      let expires_in_days: number | undefined;
      if (formData.expires_at) {
        const expiresDate = new Date(formData.expires_at);
        const today = new Date();
        const diffTime = expiresDate.getTime() - today.getTime();
        expires_in_days = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
        if (expires_in_days <= 0) {
          showError("Дата истечения должна быть в будущем");
          return;
        }
      }

      const response = await tokensApi.create({
        name: formData.name,
        expires_in_days,
      });
      setNewToken(response);
      await loadTokens();
      showSuccess("Токен создан");
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Ошибка создания токена";
      setError(message);
      showError(message);
    }
  };

  const handleSaveAndClose = () => {
    setShowCreateForm(false);
    setNewToken(null);
    setFormData({ name: "", expires_at: "" });
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`Удалить токен "${name}"?`)) return;
    try {
      await tokensApi.delete(id);
      await loadTokens();
      showSuccess(`Токен "${name}" удалён`);
    } catch (err) {
      showError(err instanceof Error ? err.message : "Ошибка удаления токена");
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      showSuccess("Токен скопирован в буфер обмена");
    });
  };

  if (isLoading) {
    return <div className="text-center py-8">Загрузка токенов...</div>;
  }

  return (
    <div className="profile-page">
      {/* Секция профиля */}
      <div className="profile-user-section">
        <Avatar
          size="big"
          initials={getInitialsFromEmail(user?.email || getDisplayName(user) || "U")}
        />
        <div className="profile-user-info">
          <p className="profile-user-email">
            {getDisplayName(user)}
          </p>
          <Button variant="basic-secondary" size="big-text" fullWidth>
            Изменить пароль
          </Button>
        </div>
      </div>

      {/* Секция токенов */}
      <div className="data-table-section">
        <div className="data-table-section-header">
          <div className="data-table-section-title">Токены</div>
          <Button
            variant="basic-secondary"
            size="medium-text"
            onClick={() => {
              setShowCreateForm(true);
              setNewToken(null);
              setFormData({ name: "", expires_at: "" });
            }}
          >
            Создать новый токен
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
            <DataTableHeaderCell>Окончание действия</DataTableHeaderCell>
            <DataTableHeaderCell align="right">Действия</DataTableHeaderCell>
          </DataTableHeader>
          <DataTableBody>
            {/* Форма создания токена - inline в таблице */}
            {showCreateForm && (
              <DataTableRow isEditing>
                <DataTableCell>
                  <Input
                    value={formData.name}
                    onChange={(e) =>
                      setFormData({ ...formData, name: e.target.value })
                    }
                    placeholder="Macbook"
                    disabled={!!newToken}
                  />
                </DataTableCell>
                <DataTableCell>
                  <Input
                    type="date"
                    value={formData.expires_at}
                    onChange={(e) =>
                      setFormData({ ...formData, expires_at: e.target.value })
                    }
                    placeholder="31.08.2026"
                    disabled={!!newToken}
                  />
                </DataTableCell>
                <DataTableCell>
                  {newToken ? (
                    <div className="flex flex-col gap-1">
                      <div className="relative">
                        <Input
                          value={newToken.token}
                          readOnly
                          rightIcon={
                            <button
                              type="button"
                              onClick={() => copyToClipboard(newToken.token)}
                              className="text-basic-gray hover:text-text-dark transition"
                            >
                              <CopyIcon className="size-5" />
                            </button>
                          }
                        />
                      </div>
                      <p className="font-inter font-light text-[14px] text-text-dark">
                        Сохраните токен себе - здесь его нельзя будет посмотреть
                        больше
                      </p>
                    </div>
                  ) : null}
                </DataTableCell>
                <DataTableActions>
                  {newToken ? (
                    <Button
                      variant="basic-secondary"
                      size="small-text"
                      onClick={handleSaveAndClose}
                    >
                      Закрыть
                    </Button>
                  ) : (
                    <Button
                      variant="primary"
                      size="small-text"
                      onClick={handleCreate}
                    >
                      Сохранить
                    </Button>
                  )}
                </DataTableActions>
              </DataTableRow>
            )}

            {/* Список существующих токенов */}
            {tokens.length === 0 && !showCreateForm ? (
              <DataTableEmpty>
                Токены не найдены. Создайте первый токен для работы с API.
              </DataTableEmpty>
            ) : (
              sortByString(tokens, (t) => t.name).map((token) => (
                <DataTableRow key={token.id}>
                  <DataTableCell>{token.name}</DataTableCell>
                  <DataTableCell>
                    {token.expires_at
                      ? new Date(token.expires_at).toLocaleDateString("ru-RU")
                      : "бессрочно"}
                  </DataTableCell>
                  <DataTableActions>
                    <Button
                      variant="basic-secondary"
                      size="small-icon"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(token.id, token.name);
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
      </div>
    </div>
  );
};

export default Profile;
