# accounting-copilot

A CLI accounting system with CRUD functionality in Rust.

## Schema

- `id`
- `journal_date`
- `account_id`
- `amount_debt`
- `amount_credit`
- `total`
- `reconciled`
- `isdeleted` (`yes` or `no` — entries with `yes` are hidden from user)

## Commands

- `add`: Add a journal entry.
- `list`: List all entries (excluding deleted).
- `get`: Show details of a single entry.
- `update`: Update an existing entry.
- `delete`: Soft-delete an entry (set isdeleted to 'yes').
- `exit`: Quit the program.

Entries are persisted in plain text (`entries.txt`) as JSON lines.