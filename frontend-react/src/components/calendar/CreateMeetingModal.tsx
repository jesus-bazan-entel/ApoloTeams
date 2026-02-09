import { useState, useEffect } from 'react';
import { useStore } from '../../store';
import { apiClient } from '../../api/client';
import type { User, RecurrenceType } from '../../types';

interface CreateMeetingModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialDate?: Date;
}

export function CreateMeetingModal({ isOpen, onClose, initialDate }: CreateMeetingModalProps) {
  const { addMeeting } = useStore();

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [startDate, setStartDate] = useState('');
  const [startTime, setStartTime] = useState('09:00');
  const [endDate, setEndDate] = useState('');
  const [endTime, setEndTime] = useState('10:00');
  const [isOnline, setIsOnline] = useState(true);
  const [location, setLocation] = useState('');
  const [recurrence, setRecurrence] = useState<RecurrenceType>('none');
  const [participantSearch, setParticipantSearch] = useState('');
  const [searchResults, setSearchResults] = useState<User[]>([]);
  const [selectedParticipants, setSelectedParticipants] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialDate) {
      const dateStr = initialDate.toISOString().split('T')[0];
      setStartDate(dateStr);
      setEndDate(dateStr);
    } else {
      const today = new Date().toISOString().split('T')[0];
      setStartDate(today);
      setEndDate(today);
    }
  }, [initialDate, isOpen]);

  useEffect(() => {
    if (participantSearch.length >= 2) {
      const searchUsers = async () => {
        try {
          const users = await apiClient.searchUsers(participantSearch);
          setSearchResults(users.filter(u => !selectedParticipants.find(p => p.id === u.id)));
        } catch (err) {
          console.error('Failed to search users:', err);
        }
      };
      const debounce = setTimeout(searchUsers, 300);
      return () => clearTimeout(debounce);
    } else {
      setSearchResults([]);
    }
  }, [participantSearch, selectedParticipants]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);

    try {
      const startDateTime = new Date(`${startDate}T${startTime}:00`);
      const endDateTime = new Date(`${endDate}T${endTime}:00`);

      if (endDateTime <= startDateTime) {
        setError('La hora de fin debe ser posterior a la hora de inicio');
        setIsLoading(false);
        return;
      }

      const meeting = await apiClient.createMeeting({
        title,
        description: description || undefined,
        start_time: startDateTime.toISOString(),
        end_time: endDateTime.toISOString(),
        is_online: isOnline,
        location: location || undefined,
        recurrence,
        participant_ids: selectedParticipants.map((p) => p.id),
      });

      addMeeting(meeting);
      resetForm();
      onClose();
    } catch (err: any) {
      setError(err.response?.data?.message || 'Error al crear la reunion');
    } finally {
      setIsLoading(false);
    }
  };

  const resetForm = () => {
    setTitle('');
    setDescription('');
    setStartTime('09:00');
    setEndTime('10:00');
    setIsOnline(true);
    setLocation('');
    setRecurrence('none');
    setParticipantSearch('');
    setSearchResults([]);
    setSelectedParticipants([]);
    setError(null);
  };

  const addParticipant = (user: User) => {
    setSelectedParticipants([...selectedParticipants, user]);
    setParticipantSearch('');
    setSearchResults([]);
  };

  const removeParticipant = (userId: string) => {
    setSelectedParticipants(selectedParticipants.filter((p) => p.id !== userId));
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <h2 className="text-xl font-semibold text-gray-900">Nueva Reunion</h2>
          <button
            onClick={() => {
              resetForm();
              onClose();
            }}
            className="p-2 hover:bg-gray-100 rounded-full"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-4 overflow-y-auto max-h-[calc(90vh-120px)]">
          {error && (
            <div className="mb-4 p-3 bg-red-100 text-red-700 rounded-lg">
              {error}
            </div>
          )}

          {/* Title */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Titulo *
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
              className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              placeholder="Nombre de la reunion"
            />
          </div>

          {/* Description */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Descripcion
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              placeholder="Descripcion opcional"
            />
          </div>

          {/* Date and Time */}
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Fecha de inicio *
              </label>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                required
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Hora de inicio *
              </label>
              <input
                type="time"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                required
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Fecha de fin *
              </label>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                required
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Hora de fin *
              </label>
              <input
                type="time"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                required
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
              />
            </div>
          </div>

          {/* Online/Location */}
          <div className="mb-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={isOnline}
                onChange={(e) => setIsOnline(e.target.checked)}
                className="w-4 h-4 text-indigo-600 rounded focus:ring-indigo-500"
              />
              <span className="text-sm font-medium text-gray-700">
                Reunion online (videollamada)
              </span>
            </label>
          </div>

          {!isOnline && (
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Ubicacion
              </label>
              <input
                type="text"
                value={location}
                onChange={(e) => setLocation(e.target.value)}
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
                placeholder="Direccion o sala"
              />
            </div>
          )}

          {/* Recurrence */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Repetir
            </label>
            <select
              value={recurrence}
              onChange={(e) => setRecurrence(e.target.value as RecurrenceType)}
              className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
            >
              <option value="none">No repetir</option>
              <option value="daily">Diariamente</option>
              <option value="weekly">Semanalmente</option>
              <option value="monthly">Mensualmente</option>
            </select>
          </div>

          {/* Participants */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Participantes
            </label>
            <div className="relative">
              <input
                type="text"
                value={participantSearch}
                onChange={(e) => setParticipantSearch(e.target.value)}
                className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-indigo-500"
                placeholder="Buscar usuarios..."
              />
              {searchResults.length > 0 && (
                <div className="absolute z-10 w-full mt-1 bg-white border border-slate-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                  {searchResults.map((user) => (
                    <button
                      key={user.id}
                      type="button"
                      onClick={() => addParticipant(user)}
                      className="w-full px-3 py-2 text-left hover:bg-gray-100 flex items-center gap-2"
                    >
                      <div className="w-8 h-8 bg-indigo-600 rounded-full flex items-center justify-center text-white text-sm font-medium">
                        {user.display_name.charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <div className="text-sm font-medium text-gray-900">
                          {user.display_name}
                        </div>
                        <div className="text-xs text-gray-500">{user.email}</div>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Selected participants */}
            {selectedParticipants.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-2">
                {selectedParticipants.map((user) => (
                  <div
                    key={user.id}
                    className="flex items-center gap-1 px-2 py-1 bg-indigo-100 text-indigo-800 rounded-full text-sm"
                  >
                    <span>{user.display_name}</span>
                    <button
                      type="button"
                      onClick={() => removeParticipant(user.id)}
                      className="ml-1 hover:text-indigo-600"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-4 border-t">
            <button
              type="button"
              onClick={() => {
                resetForm();
                onClose();
              }}
              className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
            >
              Cancelar
            </button>
            <button
              type="submit"
              disabled={isLoading || !title}
              className="btn-teams-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? 'Creando...' : 'Crear Reunion'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
