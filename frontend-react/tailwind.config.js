/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // MS Teams Primary Colors
        teams: {
          purple: {
            DEFAULT: '#6264A7',
            50: '#F3F2FC',
            100: '#E8E8F8',
            200: '#D1D1F1',
            300: '#B9B9EA',
            400: '#8E90C8',
            500: '#6264A7',
            600: '#4F5196',
            700: '#3D3F85',
            800: '#2B2D74',
            900: '#1A1B63',
          },
          blue: {
            DEFAULT: '#464EB8',
            50: '#EEF0FC',
            100: '#DDE1F9',
            200: '#BBC3F3',
            300: '#99A5ED',
            400: '#6F7AD3',
            500: '#464EB8',
            600: '#383EA3',
            700: '#2A2E8E',
            800: '#1C1E79',
            900: '#0E0E64',
          },
        },
        // MS Teams UI Colors
        surface: {
          DEFAULT: '#F5F5F5',
          dark: '#292929',
          darker: '#1F1F1F',
          light: '#FFFFFF',
          hover: '#E5E5E5',
          'hover-dark': '#3D3D3D',
        },
        // Sidebar colors
        sidebar: {
          DEFAULT: '#292929',
          hover: '#3D3D3D',
          active: '#464775',
          text: '#FFFFFF',
          'text-muted': '#B3B3B3',
        },
        // Status colors
        status: {
          online: '#92C353',
          away: '#FFAA44',
          busy: '#C4314B',
          dnd: '#C4314B',
          offline: '#8A8886',
        },
        // Message colors
        message: {
          bg: '#FFFFFF',
          'bg-own': '#E5E5FC',
          hover: '#F5F5F5',
          border: '#E0E0E0',
        },
        // Text colors
        text: {
          primary: '#252423',
          secondary: '#605E5C',
          muted: '#8A8886',
          inverse: '#FFFFFF',
        },
      },
      fontFamily: {
        sans: ['Segoe UI', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
      boxShadow: {
        'teams': '0 1.6px 3.6px 0 rgba(0,0,0,0.132), 0 0.3px 0.9px 0 rgba(0,0,0,0.108)',
        'teams-lg': '0 6.4px 14.4px 0 rgba(0,0,0,0.132), 0 1.2px 3.6px 0 rgba(0,0,0,0.108)',
        'teams-xl': '0 25.6px 57.6px 0 rgba(0,0,0,0.22), 0 4.8px 14.4px 0 rgba(0,0,0,0.18)',
      },
      borderRadius: {
        'teams': '4px',
        'teams-lg': '8px',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-in-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'pulse-teams': 'pulseTeams 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
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
        pulseTeams: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },
      },
    },
  },
  plugins: [],
}
