-- Supersedes 009, whose `stage` was never written: urgency is a flag of its own, kept wherever the todo stands on the
-- board, and being worked on is when it was started.
ALTER TABLE todos
DROP COLUMN stage;

-- Whether it's urgent: shown in the list, and in the board's "Urgent" column until started.
ALTER TABLE todos
ADD COLUMN urgent INTEGER NOT NULL DEFAULT 0 CHECK (urgent IN (0, 1));

-- When it was started, i.e. put in the board's "Doing" column: NULL before, or once put back. Kept once completed, so
-- that reopened, it's back in "Doing". Within a column, todos keep the order of their day (`position`).
ALTER TABLE todos
ADD COLUMN started_at INTEGER;
