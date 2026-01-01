// Компонент Header для переиспользования
import { Link, useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/useAuth";
import { Avatar } from "./ui/Avatar";
import { Button } from "./ui/Button";
import { ExitIcon } from "./ui/Icons";
import { getInitialsFromEmail } from "@/utils/avatar";
import logoImage from "@/assets/logo.svg";
import "@/styles/components/header.css";

export const Header: React.FC = () => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  return (
    <header className="header">
      <div className="header-content">
        <Link to="/projects" className="header-logo-link">
          <img src={logoImage} alt="CFGY" className="header-logo" />
        </Link>
        <div className="header-right">
          <Link to="/profile" className="header-user-link">
            <Avatar
              size="small"
              initials={getInitialsFromEmail(user?.username || "U")}
              className="bg-white text-text-dark"
            />
            <span className="header-user-email">
              {user?.username || "Пользователь"}
            </span>
          </Link>
          <Button
            variant="basic-secondary"
            size="small-icon"
            onClick={handleLogout}
          >
            <ExitIcon className="size-6" />
          </Button>
        </div>
      </div>
    </header>
  );
};
