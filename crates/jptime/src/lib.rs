//! Japanese readings of the time and date: kanji, kana (hiragana) and romaji.
//!
//! The digital clock shows 24-hour time (as in Japan); the *spoken* reading
//! still uses 午前/午後 (gozen/gogo), which is how a time is said aloud.
//!
//! Correctness is the point of a learning aid, so the irregular readings are
//! encoded explicitly and tested: hours 4/7/9 (よじ/しちじ/くじ), the 分 sound
//! changes (いっぷん… ろっぷん… じゅっぷん), months 4/7/9 (しがつ/しちがつ/くがつ),
//! the special days (ついたち…とおか, じゅうよっか, はつか, にじゅうよっか), the
//! day-counter 7/9 (しち/く) and the year euphony (4→よ, 9→く).
//!
//! Ported from `clock-mod` (tools/voicepack/voicepack/jptime.py, src/core/jptime.c).

#![forbid(unsafe_code)]

/// A reading in three forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// How it is written, e.g. `午後3時15分`, `2026年10月27日 火曜日`.
    pub kanji: String,
    /// How it is read, in hiragana, e.g. `ごごさんじじゅうごふん`.
    pub kana: String,
    /// Hepburn romaji of the reading, e.g. `gogo san-ji juugo-fun`.
    pub romaji: String,
}

const ONES_KANA: [&str; 10] = ["", "いち", "に", "さん", "よん", "ご", "ろく", "なな", "はち", "きゅう"];
const ONES_ROMAJI: [&str; 10] = ["", "ichi", "ni", "san", "yon", "go", "roku", "nana", "hachi", "kyuu"];
const HUND_KANA: [&str; 10] =
    ["", "ひゃく", "にひゃく", "さんびゃく", "よんひゃく", "ごひゃく", "ろっぴゃく", "ななひゃく", "はっぴゃく", "きゅうひゃく"];
const HUND_ROMAJI: [&str; 10] =
    ["", "hyaku", "nihyaku", "sanbyaku", "yonhyaku", "gohyaku", "roppyaku", "nanahyaku", "happyaku", "kyuuhyaku"];
const THOU_KANA: [&str; 10] =
    ["", "せん", "にせん", "さんぜん", "よんせん", "ごせん", "ろくせん", "ななせん", "はっせん", "きゅうせん"];
const THOU_ROMAJI: [&str; 10] =
    ["", "sen", "nisen", "sanzen", "yonsen", "gosen", "rokusen", "nanasen", "hassen", "kyuusen"];

fn number(n: i32, ones: &[&str; 10], hund: &[&str; 10], thou: &[&str; 10], juu: &str) -> String {
    if n <= 0 || n >= 10000 {
        return String::new();
    }
    let mut s = String::new();
    let mut v = n;
    s.push_str(thou[(v / 1000) as usize]);
    v %= 1000;
    s.push_str(hund[(v / 100) as usize]);
    v %= 100;
    if v >= 10 {
        if v / 10 == 1 {
            s.push_str(juu);
        } else {
            s.push_str(ones[(v / 10) as usize]);
            s.push_str(juu);
        }
        v %= 10;
    }
    s.push_str(ones[v as usize]);
    s
}

const HOUR_KANA: [&str; 13] = [
    "", "いちじ", "にじ", "さんじ", "よじ", "ごじ", "ろくじ", "しちじ", "はちじ", "くじ", "じゅうじ", "じゅういちじ", "じゅうにじ",
];
const HOUR_ROMAJI: [&str; 13] = [
    "", "ichi-ji", "ni-ji", "san-ji", "yo-ji", "go-ji", "roku-ji", "shichi-ji", "hachi-ji", "ku-ji", "juu-ji",
    "juuichi-ji", "juuni-ji",
];
const MIN_ONES_KANA: [&str; 10] =
    ["", "いっぷん", "にふん", "さんぷん", "よんぷん", "ごふん", "ろっぷん", "ななふん", "はっぷん", "きゅうふん"];
const MIN_ONES_ROMAJI: [&str; 10] =
    ["", "ippun", "nifun", "sanpun", "yonpun", "gofun", "roppun", "nanafun", "happun", "kyuufun"];
const MIN_TENS_KANA: [&str; 6] = ["", "じゅう", "にじゅう", "さんじゅう", "よんじゅう", "ごじゅう"];
const MIN_TENS_ROMAJI: [&str; 6] = ["", "juu", "nijuu", "sanjuu", "yonjuu", "gojuu"];
const MIN_TEN0_KANA: [&str; 6] = ["", "じゅっぷん", "にじゅっぷん", "さんじゅっぷん", "よんじゅっぷん", "ごじゅっぷん"];
const MIN_TEN0_ROMAJI: [&str; 6] = ["", "juppun", "nijuppun", "sanjuppun", "yonjuppun", "gojuppun"];

