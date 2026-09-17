pub fn build_books_plist(identifiers: &[String]) -> Vec<u8> {
    let mut rows = Vec::new();
    for (index, identifier) in identifiers.iter().enumerate() {
        let mut row = plist::Dictionary::new();
        row.insert("Persistent ID".to_string(), plist::Value::String(identifier.clone()));
        row.insert("Item ID".to_string(), plist::Value::String((index + 1).to_string()));
        row.insert("DSID".to_string(), plist::Value::String("1".to_string()));
        rows.push(plist::Value::Dictionary(row));
    }
    let mut root = plist::Dictionary::new();
    root.insert("Books".to_string(), plist::Value::Array(rows));
    let mut buffer = Vec::new();
    plist::to_writer_binary(&mut buffer, &plist::Value::Dictionary(root)).unwrap_or_default();
    buffer
}
