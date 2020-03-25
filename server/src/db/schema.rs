table! {
    artifacts (id) {
        id -> Integer,
        build_id -> Integer,
        date -> Timestamp,
        hash -> Text,
        author -> Text,
        merged_by -> Text,
        platform -> Text,
        channel -> Text,
        file_name -> Text,
        download_uri -> Text,
    }
}
