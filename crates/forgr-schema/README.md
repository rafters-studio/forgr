# forgr-schema

What an app's schema.json says: its tables, columns, keys and CHECK lists.

forgr-schema reads a schema.json in migratr's format, whoever wrote it, into a
model. It replays the file's SQL into an in-memory SQLite, tables first, and
takes each table's columns, primary key and foreign keys from what SQLite
reports. A `CHECK (col IN (...))` list comes from the CREATE TABLE text. The
file is never written and no database file is opened.

Most users want the [forgr](https://github.com/rafters-studio/forgr#readme)
crate, which uses this one to build test data from the schema. Depend on
forgr-schema directly only to read a schema.json yourself.

`Schema::read(text)` returns a `Schema`: its `tables` in file order, each a
`Table` with `columns`, `primary_key` and `foreign_keys`. Virtual tables and
their shadow tables are left out. `replay(text)` returns the in-memory
connection the replay built. Both return an `Error`: `Parse` when the text is not in migratr's format,
`Open` when no in-memory SQLite could be opened, and `Replay` when an object's
SQL did not replay. `Schema::read` can also return `Read`, when a replayed
table could not be read back.

forgr-schema is test infrastructure for forgr. It does not run migrations or
move rows; those belong to migratr and smugglr.
