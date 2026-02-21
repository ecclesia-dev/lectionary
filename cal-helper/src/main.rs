// lectionary-cal: compute Divinum Officium Missa key for a given date
// Outputs: TEMPORA_KEY\tSANCTI_KEY\tTITLE
// The shell script uses these to look up Mass propers from DO data files.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let date = if args.len() > 1 {
        NaiveDate::parse_from_str(&args[1], "%Y-%m-%d")
            .expect("Usage: lectionary-cal [YYYY-MM-DD]")
    } else {
        chrono::Local::now().date_naive()
    };

    let (tempora_key, sancti_key, title, color, source) = resolve_day(date);
    println!("{}\t{}\t{}\t{}\t{}", tempora_key, sancti_key, title, color, source);
}

fn easter(year: i32) -> NaiveDate {
    // Anonymous Gregorian algorithm
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap()
}

fn resolve_day(date: NaiveDate) -> (String, String, String, String, String) {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    let dow = date.weekday(); // Mon=0 in chrono, Sun=6
    let sancti_key = format!("{:02}-{:02}", month, day);
    
    let easter_date = easter(year);
    let days_from_easter = (date - easter_date).num_days();

    // Moveable dates
    let septuagesima = easter_date - Duration::days(63);
    let ash_wed = easter_date - Duration::days(46);
    let passion_sun = easter_date - Duration::days(14);
    let palm_sun = easter_date - Duration::days(7);
    let ascension = easter_date + Duration::days(39);
    let pentecost = easter_date + Duration::days(49);
    let corpus_christi = easter_date + Duration::days(60);
    let sacred_heart = easter_date + Duration::days(68);
    
    // Advent 1: Sunday nearest Nov 30 (on or before Dec 3)
    let dec25 = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
    let advent1 = {
        let mut d = dec25 - Duration::days(28);
        while d.weekday() != Weekday::Sun { d += Duration::days(1); }
        d
    };

    let epiphany = NaiveDate::from_ymd_opt(year, 1, 6).unwrap();

    // DOW number for DO key: 0=Sun, 1=Mon ... 6=Sat
    let dow_num = match dow {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    };

    // Check special fixed feasts first (Class I/II that override Tempora)
    let special = check_special_feast(date, &sancti_key, easter_date);
    if let Some((title, color)) = special {
        let tempora = compute_tempora_key(date, year, easter_date, epiphany, septuagesima,
            ash_wed, passion_sun, palm_sun, ascension, pentecost, advent1, dow_num);
        return (tempora, sancti_key, title, color, "sancti".into());
    }

    // Compute Tempora key
    let tempora = compute_tempora_key(date, year, easter_date, epiphany, septuagesima,
        ash_wed, passion_sun, palm_sun, ascension, pentecost, advent1, dow_num);
    
    let title = compute_title(date, year, easter_date, epiphany, septuagesima,
        ash_wed, passion_sun, palm_sun, ascension, pentecost, corpus_christi, 
        sacred_heart, advent1, dow_num);
    
    let color = compute_color(date, year, easter_date, epiphany, septuagesima,
        ash_wed, passion_sun, palm_sun, ascension, pentecost, advent1, dow_num);
    
    (tempora, sancti_key, title, color, "tempora".into())
}

