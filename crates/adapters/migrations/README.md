# Migrations

Each file here is one schema change, named `<UTC timestamp>_<description>.sql`.
`sqlx migrate add <description>` creates one.
The server applies every pending migration at startup, before it serves (spec rule 2.4.6).

- **Forward-only.** There are no down migrations, and an applied file is never edited: sqlx refuses to start if its checksum changed.
- **Expand before you contract.** Rollback moves code but not schema, so the previous revision must still run on the new schema.
  Add the new column or table first; remove the old one in a later release.
