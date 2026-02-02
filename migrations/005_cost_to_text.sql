-- Convert cost/amount columns from BIGINT to TEXT to store raw 256-bit token values
-- Raw token values with 18 decimals can exceed 64-bit integer limits

ALTER TABLE threads ALTER COLUMN cost TYPE TEXT USING cost::TEXT;
ALTER TABLE earnings ALTER COLUMN amount TYPE TEXT USING amount::TEXT;
