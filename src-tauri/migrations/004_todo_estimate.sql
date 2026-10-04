-- Positive; in `estimate_unit`.
ALTER TABLE todos
ADD COLUMN estimate INTEGER CHECK (estimate > 0);

-- 'minutes' or 'points'; set exactly when `estimate` is.
ALTER TABLE todos
ADD COLUMN estimate_unit TEXT CHECK (estimate_unit IN ('minutes', 'points')) CHECK ((estimate IS NULL) = (estimate_unit IS NULL));
