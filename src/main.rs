use std::env;
use std::time::{ SystemTime, UNIX_EPOCH };
use std::{ thread, time::Duration };
use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc };
use reqwest::blocking::Client;
use reqwest::header::{ CACHE_CONTROL, USER_AGENT };

fn main() {
    // parsing args
    let mut args = env::args().skip(1);
    let url = match args.next() {
        Some(u) => u,
        None => {
            eprintln!("usage: <url> [interval_in_seconds]");
            return;
        }
    };
    let interval: u64 = args
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2);

    if interval == 0 {
        eprintln!("error: interval must be an integer >= 1 (seconds)");
        return;
    }

    let client = Client::builder().build().expect("failed to build client");
    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        let stop_flag_clone = stop_flag.clone();
        ctrlc
            ::set_handler(move || {
                stop_flag_clone.store(true, Ordering::SeqCst);
            })
            .expect("failed to set ctrl+c handler");
    }

    println!(
        "\nstarting page refreshing for {} with interval {}s\n(ctrl+c to stop)\n",
        url,
        interval
    );

    // main loop
    while !stop_flag.load(Ordering::SeqCst) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        match
            client
                .get(&url)
                .header(
                    USER_AGENT,
                    "Mozilla/5.0 (Windows NT 6.2; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.40 Safari/537.36"
                )
                .header(CACHE_CONTROL, "no-cache")
                .send()
        {
            Ok(resp) => {
                let status = resp.status();
                println!("[{}] | {}", timestamp, status);
            }
            Err(e) => {
                eprintln!("[{}] | {}", timestamp, e);
            }
        }

        let mut slept = 0;
        while slept < interval && !stop_flag.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            slept += 1;
        }
    }
}
