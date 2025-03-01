-- Create registration token table
CREATE TABLE registration_tokens (
register_token TEXT PRIMARY KEY NOT NULL,
user_id INTEGER NOT NULL,
expires_at TIMESTAMP DEFAULT (DATETIME (CURRENT_TIMESTAMP, '+2 hours')),
FOREIGN KEY (user_id) REFERENCES users (id)
) ;

