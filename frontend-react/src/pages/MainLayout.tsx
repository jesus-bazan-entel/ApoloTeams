import { useState } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { useStore } from '../store';
import {
  Home,
  MessageSquare,
  CalendarDays,
  Settings,
  LogOut,
  Menu,
  X,
} from 'lucide-react';

interface NavItem {
  path: string;
  icon: React.ReactNode;
  label: string;
  matchPaths: string[];
}

export default function MainLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { logout } = useStore();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const navItems: NavItem[] = [
    {
      path: '/',
      icon: <Home className="w-[22px] h-[22px]" strokeWidth={1.8} />,
      label: 'Inicio',
      matchPaths: ['/', '/teams'],
    },
    {
      path: '/chat',
      icon: <MessageSquare className="w-[22px] h-[22px]" strokeWidth={1.8} />,
      label: 'Chat',
      matchPaths: ['/chat'],
    },
    {
      path: '/calendar',
      icon: <CalendarDays className="w-[22px] h-[22px]" strokeWidth={1.8} />,
      label: 'Calendario',
      matchPaths: ['/calendar'],
    },
  ];

  const bottomItems: NavItem[] = [
    {
      path: '/settings',
      icon: <Settings className="w-5 h-5" strokeWidth={1.8} />,
      label: 'Ajustes',
      matchPaths: ['/settings'],
    },
  ];

  const isActive = (item: NavItem) => {
    return item.matchPaths.some((p) => {
      if (p === '/') return location.pathname === '/';
      return location.pathname.startsWith(p);
    });
  };

  const handleNavClick = (path: string) => {
    navigate(path);
    setMobileMenuOpen(false);
  };

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="flex h-screen bg-slate-50 overflow-hidden">
      {/* Mobile overlay */}
      {mobileMenuOpen && (
        <div
          className="fixed inset-0 bg-black/30 backdrop-blur-sm z-40 lg:hidden"
          onClick={() => setMobileMenuOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`
          fixed lg:relative inset-y-0 left-0 z-50
          w-[68px] bg-gradient-to-b from-slate-900 via-slate-900 to-slate-800
          flex flex-col items-center py-3 flex-shrink-0
          transform transition-transform duration-300 ease-in-out
          ${mobileMenuOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
          shadow-xl lg:shadow-none
        `}
      >
        {/* Close button mobile */}
        <button
          onClick={() => setMobileMenuOpen(false)}
          className="lg:hidden absolute top-3 right-3 text-slate-400 hover:text-white"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Logo */}
        <div className="relative mb-1">
          <div className="w-11 h-11 rounded-xl bg-gradient-to-br from-indigo-500 to-cyan-400 flex items-center justify-center shadow-lg shadow-indigo-500/25">
            <span className="text-white font-bold text-base tracking-tight">AN</span>
          </div>
        </div>

        {/* Divider */}
        <div className="w-8 h-px bg-slate-700/60 my-3" />

        {/* Main nav */}
        <nav className="flex-1 flex flex-col items-center gap-1 w-full px-2">
          {navItems.map((item) => {
            const active = isActive(item);
            return (
              <button
                key={item.path}
                onClick={() => handleNavClick(item.path)}
                className={`
                  group relative w-full flex flex-col items-center justify-center
                  py-2.5 rounded-xl transition-all duration-200
                  ${active
                    ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/30'
                    : 'text-slate-400 hover:bg-white/[0.07] hover:text-slate-200'
                  }
                `}
                title={item.label}
              >
                {/* Active indicator bar */}
                {active && (
                  <div className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 bg-white rounded-r-full -ml-2" />
                )}
                {item.icon}
                <span className={`text-[10px] mt-1 font-medium leading-none ${active ? 'text-white' : 'text-slate-500 group-hover:text-slate-300'}`}>
                  {item.label}
                </span>

                {/* Tooltip */}
                <div className="absolute left-full ml-3 px-2.5 py-1 bg-slate-800 text-white text-xs rounded-md
                  opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150
                  pointer-events-none whitespace-nowrap shadow-lg z-50 hidden lg:block">
                  {item.label}
                  <div className="absolute right-full top-1/2 -translate-y-1/2 border-4 border-transparent border-r-slate-800" />
                </div>
              </button>
            );
          })}
        </nav>

        {/* Divider */}
        <div className="w-8 h-px bg-slate-700/60 my-3" />

        {/* Bottom nav */}
        <div className="flex flex-col items-center gap-1 w-full px-2">
          {bottomItems.map((item) => {
            const active = isActive(item);
            return (
              <button
                key={item.path}
                onClick={() => handleNavClick(item.path)}
                className={`
                  group relative w-full flex flex-col items-center justify-center
                  py-2.5 rounded-xl transition-all duration-200
                  ${active
                    ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/30'
                    : 'text-slate-400 hover:bg-white/[0.07] hover:text-slate-200'
                  }
                `}
                title={item.label}
              >
                {active && (
                  <div className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 bg-white rounded-r-full -ml-2" />
                )}
                {item.icon}
                <span className={`text-[10px] mt-1 font-medium leading-none ${active ? 'text-white' : 'text-slate-500 group-hover:text-slate-300'}`}>
                  {item.label}
                </span>
              </button>
            );
          })}

          {/* Logout */}
          <button
            onClick={handleLogout}
            className="group relative w-full flex flex-col items-center justify-center
              py-2.5 rounded-xl transition-all duration-200
              text-slate-500 hover:bg-red-500/10 hover:text-red-400"
            title="Cerrar sesion"
          >
            <LogOut className="w-5 h-5" strokeWidth={1.8} />
            <span className="text-[10px] mt-1 font-medium leading-none text-slate-500 group-hover:text-red-400">
              Salir
            </span>
          </button>
        </div>

        {/* Bottom padding */}
        <div className="h-2" />
      </aside>

      {/* Mobile menu trigger */}
      <button
        onClick={() => setMobileMenuOpen(true)}
        className="lg:hidden fixed top-3 left-3 z-30 p-2 bg-white rounded-xl shadow-md border border-slate-200 text-slate-600 hover:text-slate-900"
      >
        <Menu className="w-5 h-5" />
      </button>

      {/* Main content area */}
      <main className="flex-1 min-w-0 overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
