#[derive(Debug, Clone)]
pub struct Envelope {
    pub subject: Option<String>,
}

pub fn fetch_envelopes(buf: &[u8]) -> Vec<(u32, Envelope)> {
    let mut res = Vec::new();
    let mut i = 0;
    while let Some(pos) = find_subsequence(&buf[i..], b"* ") {
        let start = i + pos + 2; // after "* "
        // parse number
        let (num, after_num) = match parse_number(buf, start) {
            Some(v) => v,
            None => {
                i = start;
                continue;
            }
        };

        // find FETCH then ENVELOPE (
        let mut j = after_num;
        if let Some(fetch_pos) = find_subsequence(&buf[j..], b" FETCH ") {
            j = j + fetch_pos + " FETCH ".len();
        } else {
            i = j;
            continue;
        }
        if let Some(env_pos) = find_subsequence(&buf[j..], b"ENVELOPE (") {
            j = j + env_pos + "ENVELOPE (".len();
        } else {
            i = j;
            continue;
        }

        // parse date, subject
        if let Some((_date, next)) = parse_string(buf, j) {
            j = next;
        } else {
            i = j;
            continue;
        }
        let subject = match parse_string(buf, j) {
            Some((s, next)) => {
                j = next;
                s
            }
            None => None,
        };
        res.push((num, Envelope { subject }));
        i = j;
    }
    res
}

fn parse_string(buf: &[u8], mut i: usize) -> Option<(Option<String>, usize)> {
    skip_ws(buf, &mut i);
    if i >= buf.len() {
        return None;
    }
    // NIL
    if buf.get(i..i + 3)? == b"NIL" {
        return Some((None, i + 3));
    }
    // Quoted
    if buf[i] == b'"' {
        let (s, n) = parse_quoted(buf, i + 1)?;
        return Some((Some(s), n));
    }
    // Literal
    if buf[i] == b'{' {
        let (s, n) = parse_literal(buf, i)?;
        return Some((Some(s), n));
    }
    None
}

fn parse_quoted(buf: &[u8], mut i: usize) -> Option<(String, usize)> {
    let mut out = String::new();
    let mut escaped = false;
    while i < buf.len() {
        let b = buf[i];
        if escaped {
            out.push(b as char);
            escaped = false;
            i += 1;
            continue;
        }
        match b {
            b'\\' => {
                escaped = true;
                i += 1;
            }
            b'"' => {
                return Some((out, i + 1));
            }
            _ => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    None
}

fn parse_literal(buf: &[u8], mut i: usize) -> Option<(String, usize)> {
    // Expect {digits}\r\n content
    if buf[i] != b'{' {
        return None;
    }
    i += 1;
    let start = i;
    while i < buf.len() && buf[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    let n: usize = std::str::from_utf8(&buf[start..i]).ok()?.parse().ok()?;
    if buf.get(i) != Some(&b'}') {
        return None;
    }
    if buf.get(i + 1) != Some(&b'\r') || buf.get(i + 2) != Some(&b'\n') {
        return None;
    }
    let content_start = i + 3;
    let content_end = content_start + n;
    if content_end > buf.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&buf[content_start..content_end]).into_owned();
    Some((s, content_end))
}

fn skip_ws(buf: &[u8], i: &mut usize) {
    while *i < buf.len() && buf[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn find_subsequence(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn parse_number(buf: &[u8], mut i: usize) -> Option<(u32, usize)> {
    skip_ws(buf, &mut i);
    let start = i;
    while i < buf.len() && buf[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    let n: u32 = std::str::from_utf8(&buf[start..i]).ok()?.parse().ok()?;
    Some((n, i))
}
