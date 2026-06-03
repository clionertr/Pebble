pub(crate) fn parse_csv_query_ids(raw: Option<&str>) -> Option<Vec<String>> {
    raw.map(|value| {
        value
            .split(',')
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_query_ids_trim_and_drop_empty_items() {
        let ids = parse_csv_query_ids(Some(" inbox, , archive ,,sent "));

        assert_eq!(
            ids,
            Some(vec![
                "inbox".to_string(),
                "archive".to_string(),
                "sent".to_string()
            ])
        );
    }

    #[test]
    fn csv_query_ids_preserve_missing_parameter() {
        assert_eq!(parse_csv_query_ids(None), None);
    }
}
