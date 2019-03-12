use failure::Error;
use hashbrown::HashMap;
use rayon::slice::ParallelSliceMut;
use signal_hook;
use std::{
    cmp::Ord,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::{
    clap::{_clap_count_exprs, arg_enum},
    StructOpt,
};

arg_enum! {
    #[derive(Debug)]
    pub enum SortingOrder {
        Key,
        Count,
        None,
    }
}

#[derive(StructOpt, Debug)]
pub struct Config {
    #[structopt(
        long = "sortby",
        short = "s",
        default_value = "Count",
        raw(
            possible_values = "&SortingOrder::variants()",
            case_insensitive = "true"
        )
    )]
    sort_by: SortingOrder,
    #[structopt(long = "top")]
    top: Option<usize>,
    #[structopt()]
    input: Option<String>,
}

pub fn run(config: Config) -> Result<(), Error> {
    let sig_pipe = watch_sig_pipe()?;

    let reader = create_reader(&config.input)?;

    let counter = count_items(reader)?;

    let mut counts: Vec<_> = counter.iter().collect();
    sort_counts(&mut counts, &config.sort_by);

    let n = config.top.unwrap_or_else(|| counts.len());

    let stdout = stdout();
    let handle = stdout.lock();

    output_counts(handle, counts, n, sig_pipe)?;

    Ok(())
}

fn create_reader(input: &Option<String>) -> Result<Box<BufRead>, Error> {
    let reader: Box<BufRead> = match input {
        Some(file_name) => Box::new(BufReader::new(File::open(file_name)?)),
        None => Box::new(BufReader::new(stdin())),
    };
    Ok(reader)
}

fn count_items(reader: Box<BufRead>) -> Result<HashMap<std::string::String, u64>, Error> {
    let mut counter: HashMap<_, u64> = Default::default();

    for line in reader.lines() {
        *counter.entry(line?).or_insert(0) += 1;
    }

    Ok(counter)
}

fn sort_counts<S: Ord + Sync, T: Ord + Sync>(
    counts: &mut Vec<(&S, &T)>,
    sorting_order: &SortingOrder,
) {
    match sorting_order {
        SortingOrder::Key => {
            counts.par_sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(b.1).reverse()))
        }
        SortingOrder::Count => {
            counts.par_sort_unstable_by(|a, b| a.1.cmp(b.1).reverse().then(a.0.cmp(b.0)))
        }
        SortingOrder::None => (),
    }
}

fn output_counts<T: Write>(
    mut io: T,
    counts: Vec<(&String, &u64)>,
    n: usize,
    sig_pipe: Arc<AtomicBool>,
) -> Result<(), Error> {
    for (key, count) in counts.iter().take(n) {
        writeln!(io, "{}\t{}", key, count)?;
        if sig_pipe.load(Ordering::Relaxed) {
            break;
        }
    }
    Ok(())
}

fn watch_sig_pipe() -> Result<Arc<AtomicBool>, Error> {
    let sig_pipe = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGPIPE, Arc::clone(&sig_pipe))?;
    Ok(sig_pipe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_counts_by_key() {
        let mut input = vec![(&"b", &3), (&"c", &2), (&"a", &1)];
        let output = vec![(&"a", &1), (&"b", &3), (&"c", &2)];

        sort_counts(&mut input, &SortingOrder::Key);

        assert_eq!(input, output);
    }

    #[test]
    fn test_sort_counts_by_counts() {
        let mut input = vec![(&"c", &2), (&"a", &1), (&"b", &3)];
        let output = vec![(&"b", &3), (&"c", &2), (&"a", &1)];

        sort_counts(&mut input, &SortingOrder::Count);

        assert_eq!(input, output);
    }

}
