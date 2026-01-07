-- Add display_name and triggers to personas table
-- display_name: custom name the persona responds to (if NULL, uses bot's default name)
-- triggers: comma-separated keywords that activate this persona

ALTER TABLE personas ADD COLUMN display_name TEXT;
ALTER TABLE personas ADD COLUMN triggers TEXT;

-- Don't set display_name for Default persona - it should use bot's name
-- Only set display_name for personas that have a specific character name
UPDATE personas SET display_name = 'Чувак' WHERE name = 'Чувак';
UPDATE personas SET display_name = 'Сократ' WHERE name = 'Сократ';
UPDATE personas SET display_name = 'Бро' WHERE name = 'Бро';