/// Reading of a 24-hour time. `:30` is read as 半 (han).
pub fn time_reading(h24: u8, minute: u8) -> Reading {
    assert!(h24 < 24 && minute < 60, "time out of range");
    let (period_k, period_r) = if h24 < 12 { ("午前", "gozen") } else { ("午後", "gogo") };
    let period_kana = if h24 < 12 { "ごぜん" } else { "ごご" };
    let h12 = if h24 % 12 == 0 { 12 } else { (h24 % 12) as usize };

    if minute == 30 {
        return Reading {
            kanji: format!("{period_k}{h12}時半"),
            kana: format!("{period_kana}{}はん", HOUR_KANA[h12]),
            romaji: format!("{period_r} {} han", HOUR_ROMAJI[h12]),
        };
    }
    let (min_kana, min_romaji) = minute_reading(minute);
    let min_kanji = if minute == 0 { String::new() } else { format!("{minute}分") };
    let sep = if min_romaji.is_empty() { "" } else { " " };
    Reading {
        kanji: format!("{period_k}{h12}時{min_kanji}"),
        kana: format!("{period_kana}{}{min_kana}", HOUR_KANA[h12]),
        romaji: format!("{period_r} {}{sep}{min_romaji}", HOUR_ROMAJI[h12]),
    }
}

fn minute_reading(m: u8) -> (String, String) {
    if m == 0 {
        return (String::new(), String::new());
    }
    let (tens, ones) = ((m / 10) as usize, (m % 10) as usize);
    if ones == 0 {
        (MIN_TEN0_KANA[tens].into(), MIN_TEN0_ROMAJI[tens].into())
    } else {
        (
            format!("{}{}", MIN_TENS_KANA[tens], MIN_ONES_KANA[ones]),
            format!("{}{}", MIN_TENS_ROMAJI[tens], MIN_ONES_ROMAJI[ones]),
        )
    }
}

const MONTH_KANA: [&str; 13] = [
    "", "いちがつ", "にがつ", "さんがつ", "しがつ", "ごがつ", "ろくがつ", "しちがつ", "はちがつ", "くがつ", "じゅうがつ",
    "じゅういちがつ", "じゅうにがつ",
];
const MONTH_ROMAJI: [&str; 13] = [
    "", "ichigatsu", "nigatsu", "sangatsu", "shigatsu", "gogatsu", "rokugatsu", "shichigatsu", "hachigatsu",
    "kugatsu", "juugatsu", "juuichigatsu", "juunigatsu",
];
const WEEKDAY: [(&str, &str, &str); 7] = [
    ("日曜日", "にちようび", "nichiyoubi"),
    ("月曜日", "げつようび", "getsuyoubi"),
    ("火曜日", "かようび", "kayoubi"),
    ("水曜日", "すいようび", "suiyoubi"),
    ("木曜日", "もくようび", "mokuyoubi"),
    ("金曜日", "きんようび", "kinyoubi"),
    ("土曜日", "どようび", "doyoubi"),
];

/// Sakamoto's algorithm; 0 = Sunday.
pub fn weekday_sun0(y: i32, m: u8, d: u8) -> usize {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = y;
    if m < 3 {
        y -= 1;
    }
    let w = (y + y / 4 - y / 100 + y / 400 + T[(m - 1) as usize] + d as i32) % 7;
    (if w < 0 { w + 7 } else { w }) as usize
}

fn day_reading(d: u8) -> (String, String) {
    const SPECIAL: [(u8, &str, &str); 13] = [
        (1, "ついたち", "tsuitachi"), (2, "ふつか", "futsuka"), (3, "みっか", "mikka"), (4, "よっか", "yokka"),
        (5, "いつか", "itsuka"), (6, "むいか", "muika"), (7, "なのか", "nanoka"), (8, "ようか", "youka"),
        (9, "ここのか", "kokonoka"), (10, "とおか", "tooka"), (14, "じゅうよっか", "juuyokka"),
        (20, "はつか", "hatsuka"), (24, "にじゅうよっか", "nijuuyokka"),
    ];
    if let Some(&(_, k, r)) = SPECIAL.iter().find(|&&(n, _, _)| n == d) {
        return (k.into(), r.into());
    }
    // day-counter ones: 7 -> しち, 9 -> く (unlike the plain number)
    const DAY_ONES_KANA: [&str; 10] = ["", "いち", "に", "さん", "", "ご", "ろく", "しち", "はち", "く"];
    const DAY_ONES_ROMAJI: [&str; 10] = ["", "ichi", "ni", "san", "", "go", "roku", "shichi", "hachi", "ku"];
    const DAY_TENS_KANA: [&str; 4] = ["", "じゅう", "にじゅう", "さんじゅう"];
    const DAY_TENS_ROMAJI: [&str; 4] = ["", "juu", "nijuu", "sanjuu"];
    let (t, o) = ((d / 10) as usize, (d % 10) as usize);
    (
        format!("{}{}にち", DAY_TENS_KANA[t], DAY_ONES_KANA[o]),
        format!("{}{}nichi", DAY_TENS_ROMAJI[t], DAY_ONES_ROMAJI[o]),
    )
}

