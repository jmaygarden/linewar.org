WITH map AS (
    SELECT
        DISTINCT avatar AS url,
        substring(avatar, '\/([0-9A-Za-z]+)_[^\/]*$') AS hash
    FROM
        leaderboard
        LEFT JOIN avatar_map ON leaderboard.avatar = avatar_map.url
    WHERE
        avatar_map.url IS NULL
),
hash AS (
    INSERT INTO
        avatar_hash(hash)
    SELECT
        map.hash
    FROM
        map ON CONFLICT DO NOTHING
)
INSERT INTO
    avatar_map(url, avatar_hash_id) (
        SELECT
            map.url,
            avatar_hash.id
        FROM
            map
            INNER JOIN avatar_hash ON map.hash = avatar_hash.hash
    ) ON CONFLICT DO NOTHING;