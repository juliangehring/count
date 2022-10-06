use clap::{Parser, ValueEnum};
use hashbrown::HashMap;
use partial_sort::PartialSort;
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use std::{
    cmp::Ord,
    error::Error,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

#[derive(Clone, Debug, Parser, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum SortingOrder {
    count,
    key,
    none,
}

#[derive(Parser, Debug)]
pub struct Config {
    #[clap(
        short,
        long,
        default_value = "count",
        ignore_case(true),
        value_enum,
        value_parser
    )]
    sort_by: SortingOrder,
    #[clap(short, long)]
    max_items: Option<usize>,
    #[clap(value_parser)]
    input: Option<PathBuf>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let reader = create_reader(&config.input)?;

    let counter = count_items(reader);

    let mut counts: Vec<_> = counter.par_iter().collect();

    sort_counts(&mut counts, &config.sort_by, config.max_items);

    let stdout = stdout();
    let stdout = stdout.lock();
    let stdout = BufWriter::new(stdout);

    output_counts(stdout, counts, config.max_items)?;

    Ok(())
}

fn create_reader(input: &Option<PathBuf>) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
    let reader: Box<dyn BufRead> = match input {
        Some(file_name) => Box::new(BufReader::new(File::open(file_name)?)),
        None => Box::new(BufReader::new(stdin())),
    };

    Ok(reader)
}

fn count_items(mut reader: Box<dyn BufRead>) -> HashMap<Vec<u8>, u64> {
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

    counter
}

fn sort_counts<S: Ord + Sync, T: Ord + Sync>(
    counts: &mut Vec<(&S, &T)>,
    sorting_order: &SortingOrder,
    num_items: Option<usize>,
) {
    match (sorting_order, num_items) {
        (SortingOrder::key, Some(n)) => {
            counts.partial_sort(n, |k, v| k.0.cmp(v.0).then(k.1.cmp(k.1).reverse()))
        }
        (SortingOrder::count, Some(n)) => {
            counts.partial_sort(n, |k, v| k.1.cmp(v.1).reverse().then(k.0.cmp(v.0)))
        }
        (SortingOrder::key, None) => {
            counts.par_sort_unstable_by(|k, v| k.0.cmp(v.0).then(k.1.cmp(k.1).reverse()))
        }
        (SortingOrder::count, None) => {
            counts.par_sort_unstable_by(|k, v| k.1.cmp(v.1).reverse().then(k.0.cmp(v.0)))
        }
        (SortingOrder::none, _) => (),
    }
}

fn output_counts<T: Write>(
    mut io: T,
    counts: Vec<(&Vec<u8>, &u64)>,
    num_items: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let n = num_items.unwrap_or(counts.len());
    for (key, count) in counts.into_iter().take(n) {
        writeln!(io, "{}\t{}", String::from_utf8(key.to_owned())?, count)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{sort_counts, SortingOrder};

    #[test]
    fn test_full_sort_counts_by_key() {
        let mut input = vec![(&"b", &3), (&"c", &2), (&"a", &1)];
        let output = vec![(&"a", &1), (&"b", &3), (&"c", &2)];

        sort_counts(&mut input, &SortingOrder::key, None);

        assert_eq!(input, output);
    }

    #[test]
    fn test_partial_sort_counts_by_key() {
        let mut input = vec![(&"b", &3), (&"c", &2), (&"a", &1)];
        let output = vec![(&"a", &1), (&"b", &3), (&"c", &2)];

        sort_counts(&mut input, &SortingOrder::key, Some(2));

        assert_eq!(input, output);
    }

    #[test]
    fn test_full_sort_counts_by_counts() {
        let mut input = vec![(&"c", &2), (&"a", &1), (&"b", &3)];
        let output = vec![(&"b", &3), (&"c", &2), (&"a", &1)];

        sort_counts(&mut input, &SortingOrder::count, None);

        assert_eq!(input, output);
    }

    #[test]
    fn test_partial_sort_counts_by_counts() {
        let mut input = vec![(&"c", &2), (&"a", &1), (&"b", &3)];
        let output = vec![(&"b", &3), (&"c", &2), (&"a", &1)];

        sort_counts(&mut input, &SortingOrder::count, Some(2));

        assert_eq!(input, output);
    }
}
