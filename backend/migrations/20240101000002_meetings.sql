-- Meetings table for scheduling meetings/events
CREATE TABLE IF NOT EXISTS meetings (
    id UUID PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    organizer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    status TEXT NOT NULL DEFAULT 'scheduled',
    is_online BOOLEAN NOT NULL DEFAULT TRUE,
    meeting_link TEXT,
    location TEXT,
    recurrence TEXT NOT NULL DEFAULT 'none',
    channel_id UUID REFERENCES channels(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_meetings_organizer ON meetings(organizer_id);
CREATE INDEX IF NOT EXISTS idx_meetings_start_time ON meetings(start_time);
CREATE INDEX IF NOT EXISTS idx_meetings_end_time ON meetings(end_time);
CREATE INDEX IF NOT EXISTS idx_meetings_status ON meetings(status);
CREATE INDEX IF NOT EXISTS idx_meetings_channel ON meetings(channel_id);

-- Meeting participants table
CREATE TABLE IF NOT EXISTS meeting_participants (
    id UUID PRIMARY KEY NOT NULL,
    meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    response_status TEXT NOT NULL DEFAULT 'pending',
    is_organizer BOOLEAN NOT NULL DEFAULT FALSE,
    invited_at TIMESTAMPTZ NOT NULL,
    responded_at TIMESTAMPTZ,
    UNIQUE(meeting_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_meeting_participants_meeting ON meeting_participants(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_participants_user ON meeting_participants(user_id);
CREATE INDEX IF NOT EXISTS idx_meeting_participants_status ON meeting_participants(response_status);
