DROP TABLE associated_leaderboard;

DROP TABLE steam_association;

DROP TABLE avatar_map;

DROP TABLE avatar_hash;

DROP TABLE names;

ALTER TABLE
    leaderboard RENAME COLUMN leaderboard_scrape_id TO scrape_id;