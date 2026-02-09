import { useState, useMemo } from 'react';
import type { Meeting } from '../../types';

interface CalendarProps {
  meetings: Meeting[];
  onDateSelect: (date: Date) => void;
  onMeetingClick: (meeting: Meeting) => void;
  selectedDate: Date;
}

type ViewType = 'month' | 'week' | 'day';

export function Calendar({ meetings, onDateSelect, onMeetingClick, selectedDate }: CalendarProps) {
  const [view, setView] = useState<ViewType>('month');
  const [currentDate, setCurrentDate] = useState(new Date());

  const daysInMonth = useMemo(() => {
    const year = currentDate.getFullYear();
    const month = currentDate.getMonth();
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const startDayOfWeek = firstDay.getDay();

    const days: (Date | null)[] = [];

    // Add empty slots for days before the first of the month
    for (let i = 0; i < startDayOfWeek; i++) {
      days.push(null);
    }

    // Add all days of the month
    for (let i = 1; i <= lastDay.getDate(); i++) {
      days.push(new Date(year, month, i));
    }

    return days;
  }, [currentDate]);

  const weekDays = useMemo(() => {
    const start = new Date(currentDate);
    start.setDate(start.getDate() - start.getDay());

    const days: Date[] = [];
    for (let i = 0; i < 7; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    return days;
  }, [currentDate]);

  const getMeetingsForDate = (date: Date | null): Meeting[] => {
    if (!date) return [];
    return meetings.filter((meeting) => {
      const meetingDate = new Date(meeting.start_time);
      return (
        meetingDate.getFullYear() === date.getFullYear() &&
        meetingDate.getMonth() === date.getMonth() &&
        meetingDate.getDate() === date.getDate()
      );
    });
  };

  const navigatePrev = () => {
    const newDate = new Date(currentDate);
    if (view === 'month') {
      newDate.setMonth(newDate.getMonth() - 1);
    } else if (view === 'week') {
      newDate.setDate(newDate.getDate() - 7);
    } else {
      newDate.setDate(newDate.getDate() - 1);
    }
    setCurrentDate(newDate);
  };

  const navigateNext = () => {
    const newDate = new Date(currentDate);
    if (view === 'month') {
      newDate.setMonth(newDate.getMonth() + 1);
    } else if (view === 'week') {
      newDate.setDate(newDate.getDate() + 7);
    } else {
      newDate.setDate(newDate.getDate() + 1);
    }
    setCurrentDate(newDate);
  };

  const goToToday = () => {
    setCurrentDate(new Date());
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('es-ES', {
      month: 'long',
      year: 'numeric',
    });
  };

  const isToday = (date: Date | null) => {
    if (!date) return false;
    const today = new Date();
    return (
      date.getFullYear() === today.getFullYear() &&
      date.getMonth() === today.getMonth() &&
      date.getDate() === today.getDate()
    );
  };

  const isSelected = (date: Date | null) => {
    if (!date) return false;
    return (
      date.getFullYear() === selectedDate.getFullYear() &&
      date.getMonth() === selectedDate.getMonth() &&
      date.getDate() === selectedDate.getDate()
    );
  };

  const formatTime = (dateStr: string) => {
    return new Date(dateStr).toLocaleTimeString('es-ES', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'scheduled':
        return 'bg-indigo-500';
      case 'inprogress':
        return 'bg-green-500';
      case 'completed':
        return 'bg-gray-500';
      case 'cancelled':
        return 'bg-red-500';
      default:
        return 'bg-indigo-500';
    }
  };

  return (
    <div className="flex flex-col h-full bg-white rounded-lg shadow">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-4">
          <h2 className="text-xl font-semibold text-gray-900 capitalize">
            {formatDate(currentDate)}
          </h2>
          <button
            onClick={goToToday}
            className="px-3 py-1 text-sm bg-indigo-100 text-indigo-700 rounded-md hover:bg-indigo-200"
          >
            Hoy
          </button>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={navigatePrev}
            className="p-2 hover:bg-gray-100 rounded-full"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <button
            onClick={navigateNext}
            className="p-2 hover:bg-gray-100 rounded-full"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>

        <div className="flex bg-slate-100 rounded-lg p-1">
          {(['month', 'week', 'day'] as ViewType[]).map((v) => (
            <button
              key={v}
              onClick={() => setView(v)}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                view === v
                  ? 'bg-white shadow text-indigo-600'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              {v === 'month' ? 'Mes' : v === 'week' ? 'Semana' : 'Dia'}
            </button>
          ))}
        </div>
      </div>

      {/* Calendar Grid */}
      <div className="flex-1 overflow-auto p-4">
        {view === 'month' && (
          <div className="h-full">
            {/* Day headers */}
            <div className="grid grid-cols-7 mb-2">
              {['Dom', 'Lun', 'Mar', 'Mie', 'Jue', 'Vie', 'Sab'].map((day) => (
                <div key={day} className="text-center text-sm font-medium text-gray-500 py-2">
                  {day}
                </div>
              ))}
            </div>

            {/* Days grid */}
            <div className="grid grid-cols-7 gap-1">
              {daysInMonth.map((date, index) => {
                const dayMeetings = getMeetingsForDate(date);
                return (
                  <div
                    key={index}
                    onClick={() => date && onDateSelect(date)}
                    className={`min-h-[100px] p-2 border rounded-lg cursor-pointer transition-colors ${
                      !date
                        ? 'bg-slate-50 border-transparent'
                        : isToday(date)
                        ? 'bg-indigo-50 border-indigo-300'
                        : isSelected(date)
                        ? 'bg-purple-50 border-purple-300'
                        : 'bg-white border-gray-200 hover:bg-gray-50'
                    }`}
                  >
                    {date && (
                      <>
                        <div className={`text-sm font-medium ${
                          isToday(date) ? 'text-indigo-600' : 'text-gray-900'
                        }`}>
                          {date.getDate()}
                        </div>
                        <div className="mt-1 space-y-1">
                          {dayMeetings.slice(0, 3).map((meeting) => (
                            <div
                              key={meeting.id}
                              onClick={(e) => {
                                e.stopPropagation();
                                onMeetingClick(meeting);
                              }}
                              className={`text-xs p-1 rounded truncate text-white ${getStatusColor(meeting.status)} hover:opacity-80`}
                            >
                              {formatTime(meeting.start_time)} {meeting.title}
                            </div>
                          ))}
                          {dayMeetings.length > 3 && (
                            <div className="text-xs text-gray-500">
                              +{dayMeetings.length - 3} mas
                            </div>
                          )}
                        </div>
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {view === 'week' && (
          <div className="h-full">
            <div className="grid grid-cols-7 gap-2">
              {weekDays.map((date) => {
                const dayMeetings = getMeetingsForDate(date);
                return (
                  <div
                    key={date.toISOString()}
                    className={`min-h-[400px] p-2 border rounded-lg ${
                      isToday(date)
                        ? 'bg-indigo-50 border-indigo-300'
                        : 'bg-white border-gray-200'
                    }`}
                  >
                    <div className="text-center mb-2">
                      <div className="text-xs text-gray-500">
                        {date.toLocaleDateString('es-ES', { weekday: 'short' })}
                      </div>
                      <div className={`text-lg font-semibold ${
                        isToday(date) ? 'text-indigo-600' : 'text-gray-900'
                      }`}>
                        {date.getDate()}
                      </div>
                    </div>
                    <div className="space-y-2">
                      {dayMeetings.map((meeting) => (
                        <div
                          key={meeting.id}
                          onClick={() => onMeetingClick(meeting)}
                          className={`p-2 rounded text-white text-sm cursor-pointer ${getStatusColor(meeting.status)} hover:opacity-80`}
                        >
                          <div className="font-medium truncate">{meeting.title}</div>
                          <div className="text-xs opacity-80">
                            {formatTime(meeting.start_time)} - {formatTime(meeting.end_time)}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {view === 'day' && (
          <div className="h-full">
            <div className="text-center mb-4">
              <div className="text-lg font-semibold text-gray-900">
                {currentDate.toLocaleDateString('es-ES', { weekday: 'long', day: 'numeric', month: 'long' })}
              </div>
            </div>
            <div className="space-y-2">
              {getMeetingsForDate(currentDate).map((meeting) => (
                <div
                  key={meeting.id}
                  onClick={() => onMeetingClick(meeting)}
                  className="p-4 bg-white border border-gray-200 rounded-lg cursor-pointer hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <h3 className="font-semibold text-gray-900">{meeting.title}</h3>
                      <p className="text-sm text-gray-500">
                        {formatTime(meeting.start_time)} - {formatTime(meeting.end_time)}
                      </p>
                      {meeting.description && (
                        <p className="text-sm text-gray-600 mt-2">{meeting.description}</p>
                      )}
                    </div>
                    <span className={`px-2 py-1 text-xs text-white rounded ${getStatusColor(meeting.status)}`}>
                      {meeting.status}
                    </span>
                  </div>
                  <div className="mt-3 flex items-center gap-2 text-sm text-gray-500">
                    <span>{meeting.participants.length} participantes</span>
                    {meeting.is_online && (
                      <span className="flex items-center gap-1">
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                        </svg>
                        Online
                      </span>
                    )}
                  </div>
                </div>
              ))}
              {getMeetingsForDate(currentDate).length === 0 && (
                <div className="text-center text-gray-500 py-8">
                  No hay reuniones programadas para este dia
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
