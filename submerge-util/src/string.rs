pub const HASH_TRUNCATE_SIZE: usize = 5;
pub const ADDRESS_TRUNCATE_SIZE: usize = 3;

pub fn truncate_middle(s: &str, left_chars: usize, right_chars: usize, separator: &str) -> String {
    let char_count = s.chars().count();

    if char_count <= left_chars + right_chars {
        return s.to_string();
    }

    let left: String = s.chars().take(left_chars).collect();
    let right: String = s.chars().skip(char_count - right_chars).collect();

    format!("{left}{separator}{right}")
}

pub fn truncate_end(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "..."
    }
}

pub fn truncate_start(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let skip_count = char_count - max_chars;
        "...".to_string() + &s.chars().skip(skip_count).collect::<String>()
    }
}

pub fn truncate_hash(hash: &str) -> String {
    truncate_middle(hash, HASH_TRUNCATE_SIZE, HASH_TRUNCATE_SIZE, "...")
}

pub fn truncate_address(hash: &str) -> String {
    truncate_middle(hash, ADDRESS_TRUNCATE_SIZE, ADDRESS_TRUNCATE_SIZE, "...")
}
