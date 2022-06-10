WITH lag_games AS (
    SELECT
        *,
        LAG(wins + losses) OVER (
            PARTITION BY steam_id
            ORDER BY
                at DESC
        ) AS prev_games
    FROM
        leaderboard_view
    ORDER BY
        at DESC
),
recent_leaders AS (
    SELECT
        steam_id,
        MAX(at) AS last_at
    FROM
        lag_games
    WHERE
        wins + losses <> prev_games
        AND at > NOW() - INTERVAL '1 week'
    GROUP BY
        steam_id
    ORDER BY
        MAX(rating) DESC
    LIMIT
        100
)
SELECT
    rank,
    name,
    rating,
    wins,
    losses,
    leaderboard_view.steam_id,
    last_at
FROM
    leaderboard_view
    INNER JOIN recent_leaders ON leaderboard_view.steam_id = recent_leaders.steam_id
    INNER JOIN (
        SELECT
            at
        FROM
            leaderboard_scrape
        ORDER BY
            at DESC
        LIMIT
            1
    ) AS scrape ON leaderboard_view.at = scrape.at
ORDER BY
    rating DESC