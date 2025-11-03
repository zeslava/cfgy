// Layout компонент с навигацией
import { Outlet, Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

const Layout: React.FC = () => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="flex flex-col min-h-screen">
      <header className="bg-[#2d3748] text-white py-4 shadow">
        <div className="max-w-[1200px] mx-auto px-4 flex items-center justify-between gap-8">
          <h1 className="m-0 text-2xl font-bold">
            <Link to="/projects" className="text-white no-underline hover:opacity-80">
              CFGY
            </Link>
          </h1>
          <nav className="flex gap-6 flex-1">
            <Link to="/projects" className="text-white no-underline px-4 py-2 rounded transition hover:bg-white/10">
              Проекты
            </Link>
            <Link to="/tokens" className="text-white no-underline px-4 py-2 rounded transition hover:bg-white/10">
              Токены
            </Link>
          </nav>
          <div className="flex items-center gap-4">
            <span>{user?.email}</span>
            <button 
              onClick={handleLogout} 
              className="bg-red-600 text-white border-none px-4 py-2 rounded cursor-pointer text-sm transition hover:bg-red-700"
            >
              Выйти
            </button>
          </div>
        </div>
      </header>
      <main className="flex-1 max-w-[1200px] w-full mx-auto p-8 px-4">
        <Outlet />
      </main>
    </div>
  );
};

export default Layout;

