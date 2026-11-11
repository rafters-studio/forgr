CREATE INDEX IF NOT EXISTS notes_owner ON notes(owner);
SELECT owner, count(*) FROM notes GROUP BY owner;