fn year_reading(y: i32) -> (String, String) {
    let mut kana = number(y, &ONES_KANA, &HUND_KANA, &THOU_KANA, "じゅう");
    let mut romaji = number(y, &ONES_ROMAJI, &HUND_ROMAJI, &THOU_ROMAJI, "juu");
    // euphony before 年: final 4 -> よ/yo, final 9 -> く/ku
    if let Some(stripped) = kana.strip_suffix("よん") {
        kana = format!("{stripped}よ");
    } else if let Some(stripped) = kana.strip_suffix("きゅう") {
        kana = format!("{stripped}く");
    }
    if let Some(stripped) = romaji.strip_suffix("yon") {
        romaji = format!("{stripped}yo");
    } else if let Some(stripped) = romaji.strip_suffix("kyuu") {
        romaji = format!("{stripped}ku");
    }
    (format!("{kana}ねん"), format!("{romaji}nen"))
}

/// Reading of a date, including the day of the week.
pub fn date_reading(y: i32, m: u8, d: u8) -> Reading {
    assert!((1..=12).contains(&m) && (1..=31).contains(&d), "date out of range");
    let (wk_k, wk_kana, wk_r) = WEEKDAY[weekday_sun0(y, m, d)];
    let (year_k, year_r) = year_reading(y);
    let (day_k, day_r) = day_reading(d);
    Reading {
        kanji: format!("{y}年{m}月{d}日 {wk_k}"),
        kana: format!("{year_k}{}{day_k}{wk_kana}", MONTH_KANA[m as usize]),
        romaji: format!("{year_r} {} {day_r} {wk_r}", MONTH_ROMAJI[m as usize]),
    }
}

/// English weekday name (for the big top line, like the stock clock).
pub fn weekday_en(y: i32, m: u8, d: u8) -> &'static str {
    ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][weekday_sun0(y, m, d)]
}

/// Part of day for the subtitle line (Morning / Afternoon / Evening / Night).
pub fn part_of_day(h24: u8) -> &'static str {
    match h24 {
        5..=11 => "Morning",
        12..=16 => "Afternoon",
        17..=20 => "Evening",
        _ => "Night",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_irregulars() {
        assert_eq!(time_reading(15, 15).kana, "ごごさんじじゅうごふん");
        assert_eq!(time_reading(4, 0).kana, "ごぜんよじ"); // 4 = よじ
        assert_eq!(time_reading(19, 0).kana, "ごごしちじ"); // 7 = しちじ
        assert_eq!(time_reading(9, 0).kana, "ごぜんくじ"); // 9 = くじ
        assert_eq!(time_reading(10, 1).kana, "ごぜんじゅうじいっぷん");
        assert_eq!(time_reading(10, 6).kana, "ごぜんじゅうじろっぷん");
        assert_eq!(time_reading(10, 10).kana, "ごぜんじゅうじじゅっぷん");
        assert_eq!(time_reading(15, 30).kana, "ごごさんじはん");
        assert_eq!(time_reading(15, 15).romaji, "gogo san-ji juugofun");
        assert_eq!(time_reading(0, 0).kanji, "午前12時");
    }

    #[test]
    fn date_irregulars() {
        let r = date_reading(2026, 10, 27); // Tuesday
        assert_eq!(r.kanji, "2026年10月27日 火曜日");
        assert_eq!(r.kana, "にせんにじゅうろくねんじゅうがつにじゅうしちにちかようび");
        assert_eq!(r.romaji, "nisennijuurokunen juugatsu nijuushichinichi kayoubi");
        assert!(date_reading(2026, 4, 1).kana.contains("しがつついたち")); // April, 1st
        assert!(date_reading(2026, 9, 20).kana.contains("くがつはつか")); // Sep, 20th
        assert!(date_reading(2024, 2, 29).romaji.contains("nisennijuuyonen nigatsu nijuukunichi"));
    }

    #[test]
    fn helpers() {
        assert_eq!(weekday_en(2026, 10, 27), "Tuesday");
        assert_eq!(part_of_day(10), "Morning");
        assert_eq!(part_of_day(15), "Afternoon");
    }
}
