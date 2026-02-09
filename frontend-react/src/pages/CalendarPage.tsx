import { useEffect, useState } from 'react';
import { useStore } from '../store';
import { apiClient } from '../api/client';
import { Calendar } from '../components/calendar/Calendar';
import { CreateMeetingModal } from '../components/calendar/CreateMeetingModal';
import { MeetingDetailsModal } from '../components/calendar/MeetingDetailsModal';
import type { Meeting } from '../types';

export function CalendarPage() {
  const { meetings, setMeetings, selectedMeeting, setSelectedMeeting } = useStore();
  const [isLoading, setIsLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedDate, setSelectedDate] = useState(new Date());

  useEffect(() => {
    loadMeetings();
  }, []);

  const loadMeetings = async () => {
    try {
      const data = await apiClient.listMeetings();
      setMeetings(data);
    } catch (err) {
      console.error('Failed to load meetings:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDateSelect = (date: Date) => {
    setSelectedDate(date);
  };

  const handleMeetingClick = (meeting: Meeting) => {
    setSelectedMeeting(meeting);
  };

  const handleCreateMeeting = () => {
    setShowCreateModal(true);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-slate-100">
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-white border-b border-slate-200">
        <div className="flex items-center gap-3">
          <svg className="w-8 h-8 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <div>
            <h1 className="text-xl font-semibold text-gray-900">Calendario</h1>
            <p className="text-sm text-gray-500">
              {meetings.length} reunion{meetings.length !== 1 ? 'es' : ''} programada{meetings.length !== 1 ? 's' : ''}
            </p>
          </div>
        </div>
        <button
          onClick={handleCreateMeeting}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition-colors"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Nueva Reunion
        </button>
      </div>

      {/* Content area: Calendar + Upcoming meetings */}
      <div className="flex-1 flex min-h-0 overflow-hidden">
        {/* Calendar */}
        <div className="flex-1 p-4 overflow-auto">
          <Calendar
            meetings={meetings}
            onDateSelect={handleDateSelect}
            onMeetingClick={handleMeetingClick}
            selectedDate={selectedDate}
          />
        </div>

        {/* Upcoming meetings sidebar */}
        <div className="hidden xl:flex flex-col w-80 bg-white border-l border-slate-200 flex-shrink-0">
          <div className="p-4 border-b">
            <h2 className="font-semibold text-gray-900">Proximas Reuniones</h2>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-3">
            {meetings
              .filter((m) => new Date(m.start_time) >= new Date() && m.status === 'scheduled')
              .slice(0, 5)
              .map((meeting) => (
                <div
                  key={meeting.id}
                  onClick={() => handleMeetingClick(meeting)}
                  className="p-3 bg-gray-50 rounded-lg cursor-pointer hover:bg-gray-100 transition-colors"
                >
                  <div className="font-medium text-gray-900 truncate">{meeting.title}</div>
                  <div className="text-sm text-gray-500 mt-1">
                    {new Date(meeting.start_time).toLocaleDateString('es-ES', {
                      weekday: 'short',
                      day: 'numeric',
                      month: 'short',
                    })}
                    {' - '}
                    {new Date(meeting.start_time).toLocaleTimeString('es-ES', {
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </div>
                  <div className="flex items-center gap-2 mt-2 text-xs text-gray-500">
                    <span>{meeting.participants.length} participantes</span>
                    {meeting.is_online && (
                      <span className="flex items-center gap-1">
                        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                        </svg>
                        Online
                      </span>
                    )}
                  </div>
                </div>
              ))}
            {meetings.filter((m) => new Date(m.start_time) >= new Date() && m.status === 'scheduled').length === 0 && (
              <div className="text-center text-gray-500 py-8">
                No hay reuniones proximas
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Modals */}
      <CreateMeetingModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        initialDate={selectedDate}
      />

      <MeetingDetailsModal
        meeting={selectedMeeting}
        onClose={() => setSelectedMeeting(null)}
      />
    </div>
  );
}
