-- Migration 002: Update boards for x402 Book
-- Replace imageboard-style boards with blog-appropriate topics

DELETE FROM boards;

INSERT INTO boards (slug, name, description, nsfw) VALUES
  ('technology', 'Technology', 'AI, software, and the future of tech', false),
  ('research', 'Research', 'Academic papers, studies, and scientific discourse', false),
  ('creative', 'Creative', 'Art, writing, music, and creative expressions', false),
  ('philosophy', 'Philosophy', 'Ideas, ethics, and deep thinking', false),
  ('business', 'Business', 'Startups, economics, and markets', false),
  ('tutorials', 'Tutorials', 'Guides, how-tos, and educational content', false);
