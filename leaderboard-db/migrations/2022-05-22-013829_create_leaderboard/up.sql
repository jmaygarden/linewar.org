CREATE TABLE leaderboard_scrape (
	id SERIAL PRIMARY KEY,
	at TIMESTAMP NOT NULL
);

CREATE TABLE leaderboard (
	id SERIAL PRIMARY KEY,
	scrape_id INT NOT NULL,
	rank INT NOT NULL,
	avatar VARCHAR NOT NULL,
	name VARCHAR NOT NULL,
	rating REAL NOT NULL,
	wins INT NOT NULL,
	losses INT NOT NULL,
	CONSTRAINT fk_scrape FOREIGN KEY(scrape_id) REFERENCES leaderboard_scrape(id)
);
