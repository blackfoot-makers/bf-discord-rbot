table! {
    airtable (id) {
        id -> Int4,
        aid -> Varchar,
        created_time -> Int8,
        content -> Varchar,
        triggered -> Bool,
    }
}

table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
    }
}

table! {
    users (id) {
        id -> Int4,
        discordid -> Int8,
        role -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    airtable,
    messages,
    users,
);
