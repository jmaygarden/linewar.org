INSERT INTO
    associated_leaderboard
SELECT
    leaderboard.id,
    steam_association.id
FROM
    leaderboard
    INNER JOIN names on leaderboard.name = names.name
    INNER JOIN avatar_map ON leaderboard.avatar = avatar_map.url
    INNER JOIN steam_association ON names.id = steam_association.names_id
    AND avatar_map.avatar_hash_id = steam_association.avatar_hash_id
    LEFT JOIN associated_leaderboard ON leaderboard.id = associated_leaderboard.leaderboard_id
    AND steam_association.id = associated_leaderboard.steam_association_id
WHERE
    associated_leaderboard.leaderboard_id IS NULL;