fn compute_tempora_key(
    date: NaiveDate, year: i32, easter: NaiveDate, epiphany: NaiveDate,
    septuagesima: NaiveDate, ash_wed: NaiveDate, passion_sun: NaiveDate,
    palm_sun: NaiveDate, ascension: NaiveDate, pentecost: NaiveDate,
    advent1: NaiveDate, dow: i32,
) -> String {
    let days_from_easter = (date - easter).num_days();
    
    // Christmas octave (Dec 25 - Jan 1)
    let dec25_prev = NaiveDate::from_ymd_opt(year - 1, 12, 25).unwrap();
    let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    
    if date <= jan1 && date >= dec25_prev {
        let day_in_oct = (date - dec25_prev).num_days();
        return format!("Nat1-0{}", match day_in_oct { 0 => "".into(), n => format!("{}", n) });
    }
    
    // Jan 1 (Circumcision)
    if date == jan1 {
        return "Nat1-0".into();
    }
    
    // Jan 2-5: days within Christmas octave
    if date.month() == 1 && date.day() >= 2 && date.day() <= 5 && date < epiphany {
        return format!("Nat2-{}", dow);
    }
    
    // Epiphany (Jan 6)
    if date == epiphany {
        // Epiphany is in Sancti/01-06.txt for the Missa
        return "Epi1-0a".into();
    }
    
    // After Epiphany
    if date > epiphany && date < septuagesima {
        // Find first Sunday after Epiphany (>= Jan 7)
        let mut first_sun = epiphany + Duration::days(1);
        while first_sun.weekday() != Weekday::Sun { first_sun += Duration::days(1); }
        
        if date < first_sun {
            // Days between Epiphany and first Sunday
            return format!("Epi1-{}", dow);
        }
        
        // Week numbering: first_sun = Epi1-0 (Holy Family / 1st Sun after Epi)
        // Following Sun = Epi2-0, etc.
        let weeks_since = ((date - first_sun).num_days() / 7) + 1;
        return format!("Epi{}-{}", weeks_since, dow);
    }
    
    // Septuagesima
    if date >= septuagesima && date < ash_wed {
        let weeks = ((date - septuagesima).num_days() / 7) + 1;
        let name = match weeks {
            1 => "Quadp1",
            2 => "Quadp2", 
            3 => "Quadp3",
            _ => "Quadp3",
        };
        return format!("{}-{}", name, dow);
    }
    
    // Lent (Ash Wednesday to Passion Sunday)
    if date >= ash_wed && date < passion_sun {
        let days = (date - ash_wed).num_days();
        if days < 4 {
            // Ash Wed=Quadp3-3, Thu=Quadp3-4, Fri=Quadp3-5, Sat=Quadp3-6
            return format!("Quadp3-{}", days + 3);
        }
        let first_sun_lent = ash_wed + Duration::days(4);
        let weeks = ((date - first_sun_lent).num_days() / 7) + 1;
        return format!("Quad{}-{}", weeks, dow);
    }
    
    // Passiontide (Passion Sunday to Palm Sunday)
    if date >= passion_sun && date < palm_sun {
        let days = (date - passion_sun).num_days();
        let dow_p = (days % 7) as i32;
        return format!("Quad5-{}", dow_p);
    }
    
    // Holy Week
    if date >= palm_sun && date < easter {
        let days = (date - palm_sun).num_days();
        return format!("Quad6-{}", days);
    }
    
    // Easter Week
    if days_from_easter >= 0 && days_from_easter <= 6 {
        return format!("Pasc0-{}", days_from_easter);
    }
    
    // Easter season (weeks after Easter, up to and including Pentecost week)
    if days_from_easter > 6 && days_from_easter <= 55 {
        let week = (days_from_easter / 7) as i32;
        let d = (days_from_easter % 7) as i32;
        return format!("Pasc{}-{}", week, d);
    }
    
    // Pentecost week (octave)
    if date > pentecost && date < pentecost + Duration::days(7) {
        let d = (date - pentecost).num_days() as i32;
        return format!("Pasc7-{}", d);
    }
    
    // After Pentecost
    if date >= pentecost + Duration::days(7) && date < advent1 {
        let weeks_after_pent = ((date - pentecost).num_days() / 7) as i32;
        // Trinity Sunday is 1st Sunday after Pentecost = Pent01-0
        return format!("Pent{:02}-{}", weeks_after_pent, dow);
    }
    
    // Advent
    if date >= advent1 {
        let weeks = ((date - advent1).num_days() / 7) + 1;
        return format!("Adv{}-{}", weeks, dow);
    }
    
    // Fallback
    format!("unknown-{}", date)
}

fn check_special_feast(date: NaiveDate, sancti_key: &str, _easter: NaiveDate) -> Option<(String, String)> {
    // Major fixed feasts that have their own Missa propers in Sancti/
    match sancti_key {
        "01-01" => Some(("Circumcision of Our Lord".into(), "white".into())),
        "01-06" => Some(("Epiphany of Our Lord".into(), "white".into())),
        "02-02" => Some(("Purification of the B.V.M.".into(), "white".into())),
        "03-19" => Some(("St. Joseph, Spouse of the B.V.M.".into(), "white".into())),
        "03-25" => Some(("Annunciation of the B.V.M.".into(), "white".into())),
        "06-24" => Some(("Nativity of St. John the Baptist".into(), "white".into())),
        "06-29" => Some(("Sts. Peter and Paul".into(), "red".into())),
        "08-06" => Some(("Transfiguration of Our Lord".into(), "white".into())),
        "08-15" => Some(("Assumption of the B.V.M.".into(), "white".into())),
        "09-08" => Some(("Nativity of the B.V.M.".into(), "white".into())),
        "09-14" => Some(("Exaltation of the Holy Cross".into(), "red".into())),
        "09-29" => Some(("Dedication of St. Michael".into(), "white".into())),
        "11-01" => Some(("All Saints".into(), "white".into())),
        "11-02" => Some(("All Souls".into(), "black".into())),
        "12-08" => Some(("Immaculate Conception".into(), "white".into())),
        "12-25" => Some(("Nativity of Our Lord".into(), "white".into())),
        _ => None,
    }
}

