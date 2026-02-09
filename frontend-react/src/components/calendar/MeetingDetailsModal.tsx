import { useState } from 'react';
import { useStore } from '../../store';
import { apiClient } from '../../api/client';
import type { Meeting, MeetingResponseStatus } from '../../types';

interface MeetingDetailsModalProps {
  meeting: Meeting | null;
  onClose: () => void;
}

export function MeetingDetailsModal({ meeting, onClose }: MeetingDetailsModalProps) {
  const { currentUser, updateMeeting, removeMeeting } = useStore();
  const [isLoading, setIsLoading] = useState(false);

  if (!meeting) return null;

  const isOrganizer = meeting.organizer.id === currentUser?.id;
  const myParticipant = meeting.participants.find((p) => p.user.id === currentUser?.id);

  const formatDateTime = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('es-ES', {
      weekday: 'long',
      day: 'numeric',
      month: 'long',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const handleRespond = async (response: MeetingResponseStatus) => {
    setIsLoading(true);
    try {
      const updated = await apiClient.respondToMeeting(meeting.id, response);
      updateMeeting(updated);
    } catch (err) {
      console.error('Failed to respond:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = async () => {
    if (!confirm('Estas seguro de que deseas cancelar esta reunion?')) return;
    setIsLoading(true);
    try {
      await apiClient.cancelMeeting(meeting.id);
      removeMeeting(meeting.id);
      onClose();
    } catch (err) {
      console.error('Failed to cancel:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm('Estas seguro de que deseas eliminar esta reunion?')) return;
    setIsLoading(true);
    try {
      await apiClient.deleteMeeting(meeting.id);
      removeMeeting(meeting.id);
      onClose();
    } catch (err) {
      console.error('Failed to delete:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const getStatusBadge = (status: string) => {
    const colors: Record<string, string> = {
      scheduled: 'bg-indigo-100 text-indigo-800',
      inprogress: 'bg-green-100 text-green-800',
      completed: 'bg-gray-100 text-gray-800',
      cancelled: 'bg-red-100 text-red-800',
    };
    const labels: Record<string, string> = {
      scheduled: 'Programada',
      inprogress: 'En progreso',
      completed: 'Completada',
      cancelled: 'Cancelada',
    };
    return (
      <span className={`px-2 py-1 text-xs font-medium rounded ${colors[status] || colors.scheduled}`}>
        {labels[status] || status}
      </span>
    );
  };

  const getResponseBadge = (status: MeetingResponseStatus) => {
    const colors: Record<string, string> = {
      pending: 'bg-yellow-100 text-yellow-800',
      accepted: 'bg-green-100 text-green-800',
      declined: 'bg-red-100 text-red-800',
      tentative: 'bg-indigo-100 text-indigo-800',
    };
    const labels: Record<string, string> = {
      pending: 'Pendiente',
      accepted: 'Aceptado',
      declined: 'Rechazado',
      tentative: 'Tentativo',
    };
    return (
      <span className={`px-2 py-0.5 text-xs rounded ${colors[status]}`}>
        {labels[status]}
      </span>
    );
  };

  return (
    <div className="modal-overlay">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-lg max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <div className="flex items-center gap-3">
            <h2 className="text-xl font-semibold text-gray-900">{meeting.title}</h2>
            {getStatusBadge(meeting.status)}
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 rounded-full"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-4 overflow-y-auto max-h-[calc(90vh-160px)]">
          {/* Time */}
          <div className="mb-4">
            <div className="flex items-center gap-2 text-gray-600">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div>
                <div className="text-sm font-medium text-gray-900">
                  {formatDateTime(meeting.start_time)}
                </div>
                <div className="text-sm">
                  hasta {formatDateTime(meeting.end_time)}
                </div>
              </div>
            </div>
          </div>

          {/* Location/Online */}
          <div className="mb-4">
            <div className="flex items-center gap-2 text-gray-600">
              {meeting.is_online ? (
                <>
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  <div>
                    <div className="text-sm font-medium text-gray-900">Reunion online</div>
                    {meeting.meeting_link && (
                      <a href={meeting.meeting_link} className="text-sm text-indigo-600 hover:underline">
                        Unirse a la reunion
                      </a>
                    )}
                  </div>
                </>
              ) : (
                <>
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  <span className="text-sm">{meeting.location || 'Sin ubicacion especificada'}</span>
                </>
              )}
            </div>
          </div>

          {/* Description */}
          {meeting.description && (
            <div className="mb-4">
              <h3 className="text-sm font-medium text-gray-700 mb-1">Descripcion</h3>
              <p className="text-sm text-gray-600">{meeting.description}</p>
            </div>
          )}

          {/* Organizer */}
          <div className="mb-4">
            <h3 className="text-sm font-medium text-gray-700 mb-2">Organizador</h3>
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 bg-indigo-600 rounded-full flex items-center justify-center text-white text-sm font-medium">
                {meeting.organizer.display_name.charAt(0).toUpperCase()}
              </div>
              <div>
                <div className="text-sm font-medium text-gray-900">
                  {meeting.organizer.display_name}
                </div>
                <div className="text-xs text-gray-500">{meeting.organizer.email}</div>
              </div>
            </div>
          </div>

          {/* Participants */}
          <div className="mb-4">
            <h3 className="text-sm font-medium text-gray-700 mb-2">
              Participantes ({meeting.participants.length})
            </h3>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {meeting.participants.map((participant) => (
                <div key={participant.user.id} className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 bg-gray-400 rounded-full flex items-center justify-center text-white text-sm font-medium">
                      {participant.user.display_name.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <div className="text-sm font-medium text-gray-900">
                        {participant.user.display_name}
                        {participant.is_organizer && (
                          <span className="ml-1 text-xs text-gray-500">(Organizador)</span>
                        )}
                      </div>
                    </div>
                  </div>
                  {getResponseBadge(participant.response_status)}
                </div>
              ))}
            </div>
          </div>

          {/* Response buttons for participants */}
          {!isOrganizer && myParticipant && meeting.status === 'scheduled' && (
            <div className="mb-4 p-3 bg-gray-50 rounded-lg">
              <p className="text-sm text-gray-600 mb-2">Tu respuesta:</p>
              <div className="flex gap-2">
                <button
                  onClick={() => handleRespond('accepted')}
                  disabled={isLoading}
                  className={`px-3 py-1.5 text-sm rounded-md ${
                    myParticipant.response_status === 'accepted'
                      ? 'bg-green-600 text-white'
                      : 'bg-green-100 text-green-700 hover:bg-green-200'
                  }`}
                >
                  Aceptar
                </button>
                <button
                  onClick={() => handleRespond('tentative')}
                  disabled={isLoading}
                  className={`px-3 py-1.5 text-sm rounded-md ${
                    myParticipant.response_status === 'tentative'
                      ? 'bg-indigo-600 text-white'
                      : 'bg-indigo-100 text-indigo-700 hover:bg-indigo-200'
                  }`}
                >
                  Tentativo
                </button>
                <button
                  onClick={() => handleRespond('declined')}
                  disabled={isLoading}
                  className={`px-3 py-1.5 text-sm rounded-md ${
                    myParticipant.response_status === 'declined'
                      ? 'bg-red-600 text-white'
                      : 'bg-red-100 text-red-700 hover:bg-red-200'
                  }`}
                >
                  Rechazar
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Footer actions for organizer */}
        {isOrganizer && meeting.status === 'scheduled' && (
          <div className="flex justify-end gap-2 p-4 border-t">
            <button
              onClick={handleCancel}
              disabled={isLoading}
              className="px-4 py-2 text-yellow-700 bg-yellow-100 rounded-lg hover:bg-yellow-200 disabled:opacity-50"
            >
              Cancelar Reunion
            </button>
            <button
              onClick={handleDelete}
              disabled={isLoading}
              className="px-4 py-2 text-red-700 bg-red-100 rounded-lg hover:bg-red-200 disabled:opacity-50"
            >
              Eliminar
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
