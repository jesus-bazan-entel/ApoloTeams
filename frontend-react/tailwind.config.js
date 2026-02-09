/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Apolo Next Primary Colors
        teams: {
          purple: {
            DEFAULT: '#4F46E5',
            50: '#EEF2FF',
            100: '#E0E7FF',
            200: '#C7D2FE',
            300: '#A5B4FC',
            400: '#818CF8',
            500: '#6366F1',
            600: '#4F46E5',
            700: '#4338CA',
            800: '#3730A3',
            900: '#312E81',
          },
          blue: {
            DEFAULT: '#06B6D4',
            50: '#ECFEFF',
            100: '#CFFAFE',
            200: '#A5F3FC',
            300: '#67E8F9',
            400: '#22D3EE',
            500: '#06B6D4',
            600: '#0891B2',
            700: '#0E7490',
            800: '#155E75',
            900: '#164E63',
          },
        },
        // Apolo Next UI Colors
        surface: {
          DEFAULT: '#F8FAFC',
          dark: '#0F172A',
          darker: '#020617',
          light: '#FFFFFF',
          hover: '#F1F5F9',
          'hover-dark': '#1E293B',
        },
        // Sidebar colors
        sidebar: {
          DEFAULT: '#0F172A',
          hover: '#1E293B',
          active: '#4F46E5',
          'text-muted': '#94A3B8',
        },
        // Status colors
        status: {
          online: '#22C55E',
          away: '#F59E0B',
          busy: '#EF4444',
          dnd: '#EF4444',
          offline: '#94A3B8',
        },
        // Message colors
        message: {
          bg: '#FFFFFF',
          'bg-own': '#EEF2FF',
          hover: '#F8FAFC',
          border: '#E2E8F0',
        },
        // Text colors
        text: {
          primary: '#0F172A',
          secondary: '#475569',
          muted: '#94A3B8',
          inverse: '#FFFFFF',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
      boxShadow: {
        'teams': '0 1px 3px 0 rgba(0,0,0,0.1), 0 1px 2px -1px rgba(0,0,0,0.1)',
        'teams-md': '0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)',
        'teams-lg': '0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)',
        'teams-xl': '0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)',
        'glow': '0 0 20px rgba(79, 70, 229, 0.3)',
        'glow-cyan': '0 0 20px rgba(6, 182, 212, 0.3)',
      },
      borderRadius: {
        'teams': '8px',
        'teams-lg': '16px',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-in-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-soft': 'pulseSoft 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'bounce-soft': 'bounceSoft 1s infinite',
        'glow-pulse': 'glowPulse 2s ease-in-out infinite',
        'shimmer': 'shimmer 2s linear infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateX(-10px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        pulseSoft: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
        bounceSoft: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-5px)' },
        },
        glowPulse: {
          '0%, 100%': { boxShadow: '0 0 20px rgba(79, 70, 229, 0.3)' },
          '50%': { boxShadow: '0 0 30px rgba(79, 70, 229, 0.5)' },
        },
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
      },
      transitionDuration: {
        '200': '200ms',
        '300': '300ms',
      },
      transitionTimingFunction: {
        'teams': 'cubic-bezier(0.4, 0, 0.2, 1)',
      },
      backgroundImage: {
        'gradient-apolo': 'linear-gradient(135deg, #4F46E5 0%, #06B6D4 100%)',
        'gradient-cosmic': 'linear-gradient(135deg, #312E81 0%, #4F46E5 50%, #06B6D4 100%)',
      },
    },
  },
  plugins: [],
}
