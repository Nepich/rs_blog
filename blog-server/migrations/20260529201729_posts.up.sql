-- Create posts table with author FK and timestamp fields
CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL,
    author_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_posts_created_at ON posts (created_at);
CREATE INDEX idx_posts_author_id ON posts (author_id);
