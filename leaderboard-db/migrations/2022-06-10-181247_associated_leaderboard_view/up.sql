CREATE VIEW leaderboard_view AS
SELECT
    leaderboard.id,
    leaderboard_scrape_id,
    at,
    rank,
    avatar,
    name,
    rating,
    wins,
    losses,
    steam_id
FROM
    leaderboard
    INNER JOIN leaderboard_scrape ON leaderboard.leaderboard_scrape_id = leaderboard_scrape.id
    LEFT JOIN associated_leaderboard ON leaderboard.id = associated_leaderboard.leaderboard_id
    LEFT JOIN steam_association ON steam_association.id = associated_leaderboard.steam_association_id
ORDER BY
    at DESC,
    rank;

CREATE VIEW current_leaderboard AS
SELECT
    leaderboard.id,
    at,
    rank,
    avatar,
    name,
    rating,
    wins,
    losses,
    steam_id
FROM
    leaderboard
    INNER JOIN leaderboard_scrape ON leaderboard.leaderboard_scrape_id = leaderboard_scrape.id
    LEFT JOIN associated_leaderboard ON leaderboard.id = associated_leaderboard.leaderboard_id
    LEFT JOIN steam_association ON steam_association.id = associated_leaderboard.steam_association_id
WHERE
    leaderboard_scrape_id = (
        SELECT
            id
        FROM
            leaderboard_scrape
        ORDER BY
            at DESC
        LIMIT
            1
    )
ORDER BY
    rank;