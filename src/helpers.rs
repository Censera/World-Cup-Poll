use chrono;

pub fn log(msg: &str) {
    println!(
        "[{}] {}",
        chrono::Local::now().format("%Y/%m/%d %H:%M:%S"),
        msg
    );
}
