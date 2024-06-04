pub const DATESTAMP_FORMAT: &'static str = "%Y-%m-%d";
pub const DATETIMESTAMP_FORMAT: &'static str = "%Y-%m-%d-%H-%M-%S-%3f";

pub fn datetimestamp_today() -> String {
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

