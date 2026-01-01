// Layout компонент с навигацией
import { Outlet } from "react-router-dom";
import { Header } from "./Header";
import "@/styles/components/layout.css";

const Layout: React.FC = () => {
  return (
    <div className="layout">
      <Header />
      <main className="layout-main">
        <Outlet />
      </main>
    </div>
  );
};

export default Layout;
