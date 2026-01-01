// Главный компонент приложения
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "./contexts/AuthContext";
import { useAuth } from "./contexts/useAuth";
import { ToastProvider } from "./contexts/ToastContext";
import Layout from "./components/Layout";
import Login from "./pages/Login";
import Register from "./pages/Register";
import ForgotPasswordStep1 from "./pages/ForgotPasswordStep1";
import ForgotPasswordStep2 from "./pages/ForgotPasswordStep2";
import ForgotPasswordStep3 from "./pages/ForgotPasswordStep3";
import Projects from "./pages/Projects";
import Configurations from "./pages/Configurations";
import Parameters from "./pages/Parameters";
import Profile from "./pages/Profile";

// Защищенный роут
const PrivateRoute: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        Загрузка...
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
};

// Публичный роут (только для неавторизованных)
const PublicRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        Загрузка...
      </div>
    );
  }

  if (isAuthenticated) {
    return <Navigate to="/projects" replace />;
  }

  return <>{children}</>;
};

function AppRoutes() {
  return (
    <Routes>
      <Route
        path="/login"
        element={
          <PublicRoute>
            <Login />
          </PublicRoute>
        }
      />
      <Route
        path="/register"
        element={
          <PublicRoute>
            <Register />
          </PublicRoute>
        }
      />
      <Route
        path="/forgot-password/step1"
        element={
          <PublicRoute>
            <ForgotPasswordStep1 />
          </PublicRoute>
        }
      />
      <Route
        path="/forgot-password/step2"
        element={
          <PublicRoute>
            <ForgotPasswordStep2 />
          </PublicRoute>
        }
      />
      <Route
        path="/forgot-password/step3"
        element={
          <PublicRoute>
            <ForgotPasswordStep3 />
          </PublicRoute>
        }
      />
      <Route
        path="/"
        element={
          <PrivateRoute>
            <Layout />
          </PrivateRoute>
        }
      >
        <Route index element={<Navigate to="/projects" replace />} />
        <Route path="projects" element={<Projects />} />
        <Route path="projects/:id" element={<Configurations />} />
        <Route
          path="projects/:id/configurations/:configId/parameters"
          element={<Parameters />}
        />
        <Route path="profile" element={<Profile />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <AuthProvider>
      <ToastProvider>
        <BrowserRouter>
          <AppRoutes />
        </BrowserRouter>
      </ToastProvider>
    </AuthProvider>
  );
}

export default App;
