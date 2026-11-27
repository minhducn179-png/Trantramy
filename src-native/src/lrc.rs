use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

static LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)\]").unwrap());
static WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[<(](\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)[>)]").unwrap());
static META: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\s*\[[a-z]+:.*\]\s*$").unwrap());
static OFFSET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?im)^\s*\[offset:\s*([+-]?\d+)\s*\]\s*$").unwrap());
static TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?\]|[<(]\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?[>)]")
        .unwrap()
});
#[derive(Clone, Debug, Default)]
pub struct Word {
    pub text: String,
    pub start: f64,
    pub end: f64,
}
#[derive(Clone, Debug, Default)]
pub struct Line {
    pub time: f64,
    pub end: f64,
    pub text: String,
    pub words: Vec<Word>,
    pub word_sync: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub synced: bool,
    pub word_sync: bool,
    pub lines: Vec<Line>,
}
fn timestamp(min: &str, sec: &str) -> f64 {
    min.parse::<f64>().unwrap_or(0.0) * 60.0 + sec.replace(':', ".").parse::<f64>().unwrap_or(0.0)
}
pub fn offset(raw: &str) -> i64 {
    OFFSET
        .captures_iter(raw)
        .last()
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0)
}
pub fn set_offset(raw: &str, ms: i64) -> Result<String, String> {
    if !(-10000..=10000).contains(&ms) {
        return Err("Bù thời gian phải từ -10000 đến 10000 ms.".into());
    }
    let rows = raw
        .lines()
        .filter(|s| !OFFSET.is_match(s))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(if ms == 0 {
        rows
    } else {
        format!("[offset:{ms}]\n{rows}")
    })
}
pub fn parse(raw: &str) -> Parsed {
    let (mut timed, mut plain) = (Vec::new(), Vec::new());
    for row in raw
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !META.is_match(s))
    {
        let stamps: Vec<_> = LINE.captures_iter(row).collect();
        if stamps.is_empty() {
            plain.push(Line {
                text: row.into(),
                time: -1.0,
                end: -1.0,
                ..Default::default()
            });
            continue;
        }
        let remainder = LINE.replace_all(row, "");
        for stamp in stamps {
            let time = timestamp(&stamp[1], &stamp[2]);
            let tags: Vec<_> = WORD.captures_iter(&remainder).collect();
            let mut words = Vec::new();
            if let Some(first) = tags.first() {
                let leading = remainder[..first.get(0).unwrap().start()].trim();
                if !leading.is_empty() {
                    words.push(Word {
                        text: leading.into(),
                        start: time,
                        end: timestamp(&first[1], &first[2]).max(time),
                    });
                }
            }
            for (i, tag) in tags.iter().enumerate() {
                let start = timestamp(&tag[1], &tag[2]);
                let after = tag.get(0).unwrap().end();
                let until = tags
                    .get(i + 1)
                    .map_or(remainder.len(), |next| next.get(0).unwrap().start());
                let text = remainder[after..until].trim();
                if text.is_empty() {
                    continue;
                }
                let end = tags
                    .get(i + 1)
                    .map_or(start + (text.chars().count() as f64 * 0.12).max(0.4), |n| {
                        timestamp(&n[1], &n[2])
                    });
                let parts: Vec<_> = text.split_whitespace().collect();
                let span = (end - start).max(0.0) / parts.len() as f64;
                for (k, text) in parts.into_iter().enumerate() {
                    words.push(Word {
                        text: text.into(),
                        start: start + k as f64 * span,
                        end: start + (k + 1) as f64 * span,
                    });
                }
            }
            let word_sync = !words.is_empty();
            let text = if word_sync {
                words
                    .iter()
                    .map(|w| w.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                remainder.trim().into()
            };
            let end = words.last().map_or(time + 2.0, |w| w.end);
            timed.push(Line {
                time,
                end,
                text,
                words,
                word_sync,
            });
        }
    }
    let shift = offset(raw) as f64 / 1000.0;
    for line in &mut timed {
        line.time = (line.time - shift).max(0.0);
        line.end = (line.end - shift).max(line.time);
        for word in &mut line.words {
            word.start = (word.start - shift).max(0.0);
            word.end = (word.end - shift).max(word.start);
        }
    }
    timed.sort_by(|a, b| a.time.total_cmp(&b.time));
    let synced = !timed.is_empty();
    let word_sync = timed.iter().any(|l| l.word_sync);
    Parsed {
        synced,
        word_sync,
        lines: if synced { timed } else { plain },
    }
}
pub fn active(lines: &[Line], position: f64) -> Option<usize> {
    lines
        .iter()
        .rposition(|l| l.time >= 0.0 && l.time <= position)
}
pub fn correct_text(original: &str, corrected: &str) -> Result<String, String> {
    let replacements: Vec<_> = corrected
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let editable = original
        .lines()
        .filter(|s| !META.is_match(s) && !TAG.replace_all(s, "").trim().is_empty())
        .count();
    if replacements.len() != editable {
        return Err("Giữ nguyên số dòng để không lệch mốc thời gian.".into());
    }
    let mut next = replacements.into_iter();
    let mut rows = Vec::new();
    for row in original.lines() {
        let plain = TAG.replace_all(row, " ");
        if META.is_match(row) || plain.trim().is_empty() {
            rows.push(row.to_owned());
            continue;
        }
        let replacement = next.next().ok_or("Thiếu dòng lời.")?;
        if replacement.contains(['[', ']', '<', '>']) || WORD.is_match(replacement) {
            return Err("Chỉ nhập lời, không nhập timestamp.".into());
        }
        let words: Vec<_> = replacement.split_whitespace().collect();
        if words.len() != plain.split_whitespace().count() {
            return Err("Giữ nguyên số chữ. Dùng sửa LRC nếu cần thêm bớt.".into());
        }
        let mut result = String::new();
        let mut at = 0;
        let mut word = 0;
        for tag in TAG.find_iter(row) {
            replace_segment(&row[at..tag.start()], &words, &mut word, &mut result);
            result.push_str(tag.as_str());
            at = tag.end();
        }
        replace_segment(&row[at..], &words, &mut word, &mut result);
        rows.push(result);
    }
    Ok(rows.join("\n"))
}
fn replace_segment(s: &str, words: &[&str], index: &mut usize, out: &mut String) {
    for part in s.split_inclusive(char::is_whitespace) {
        let text = part.trim_end_matches(char::is_whitespace);
        if !text.is_empty() {
            if let Some(w) = words.get(*index) {
                out.push_str(w);
            }
            *index += 1;
        }
        out.push_str(&part[text.len()..]);
    }
}
fn normalize(word: &str) -> String {
    word.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c == 'đ' { 'd' } else { c })
        .filter(|c| c.is_alphanumeric())
        .collect()
}
pub fn align(reference: &str, transcript: &str, duration: f64) -> Result<String, String> {
    let source = parse(transcript);
    let target = parse(reference);
    let words: Vec<_> = source.lines.iter().flat_map(|l| &l.words).collect();
    let wanted: Vec<_> = target
        .lines
        .iter()
        .flat_map(|l| l.text.split_whitespace())
        .collect();
    if !source.word_sync
        || source.lines.iter().any(|l| !l.word_sync)
        || words.is_empty()
        || wanted.is_empty()
    {
        return Err("Cần bản nhận diện có mốc từng chữ từ Whisper, Groq hoặc OpenAI.".into());
    }
    if wanted.len() > 2000 || words.len() > 2400 || !duration.is_finite() || duration <= 0.0 {
        return Err("Lời quá dài hoặc chưa biết thời lượng.".into());
    }
    for (i, w) in words.iter().enumerate() {
        if w.start < 0.0
            || w.end <= w.start
            || w.end > duration + 0.1
            || (i > 0 && w.start < words[i - 1].end - 0.02)
        {
            return Err("Mốc giọng hát bị chồng hoặc vượt thời lượng.".into());
        }
    }
    let tokens: Vec<_> = words.iter().map(|w| normalize(&w.text)).collect();
    let expected: Vec<_> = wanted.iter().map(|s| normalize(s)).collect();
    if expected.iter().any(String::is_empty) {
        return Err("Bỏ ký hiệu không phải lời hát trước khi căn.".into());
    }
    let mut matched = Vec::new();
    let mut at = 0;
    for token in &expected {
        while at < tokens.len() && &tokens[at] != token {
            at += 1;
        }
        if at == tokens.len() {
            return Err("Lời và giọng hát chưa khớp đủ chữ. Chưa đổi mốc cũ.".into());
        }
        matched.push(at);
        at += 1;
    }
    let mut at = tokens.len();
    for (i, token) in expected.iter().enumerate().rev() {
        while at > 0 && &tokens[at - 1] != token {
            at -= 1;
        }
        if at == 0 || at - 1 != matched[i] {
            return Err("Chữ lặp chưa xác định được lần hát. Không tự gán mốc.".into());
        }
        at -= 1;
    }
    if (expected.len() as f64 / tokens.len() as f64) < 0.85 {
        return Err("Lời có sẵn thiếu nhiều chữ so với bản nhận diện.".into());
    }
    let mut i = 0;
    let mut rows = Vec::new();
    for line in target.lines {
        let parts: Vec<_> = line.text.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let mut row = format!("[{}]", crate::lyrics::stamp(words[matched[i]].start)?);
        for text in parts {
            let w = words[matched[i]];
            i += 1;
            row.push_str(&format!(
                "<{}>{text}<{}> ",
                crate::lyrics::stamp(w.start)?,
                crate::lyrics::stamp(w.end)?
            ));
        }
        rows.push(row.trim_end().to_owned());
    }
    Ok(rows.join("\n"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_vietnamese_word_ends_offsets_and_repeated_words() {
        let raw = "[offset:100]\n[00:01.00]<00:01.00>Xin<00:01.40> <00:01.60>chào<00:02.00>";
        let p = parse(raw);
        assert!(p.word_sync);
        assert_eq!(p.lines[0].text, "Xin chào");
        assert!((p.lines[0].words[0].end - 1.3).abs() < 0.001);
        assert_eq!(active(&p.lines, 0.5), None);
        let edited = correct_text(raw, "Xin Chào").unwrap();
        assert!(edited.contains("<00:01.60>Chào<00:02.00>"));
        assert!(correct_text(raw, "Xin chào bạn").is_err());
        assert!(
            align(
                "Xin",
                "[00:00.00]<00:00.00>Xin<00:01.00> <00:01.00>Xin<00:02.00>",
                3.0
            )
            .is_err()
        );
        let aligned = align("Xin chào", raw, 3.0).unwrap();
        assert_eq!(parse(&aligned).lines[0].text, "Xin chào");
        assert!(!parse("[00:01.00]Lời theo câu").word_sync);
        assert_eq!(offset(&set_offset(raw, -100).unwrap()), -100);
    }
}
