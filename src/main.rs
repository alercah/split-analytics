fn main() -> Result<(), failure::Error> {
    let args = clap::App::new("sum of best")
        .arg(clap::Arg::with_name("file").required(true))
        .get_matches();

    let path = std::path::PathBuf::from(args.value_of("file").unwrap());
    let file = std::io::BufReader::new(std::fs::File::open(&path)?);
    let run = livesplit_core::run::parser::composite::parse(file, Some(path), false)?.run;

    let mut attempts: Vec<_> = run
        .attempt_history()
        .iter()
        .filter(|a| a.started().is_some())
        .collect();
    attempts.sort_by(|a, b| a.started().unwrap().time.cmp(&b.started().unwrap().time));

    let mut best_by_segment = std::collections::HashMap::new();
    for segment in run.segments() {
        let mut best_times: Vec<Option<livesplit_core::TimeSpan>> = vec![None];
        let history = segment.segment_history();
        for attempt in attempts.iter() {
            let time = history.get(attempt.index());
            let mut best = *best_times.last().unwrap();
            match time {
                Some(t) => {
                    if t.real_time.is_some() {
                        match best {
                            Some(b) => {
                                if t.real_time.unwrap() < b {
                                    best = Some(t.real_time.unwrap());
                                }
                            }
                            None => {
                                best = Some(t.real_time.unwrap());
                            }
                        }
                    }
                }
                None => {}
            }
            best_times.push(best);
        }
        best_times.remove(0);
        best_by_segment.insert(segment.name(), best_times);
        print!("{},", segment.name());
    }
    println!("Total,AttemptStart");

    'attempt: for i in 0..attempts.len() {
        let attempt = attempts[i];
        for segment in run.segments() {
            if best_by_segment[segment.name()][i].is_none() {
                continue 'attempt;
            }
        }

        let mut total = livesplit_core::TimeSpan::zero();
        for segment in run.segments() {
            let segment_time = best_by_segment[segment.name()][i].unwrap();
            total += segment_time;
            print!("{},", segment_time.total_milliseconds());
        }
        println!(
            "{},{}",
            total.total_milliseconds(),
            attempt.started().unwrap().time
        );
    }
    Ok(())
}
