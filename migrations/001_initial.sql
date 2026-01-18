-- Create accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    from_jid TEXT NOT NULL,
    to_jid TEXT NOT NULL,
    body TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'chat',
    stanza_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    delivered_at DATETIME,
    displayed_at DATETIME
);

-- Create roster items table
CREATE TABLE IF NOT EXISTS roster_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_jid TEXT NOT NULL,
    contact_jid TEXT NOT NULL,
    name TEXT,
    subscription TEXT NOT NULL DEFAULT 'none',
    approved BOOLEAN NOT NULL DEFAULT FALSE,
    ask TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_jid, contact_jid)
);

-- Create roster groups table
CREATE TABLE IF NOT EXISTS roster_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_jid TEXT NOT NULL,
    contact_jid TEXT NOT NULL,
    group_name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_jid, contact_jid, group_name)
);

-- Create presence table
CREATE TABLE IF NOT EXISTS presence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jid TEXT NOT NULL UNIQUE,
    show TEXT NOT NULL DEFAULT 'offline',
    status TEXT,
    priority INTEGER DEFAULT 0,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create file transfers table
CREATE TABLE IF NOT EXISTS file_transfers (
    id TEXT PRIMARY KEY,
    from_jid TEXT NOT NULL,
    to_jid TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type TEXT,
    description TEXT,
    local_path TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    progress REAL DEFAULT 0.0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

-- Create MUC rooms table
CREATE TABLE IF NOT EXISTS muc_rooms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_jid TEXT NOT NULL UNIQUE,
    user_jid TEXT NOT NULL,
    nickname TEXT NOT NULL,
    subject TEXT,
    auto_join BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_joined DATETIME
);

-- Create MUC messages table
CREATE TABLE IF NOT EXISTS muc_messages (
    id TEXT PRIMARY KEY,
    room_jid TEXT NOT NULL,
    from_jid TEXT NOT NULL,
    nickname TEXT NOT NULL,
    body TEXT NOT NULL,
    stanza_id TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create MUC occupants table
CREATE TABLE IF NOT EXISTS muc_occupants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_jid TEXT NOT NULL,
    jid TEXT NOT NULL,
    nickname TEXT NOT NULL,
    role TEXT,
    affiliation TEXT,
    status TEXT,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(room_jid, jid)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_messages_from_to ON messages (from_jid, to_jid);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages (created_at);
CREATE INDEX IF NOT EXISTS idx_roster_user_jid ON roster_items (user_jid);
CREATE INDEX IF NOT EXISTS idx_roster_contact_jid ON roster_items (contact_jid);
CREATE INDEX IF NOT EXISTS idx_presence_jid ON presence (jid);
CREATE INDEX IF NOT EXISTS idx_file_transfers_status ON file_transfers (status);
CREATE INDEX IF NOT EXISTS idx_muc_messages_room_jid ON muc_messages (room_jid);
CREATE INDEX IF NOT EXISTS idx_muc_messages_created_at ON muc_messages (created_at);
CREATE INDEX IF NOT EXISTS idx_muc_occupants_room_jid ON muc_occupants (room_jid);