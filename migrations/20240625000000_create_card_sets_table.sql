-- Create card_sets table
CREATE TABLE IF NOT EXISTS card_sets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    release_date TIMESTAMPTZ NOT NULL,
    icon_url TEXT,
    total_cards INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on release_date
CREATE INDEX idx_card_sets_release_date ON card_sets(release_date);

-- Initialize card_sets table with data
INSERT INTO card_sets (id, name, code, release_date, icon_url, total_cards) VALUES
    ('b2e5b23b-6188-42d5-a340-729f7ba7fc29', 'Raices misticas', 'IDRMP', '2021-01-01', 'https://images.pokemontcg.io/base/symbol.png', 66),
    ('898615d5-c1e3-4e02-906a-18929e4a0deb', 'Guerra roja', 'LGRO', '2022-01-01', 'https://images.pokemontcg.io/xy/symbol.png', 108),
    ('6d8172ef-aa38-450c-aca0-ad866b4f9ada', 'Titanes de la corteza (Mazo)', 'TCDE', '2023-01-01', 'https://images.pokemontcg.io/sm/symbol.png', 28),
    ('2938307d-3833-42ef-83d0-c95e3fd525aa', 'Mini flores y tumbas', 'CMFT', '2024-01-01', 'https://images.pokemontcg.io/swsh/symbol.png', 14),
    ('e5fa7505-a4b4-4d0a-8588-a2eaef3e192f', 'Titanes de la corteza y ojos del oceano', 'TCOO', '2025-01-01', 'https://images.pokemontcg.io/swsh/symbol.png', 131);