import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Eye, EyeOff, ArrowRight, ShieldCheck, Mail, Lock } from 'lucide-react';
import { useStore } from '../store';
import { apiClient } from '../api/client';

function LoginPage() {
  const navigate = useNavigate();
  const { setAuth } = useStore();
  const [formData, setFormData] = useState({ email: '', password: '' });
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);

    try {
      const response = await apiClient.login(formData);
      setAuth(response.user, response.access_token, response.refresh_token);
      navigate('/');
    } catch (err: any) {
      setError(err.response?.data?.error?.message || 'Credenciales inválidas');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen w-full flex items-center justify-center bg-[#f3f2f1] p-4">
      <div className="w-full max-w-[440px] bg-white rounded-lg shadow-xl p-8 md:p-12">
        <div className="mb-8 flex items-center gap-3">
          <div className="w-10 h-10 flex items-center justify-center bg-[#4B53BC] rounded text-white font-bold text-xl">
            TS
          </div>
          <h1 className="text-xl font-bold text-gray-800">TeamStream</h1>
        </div>

        <div className="mb-6">
          <h2 className="text-2xl font-semibold text-gray-900">Iniciar sesión</h2>
          <p className="text-sm text-gray-600 mt-1">Usa tu cuenta corporativa para continuar</p>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-50 text-red-600 text-xs rounded border border-red-100">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-1">
            <label className="text-xs font-semibold text-gray-500 uppercase">Correo Electrónico</label>
            <div className="relative">
              <Mail className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
              <input
                type="email"
                required
                className="w-full h-11 pl-10 pr-4 border-b-2 border-gray-200 focus:border-[#4B53BC] outline-none transition-colors text-sm"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              />
            </div>
          </div>

          <div className="space-y-1">
            <label className="text-xs font-semibold text-gray-500 uppercase">Contraseña</label>
            <div className="relative">
              <Lock className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
              <input
                type={showPassword ? 'text' : 'password'}
                required
                className="w-full h-11 pl-10 pr-12 border-b-2 border-gray-200 focus:border-[#4B53BC] outline-none transition-colors text-sm"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400"
              >
                {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
              </button>
            </div>
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full h-11 bg-[#4B53BC] text-white rounded font-semibold flex items-center justify-center gap-2 hover:bg-[#3b43a3] transition-all disabled:opacity-70"
          >
            {isLoading ? (
              <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              'Entrar'
            )}
            <ArrowRight size={18} />
          </button>

          <Link
            to="/register"
            className="w-full h-11 text-[#4B53BC] text-sm font-semibold hover:bg-gray-50 rounded transition-colors flex items-center justify-center"
          >
            ¿No tienes cuenta? Regístrate
          </Link>
        </form>
      </div>
      <div className="absolute bottom-8 text-gray-400 text-xs flex items-center gap-2 font-medium">
        <ShieldCheck size={14} />
        <span>Conexión segura via Rust + Postgres</span>
      </div>
    </div>
  );
}

export default LoginPage;
