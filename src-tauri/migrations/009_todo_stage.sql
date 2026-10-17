-- Where a todo stands on the board while it's not completed: 'urgent', 'doing', or NULL for "to do", where every todo
-- starts. A completed todo is "done" whatever its stage, which is kept: reopened, it goes back where it was. Within a
-- stage, todos keep the order of their day (`position`), as the completed ones do in the list.
ALTER TABLE todos
ADD COLUMN stage TEXT CHECK (stage IN ('urgent', 'doing'));
