table! {
    airtable (id) {
        id -> Int4,
        aid -> Varchar,
        content -> Varchar,
        created_time -> Nullable<Timestamp>,
    }
}

table! {
    invites (id) {
        id -> Int4,
        code -> Varchar,
        actionrole -> Nullable<Int8>,
        actionchannel -> Nullable<Int8>,
        used_count -> Int4,
    }
}

table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
        date -> Nullable<Timestamp>,
    }
}

table! {
    messages_edits (id) {
        id -> Int4,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
        date -> Nullable<Timestamp>,
        parrent_message_id -> Int8,
    }
}

table! {
    projects (id) {
        id -> Int4,
        message_id -> Int8,
        channel_id -> Int8,
        codex -> Varchar,
        client -> Varchar,
        lead -> Varchar,
        deadline -> Varchar,
        description -> Varchar,
        contexte -> Varchar,
        created_at -> Timestamp,
        pinned_message_id -> Nullable<Int8>,
    }
}

table! {
    storage (id) {
        id -> Int4,
        datatype -> Int8,
        dataid -> Nullable<Int8>,
        data -> Varchar,
        date -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Int4,
        discordid -> Int8,
        role -> Varchar,
    }
}

joinable!(messages_edits -> messages (parrent_message_id));

allow_tables_to_appear_in_same_query!(
    airtable,
    invites,
    messages,
    messages_edits,
    projects,
    storage,
    users,
);
