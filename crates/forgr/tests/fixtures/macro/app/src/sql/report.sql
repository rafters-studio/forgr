SELECT owner, count(*) FROM notes GROUP BY owner;
SELECT e.id, e.status FROM events e WHERE e.source IS NULL;
