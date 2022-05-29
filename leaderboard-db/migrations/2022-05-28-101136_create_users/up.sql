ALTER TABLE
    leaderboard RENAME COLUMN scrape_id TO leaderboard_scrape_id;

CREATE TABLE names (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE avatar_hash (
    id SERIAL PRIMARY KEY,
    hash VARCHAR NOT NULL UNIQUE
);

CREATE TABLE avatar_map (
    url VARCHAR PRIMARY KEY,
    avatar_hash_id INT NOT NULL,
    CONSTRAINT fk_avatar_hash FOREIGN KEY (avatar_hash_id) REFERENCES avatar_hash(id)
);

CREATE TABLE steam_association (
    id SERIAL PRIMARY KEY,
    names_id INT NOT NULL,
    avatar_hash_id INT NOT NULL,
    steam_id BYTEA NOT NULL,
    CONSTRAINT fk_name FOREIGN KEY (names_id) REFERENCES names(id),
    CONSTRAINT fk_avatar_hash FOREIGN KEY (avatar_hash_id) REFERENCES avatar_hash(id)
);

CREATE INDEX leaderboard_to_steam_index ON steam_association (names_id, avatar_hash_id);

CREATE INDEX steam_to_leaderboard_index ON steam_association (steam_id);

CREATE TABLE associated_leaderboard (
    steam_association_id INT NOT NULL,
    leaderboard_id INT NOT NULL,
    CONSTRAINT pk_associated_leaderboard PRIMARY KEY (steam_association_id, leaderboard_id),
    CONSTRAINT fk_steam_association FOREIGN KEY(steam_association_id) REFERENCES steam_association(id),
    CONSTRAINT fk_leaderboard FOREIGN KEY(leaderboard_id) REFERENCES leaderboard(id)
)