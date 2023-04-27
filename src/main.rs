//! N×LFU, in a very silly implementation.
//!
//! Copyright 2023 Jonas Malaco.

use std::collections::{BinaryHeap, HashMap};
use std::io::{self, BufRead, Lines, Write};
use std::time::Instant;

const TENANT_CAP: usize = 10;

fn main() {
    let instant = Instant::now();
    run(io::stdin().lock(), io::stdout().lock());
    eprintln!("execution took {:.1?}", Instant::elapsed(&instant));
}

fn run(input: impl BufRead, mut output: impl Write) {
    let mut input = input.lines();

    let params = Params::from_lines(&mut input);

    debug_assert!(
        params
            .buffer_sizes_qt
            .iter()
            .map(|(qmin, _, _)| qmin)
            .sum::<usize>()
            <= params.buffer_size_q
    );

    let mut buffer = Buffer::with_params(params.clone());

    while let Some(op) = Operation::from_lines(&mut input) {
        debug_assert!((1..=params.num_tenants_n).contains(&(op.tenant.0 as usize)));
        debug_assert!((1..=params.db_size_dt[op.tenant.index()]).contains(&(op.page.0 as usize)));

        let loc = buffer.locate(op);
        debug_assert!((1..=params.buffer_size_q).contains(&loc));

        writeln!(&mut output, "{loc}").unwrap();
        output.flush().unwrap();
    }

    let mut hits = 0;
    let mut evictions = 0;
    let mut misses = 0;
    let mut preventable_misses = 0;

    for (t, c) in buffer
        .counters
        .iter()
        .enumerate()
        .filter(|(_, c)| c.hits + c.misses > 0)
    {
        eprintln!(
            "tenant {:>2}: {:>7} /{:>3.0}% hits, \
                    {:>7} evictions, \
                    {:>7} /{:>3.0}% misses, \
                    {:>7} /{:>3.0}% preventable misses",
            t + 1,
            c.hits,
            100. * c.hits as f32 / (c.hits as f32 + c.misses as f32),
            c.evictions,
            c.misses,
            100. * c.misses as f32 / (c.hits as f32 + c.misses as f32),
            c.preventable_misses,
            100. * c.preventable_misses as f32 / (c.hits as f32 + c.misses as f32),
        );

        hits += c.hits;
        evictions += c.evictions;
        misses += c.misses;
        preventable_misses += c.preventable_misses;

        debug_assert!(c.misses == 0 || c.preventable_misses < c.misses);
        debug_assert!(c.preventable_misses <= c.evictions);
    }

    eprintln!(
        "    total: {:>7} /{:>3.0}% hits, \
                    {:>7} evictions, \
                    {:>7} /{:>3.0}% misses, \
                    {:>7} /{:>3.0}% preventable misses",
        hits,
        100. * hits as f32 / (hits as f32 + misses as f32),
        evictions,
        misses,
        100. * misses as f32 / (hits as f32 + misses as f32),
        preventable_misses,
        100. * preventable_misses as f32 / (hits as f32 + misses as f32),
    );
}

#[derive(Debug, Clone)]
struct Buffer {
    params: Params,
    directory: [HashMap<Page, (u64, usize)>; TENANT_CAP],
    recently_seen: [BinaryHeap<HeapEntry>; TENANT_CAP],
    all_time_seen: [HashMap<Page, u64>; TENANT_CAP],
    counters: [Counters; TENANT_CAP],
    max_loc: usize,
    now: u64,
}

