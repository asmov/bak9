use chrono::TimeZone;

pub const DATESTAMP_FORMAT: &'static str = "%Y-%m-%d";
pub const DATETIMESTAMP_FORMAT: &'static str = "%Y-%m-%d-%H-%M-%S-%3f";


pub fn datetimestamp(chrono: chrono::DateTime<chrono::Local>) -> String {
    chrono.format(&DATETIMESTAMP_FORMAT).to_string()
}

pub fn from_datetimestamp(datetimestamp: &str) -> chrono::DateTime<chrono::Local> {
    let datetime = chrono::NaiveDateTime::parse_from_str(datetimestamp, DATETIMESTAMP_FORMAT).unwrap();
    chrono::Local.from_local_datetime(&datetime).unwrap()
}
 
pub fn datetimestamp_now() -> String {
    chrono::Local::now()
        .format(&DATETIMESTAMP_FORMAT)
        .to_string()
}

pub fn datetimestamp_yesterday() -> String {
    let yesterday = chrono::Local::now() - chrono::Duration::days(1);
    yesterday
        .format(&DATETIMESTAMP_FORMAT)
        .to_string()
}

pub fn datetime_now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

pub fn datestamp_today() -> String {
    chrono::Local::now()
        .format(DATESTAMP_FORMAT)
        .to_string()
}

pub fn datestamp_yesterday() -> String {
    let yesterday = chrono::Local::now() - chrono::Duration::days(1);
    yesterday
        .format(DATESTAMP_FORMAT)
        .to_string()
}

