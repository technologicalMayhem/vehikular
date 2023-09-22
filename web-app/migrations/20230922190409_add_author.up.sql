ALTER TABLE maintenance_history ADD COLUMN author_user_id int4;
ALTER TABLE maintenance_history ADD CONSTRAINT "fk-history-user" FOREIGN KEY (author_user_id) REFERENCES "user"(id);