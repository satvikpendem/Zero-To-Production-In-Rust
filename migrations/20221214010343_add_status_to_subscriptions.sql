-- Add migration script here
ALTER TABLE subscriptions
ADD COLUMN STATUS TEXT NULL;