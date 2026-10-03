-- Tag names no longer have to be unique: the index is kept, non-unique, for listing tags by name.
DROP INDEX idx_tags_name;

CREATE INDEX idx_tags_name ON tags (name)
WHERE
  deleted_at IS NULL;
