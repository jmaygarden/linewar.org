WITH lag_games AS (
    SELECT
        at,
        steam_id,
        wins + losses AS cur_games,
        LAG(wins + losses) OVER (
            PARTITION BY steam_id
            ORDER BY
                at DESC
        ) AS lag_games
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
        cur_games <> lag_games
        AND at > NOW() - INTERVAL '1 week'
    GROUP BY
        steam_id
)
SELECT
    rank,
    name,
    rating,
    wins,
    losses,
    current_leaderboard.steam_id,
    last_at
FROM
    current_leaderboard
    INNER JOIN recent_leaders ON current_leaderboard.steam_id = recent_leaders.steam_id
ORDER BY
    rank;