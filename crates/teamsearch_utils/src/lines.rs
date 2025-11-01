/// Get the line number (1-indexed) and byte range for a given byte position.
///
/// Returns a tuple of (line_number, line_start_byte, line_end_byte) where
/// line_number is 1-indexed.
pub fn get_line_range(contents: &str, byte_pos: usize) -> (usize, usize, usize) {
    // Count newlines before the position to get the line number.
    // Line numbers are 1-indexed, so we add 1.
    let line_num = contents[..byte_pos].bytes().filter(|&b| b == b'\n').count() + 1;

    // Find the start and end of the line containing this byte position.
    let line_start = if line_num == 1 {
        0
    } else {
        contents[..byte_pos].rfind('\n').map(|pos| pos + 1).unwrap_or(0)
    };

    let line_end =
        contents[byte_pos..].find('\n').map(|offset| byte_pos + offset).unwrap_or(contents.len());

    (line_num, line_start, line_end)
}
