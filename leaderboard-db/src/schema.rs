table! {
    associated_leaderboard (steam_association_id, leaderboard_id) {
        steam_association_id -> Int4,
        leaderboard_id -> Int4,
    }
}

table! {
    avatar_hash (id) {
        id -> Int4,
        hash -> Varchar,
    }
}

table! {
    avatar_map (url) {
        url -> Varchar,
        avatar_hash_id -> Int4,
    }
}

table! {
    leaderboard (id) {
        id -> Int4,
        leaderboard_scrape_id -> Int4,
        rank -> Int4,
        avatar -> Varchar,
        name -> Varchar,
        rating -> Float4,
        wins -> Int4,
        losses -> Int4,
    }
}

table! {
    leaderboard_scrape (id) {
        id -> Int4,
        at -> Timestamp,
    }
}

table! {
    names (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    steam_association (id) {
        id -> Int4,
        names_id -> Int4,
        avatar_hash_id -> Int4,
        steam_id -> Bytea,
    }
}

joinable!(associated_leaderboard -> leaderboard (leaderboard_id));
joinable!(associated_leaderboard -> steam_association (steam_association_id));
joinable!(leaderboard -> leaderboard_scrape (leaderboard_scrape_id));
joinable!(steam_association -> names (names_id));

allow_tables_to_appear_in_same_query!(
    associated_leaderboard,
    avatar_hash,
    avatar_map,
    leaderboard,
    leaderboard_scrape,
    names,
    steam_association,
);