fn compute_title(
    date: NaiveDate, year: i32, easter: NaiveDate, epiphany: NaiveDate,
    septuagesima: NaiveDate, ash_wed: NaiveDate, passion_sun: NaiveDate,
    palm_sun: NaiveDate, ascension: NaiveDate, pentecost: NaiveDate,
    corpus_christi: NaiveDate, sacred_heart: NaiveDate, advent1: NaiveDate, dow: i32,
) -> String {
    let days_from_easter = (date - easter).num_days();
    let is_sun = dow == 0;
    
    // Special moveable feasts
    if date == ascension { return "Ascension of Our Lord".into(); }
    if date == pentecost { return "Pentecost Sunday".into(); }
    if date == pentecost + Duration::days(7) && is_sun { return "Trinity Sunday".into(); }
    if date == corpus_christi { return "Corpus Christi".into(); }
    if date == sacred_heart { return "Sacred Heart of Jesus".into(); }
    if date == ash_wed { return "Ash Wednesday".into(); }
    if date == palm_sun { return "Palm Sunday".into(); }
    if date == easter - Duration::days(3) { return "Holy Thursday".into(); }
    if date == easter - Duration::days(2) { return "Good Friday".into(); }
    if date == easter - Duration::days(1) { return "Holy Saturday".into(); }
    if date == easter { return "Easter Sunday".into(); }
    if days_from_easter > 0 && days_from_easter < 7 {
        let day = ["", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        return format!("{} in Easter Week", day[days_from_easter as usize]);
    }
    
    // After Epiphany
    if date > epiphany && date < septuagesima {
        let mut first_sun = epiphany;
        while first_sun.weekday() != Weekday::Sun { first_sun += Duration::days(1); }
        let weeks = if date >= first_sun { ((date - first_sun).num_days() / 7) + 1 } else { 1 };
        if is_sun {
            return format!("{} Sunday after Epiphany", ordinal(weeks as u32));
        }
        return format!("Feria of {} week after Epiphany", ordinal(weeks as u32));
    }
    
    // Septuagesima
    if date >= septuagesima && date < ash_wed {
        let weeks = ((date - septuagesima).num_days() / 7) + 1;
        let name = match weeks {
            1 => "Septuagesima",
            2 => "Sexagesima",
            3 => "Quinquagesima",
            _ => "Quinquagesima",
        };
        if is_sun { return format!("{} Sunday", name); }
        return format!("Feria of {} week", name);
    }
    
    // Lent
    if date > ash_wed && date < passion_sun {
        let first_sun_lent = ash_wed + Duration::days(4);
        if date < first_sun_lent {
            return "Feria after Ash Wednesday".into();
        }
        let weeks = ((date - first_sun_lent).num_days() / 7) + 1;
        if is_sun { return format!("{} Sunday of Lent", ordinal(weeks as u32)); }
        return format!("Feria of {} week of Lent", ordinal(weeks as u32));
    }
    
    // Passiontide
    if date >= passion_sun && date < palm_sun {
        if is_sun { return "Passion Sunday".into(); }
        return "Feria of Passion week".into();
    }
    
    // Holy Week
    if date > palm_sun && date < easter - Duration::days(3) {
        let day = ["", "Monday", "Tuesday", "Wednesday"];
        let d = (date - palm_sun).num_days();
        if d <= 3 { return format!("{} of Holy Week", day[d as usize]); }
    }
    
    // Easter season (after octave)
    if days_from_easter >= 7 && days_from_easter < 49 {
        let week = days_from_easter / 7;
        if is_sun { return format!("{} Sunday after Easter", ordinal(week as u32)); }
        return format!("Feria of {} week after Easter", ordinal(week as u32));
    }
    
    // After Pentecost
    if date > pentecost && date < advent1 {
        let weeks = (date - pentecost).num_days() / 7;
        if is_sun { return format!("{} Sunday after Pentecost", ordinal(weeks as u32)); }
        return format!("Feria of {} week after Pentecost", ordinal(weeks as u32));
    }
    
    // Advent
    if date >= advent1 && date.month() == 12 && date.day() < 25 {
        let weeks = ((date - advent1).num_days() / 7) + 1;
        if is_sun { return format!("{} Sunday of Advent", ordinal(weeks as u32)); }
        return format!("Feria of {} week of Advent", ordinal(weeks as u32));
    }
    
    // Christmas octave
    if date.month() == 12 && date.day() >= 25 {
        if date.day() == 25 { return "Christmas Day".into(); }
        return format!("Day {} of Christmas Octave", date.day() - 25 + 1);
    }
    if date.month() == 1 && date.day() <= 5 {
        return format!("Day within Christmas Octave (Jan {})", date.day());
    }
    
    format!("{}", date.format("%B %d"))
}

fn compute_color(
    date: NaiveDate, year: i32, easter: NaiveDate, epiphany: NaiveDate,
    septuagesima: NaiveDate, ash_wed: NaiveDate, passion_sun: NaiveDate,
    palm_sun: NaiveDate, ascension: NaiveDate, pentecost: NaiveDate,
    advent1: NaiveDate, _dow: i32,
) -> String {
    let days_from_easter = (date - easter).num_days();
    
    // Easter/Christmas seasons: white
    if (days_from_easter >= 0 && days_from_easter <= 55) { return "white".into(); }
    if date.month() == 12 && date.day() >= 25 { return "white".into(); }
    if date.month() == 1 && date.day() <= 13 { return "white".into(); }
    
    // Pentecost day: red
    if date == pentecost { return "red".into(); }
    
    // Lent, Advent, Septuagesima: violet
    if date >= septuagesima && date < easter { return "violet".into(); }
    if date >= advent1 { return "violet".into(); }
    
    // After Pentecost: green
    "green".into()
}

fn ordinal(n: u32) -> String {
    match n {
        1 => "1st".into(), 2 => "2nd".into(), 3 => "3rd".into(),
        _ => format!("{}th", n),
    }
}
