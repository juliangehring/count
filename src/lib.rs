use hashbrown::HashMap;
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use signal_hook;
use std::{
    cmp::Ord,
    error::Error,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::{clap::arg_enum, StructOpt};

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
        possible_values = &SortingOrder::variants(),
        case_insensitive = true
    )]
    sort_by: SortingOrder,
    #[structopt(long = "top")]
    top: Option<usize>,
    #[structopt()]
    input: Option<String>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let sig_pipe = watch_sig_pipe()?;

    let reader = create_reader(&config.input)?;

    let counter = count_items(reader)?;

    let mut counts: Vec<_> = counter.par_iter().collect();
    sort_counts(&mut counts, &config.sort_by);

    let n = config.top.unwrap_or_else(|| counts.len());

    let stdout = stdout();
    let handle = stdout.lock();

    output_counts(handle, counts, n, sig_pipe)?;

    Ok(())
}

fn create_reader(input: &Option<String>) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
    let reader: Box<dyn BufRead> = match input {
        Some(file_name) => Box::new(BufReader::new(File::open(file_name)?)),
        None => Box::new(BufReader::new(stdin())),
    };
    Ok(reader)
}

fn count_items(mut reader: Box<dyn BufRead>) -> Result<HashMap<Vec<u8>, u64>, Box<dyn Error>> {
    let mut counter: HashMap<_, u64> = Default::default();

    let mut buf = Vec::with_capacity(64);
    while let Ok(n) = reader.read_until(b'\n', &mut buf) {
        // trim trailing newline
        if n == 0 {
            break;
        } else if buf[n - 1] == b'\n' {
            let n_end = if n > 1 && buf[n - 2] == b'\r' {
                n - 2
            } else {
                n - 1
            };
            buf.truncate(n_end);
        }
        match counter.get_mut(&buf) {
            Some(count) => {
                *count += 1;
            }
            None => {
                counter.insert(buf.to_owned(), 1);
            }
        };
        buf.clear();
    }

    Ok(counter)
}

fn sort_counts<S: Ord + Sync, T: Ord + Sync>(
    counts: &mut Vec<(&S, &T)>,
    sorting_order: &SortingOrder,
) {
    match sorting_order {
        SortingOrder::Key => {
            counts.par_sort_unstable_by(|k, v| k.0.cmp(v.0).then(k.1.cmp(k.1).reverse()))
        }
        SortingOrder::Count => {
            counts.par_sort_unstable_by(|k, v| k.1.cmp(v.1).reverse().then(k.0.cmp(v.0)))
        }
        SortingOrder::None => (),
    }
}

fn output_counts<T: Write>(
    mut io: T,
    counts: Vec<(&Vec<u8>, &u64)>,
    n: usize,
    sig_pipe: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    for (key, count) in counts.into_iter().take(n) {
        writeln!(io, "{}\t{}", String::from_utf8(key.to_owned())?, count)?;
        if sig_pipe.load(Ordering::Relaxed) {
            break;
        }
    }
    Ok(())
}

fn watch_sig_pipe() -> Result<Arc<AtomicBool>, Box<dyn Error>> {
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
