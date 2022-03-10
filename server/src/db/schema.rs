table! {
    artifacts (id) {
        id -> BigInt,
        build_id -> BigInt,
        date -> Timestamp,
        hash -> Text,
        author -> Text,
        merged_by -> Text,
        os -> Text,
        arch -> Text,
        channel -> Text,
        file_name -> Text,
        download_uri -> Text,
    }
}
