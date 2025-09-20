
/// 转义 MarkdownV2 特殊字符
pub fn escape_markdown_v2(text: &str) -> String {
    let chars_to_escape = r"_*[]()~`>#+-=|{}.!";
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if chars_to_escape.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}