#[derive(Debug, Clone, Copy, Eq)]
struct HeapEntry(Page, u64, u64);

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        (self.1, self.2) == (other.1, other.2)
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.1, other.2).cmp(&(self.1, self.2))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Buffer {
    fn with_params(params: Params) -> Self {
        Buffer {
            directory: std::array::from_fn(|_| Default::default()),
            recently_seen: std::array::from_fn(|_| Default::default()),
            all_time_seen: std::array::from_fn(|_| Default::default()),
            counters: std::array::from_fn(|_| Default::default()),
            max_loc: 0,
            now: 0,
            params,
        }
    }

    fn len(&self) -> usize {
        self.max_loc
    }

    fn locate(&mut self, op: Operation) -> usize {
        let t = op.tenant.index();
        let p = op.page;

        self.now += 1;

        // Op already in the buffer, return its location.
        if let Some((used, loc)) = self.directory[t].get_mut(&p) {
            let loc = *loc;
            self.counters[t].hits += 1;
            self.all_time_seen[t]
                .entry(p)
                .and_modify(|c| *c += 1)
                .or_insert(1);
            *used = self.now;
            self.check_invariants();
            return loc;
        }

        self.counters[t].misses += 1;
        if self.all_time_seen[t].contains_key(&p) {
            self.counters[t].preventable_misses += 1;
        }

        let at_capacity = self.directory[t].len() == self.params.buffer_sizes_qt[t].2;

        // Buffer and tenant not at capacity, insert op in empty space.
        if !at_capacity && self.len() < self.params.buffer_size_q {
            self.max_loc += 1;
            self.directory[t].insert(p, (self.now, self.max_loc));
            let c = self.all_time_seen[t]
                .entry(p)
                .and_modify(|c| *c += 1)
                .or_insert(1);
            self.recently_seen[t].push(HeapEntry(p, *c, self.now));
            self.check_invariants();
            return self.max_loc;
        }

        // Reinsert any entries at the front of the heaps with outdated `used` values.
        for ((heap, seen), subdir) in self
            .recently_seen
            .iter_mut()
            .zip(&self.all_time_seen)
            .zip(&self.directory)
        {
            while let Some(&HeapEntry(p, c, used)) = heap.peek() {
                let &latest_c = &seen[&p];
                let &latest_used = &subdir[&p].0;
                if c == latest_c && used == latest_used {
                    break;
                }
                heap.pop();
                heap.push(HeapEntry(p, latest_c, latest_used));
            }
        }

        // Contented, find the least worst page to swap with. If tenant is at capacity, it must
        // swap with one of its own pages.
        let (donor, _) = self
            .directory
            .iter()
            .zip(&self.params.buffer_sizes_qt)
            .enumerate()
            .filter(|(donor, (subdir, (qmin, _, _)))| {
                if at_capacity {
                    *donor == t
                } else if *donor == t {
                    subdir.len() >= *qmin
                } else {
                    subdir.len() > *qmin
                }
            })
            .min_by_key(|(donor, _)| {
                let &HeapEntry(_, c, used) = self.recently_seen[*donor].peek().unwrap();
                (c, used)
            })
            .unwrap();

        // Evict the worst page.
        let HeapEntry(del, _, del_used) = self.recently_seen[donor].pop().unwrap();
        let &(last_used, loc) = &self.directory[donor][&del];
        self.directory[donor].remove(&del);
        self.counters[donor].evictions += 1;
        debug_assert_eq!(del_used, last_used);

        // Store the new page.
        self.directory[t].insert(p, (self.now, loc));
        let c = self.all_time_seen[t]
            .entry(p)
            .and_modify(|c| *c += 1)
            .or_insert(1);
        self.recently_seen[t].push(HeapEntry(p, *c, self.now));
        self.check_invariants();
        loc
    }

    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        assert_eq!(self.max_loc, self.directory.iter().map(|x| x.len()).sum());

        assert!(self.len() <= self.params.buffer_size_q);

        for (t, (map, (_, _, qmax))) in self
            .directory
            .iter()
            .zip(&self.params.buffer_sizes_qt)
            .enumerate()
        {
            assert!(map.len() <= *qmax);
            assert!(map.len() <= self.params.db_size_dt[t]);
        }

        for (map, heap) in self.directory.iter().zip(&self.recently_seen) {
            assert_eq!(map.len(), heap.len());
        }
    }

    #[cfg(not(debug_assertions))]
    fn check_invariants(&self) {}
}

#[derive(Debug, Clone, Default)]
struct Params {
    pub num_tenants_n: usize,
    pub buffer_size_q: usize,
    // pub op_seq_size_m: usize,
    pub priorities_lt: Vec<u8>,
    pub db_size_dt: Vec<usize>,
    pub buffer_sizes_qt: Vec<(usize, usize, usize)>,
}

impl Params {
    fn from_lines(input: &mut Lines<impl BufRead>) -> Params {
        fn read_numbers(line: Option<Result<String, std::io::Error>>) -> Vec<usize> {
            line.unwrap()
                .unwrap()
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect()
        }

        let mut params = Params::default();

        let line = read_numbers(input.next());
        params.num_tenants_n = line[0];
        params.buffer_size_q = line[1];
        // params.op_seq_size_m = line[2];

        params.priorities_lt = read_numbers(input.next())
            .into_iter()
            .map(|lt| lt.try_into().unwrap())
            .collect();

        params.db_size_dt = read_numbers(input.next());

        let line = read_numbers(input.next());
        params.buffer_sizes_qt = line
            .chunks_exact(3)
            .map(|triple| {
                if let &[min, base, max] = triple {
                    (min, base, max)
                } else {
                    unreachable!()
                }
            })
            .collect();

        params
    }
}

/// Operation/access `Ai`, where `1 <= i <= M`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Operation {
    tenant: Tenant,
    page: Page,
}

impl Operation {
    fn from_lines(input: &mut Lines<impl BufRead>) -> Option<Operation> {
        input.next().map(|line| {
            let line = line.unwrap();
            let mut split = line.split_whitespace();
            let ui = split.next().unwrap().parse().unwrap();
            let pi = split.next().unwrap().parse().unwrap();
            Operation {
                tenant: Tenant(ui),
                page: Page(pi),
            }
        })
    }
}

/// Tenant `Ui`, where `1 <= i <= M`.
// FIXME: encode non-zero invariant (if applicable) in the type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Tenant(u8);

impl Tenant {
    fn index(&self) -> usize {
        (self.0 - 1) as _
    }
}

/// Page/object `Pi`, where `1 <= i <= M`.
// FIXME: encode non-zero invariant (if applicable) in the type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Page(u32);

#[derive(Debug, Clone, Copy, Default)]
struct Counters {
    pub hits: u32,
    pub misses: u32,
    pub preventable_misses: u32,
    pub evictions: u32,
}

#[cfg(test)]
mod tests;
