table! {
    leaderboard (id) {
        id -> Int4,
        scrape_id -> Int4,
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

joinable!(leaderboard -> leaderboard_scrape (scrape_id));

allow_tables_to_appear_in_same_query!(
    leaderboard,
    leaderboard_scrape,
);
