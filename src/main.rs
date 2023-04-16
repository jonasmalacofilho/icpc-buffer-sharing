//! Copyright 2023 Jonas Malaco.

use std::collections::HashMap;
use std::io::{self, BufRead, Lines, Write};

fn main() {
    run(io::stdin().lock(), io::stdout().lock());
}

fn run(input: impl BufRead, mut output: impl Write) {
    let mut input = input.lines();

    let params = Params::from_lines(&mut input);
    let mut buffer = Buffer::with_params(params.clone());

    while let Some(op) = Operation::from_lines(&mut input) {
        debug_assert!((1..=params.num_tenants_n).contains(&(op.tenant.0 as usize)));
        debug_assert!((1..=params.db_size_dt[op.tenant.index()]).contains(&(op.page.0 as usize)));

        let loc = buffer.locate(op);
        debug_assert!((1..=params.buffer_size_q).contains(&loc));

        writeln!(&mut output, "{loc}").unwrap();
        output.flush().unwrap();
    }

    let (mut hits, mut misses, mut evictions) = (0, 0, 0);
    for (t, c) in buffer.counters.iter().enumerate() {
        eprintln!(
            "tenant {}: {} hits, {} misses, {} evictions",
            t + 1,
            c.hits,
            c.misses,
            c.evictions
        );
        hits += c.hits;
        misses += c.misses;
        evictions += c.evictions;
    }
    eprintln!(
        "total: {} hits, {} misses, {} evictions",
        hits, misses, evictions,
    );
}

#[derive(Debug, Clone)]
struct Buffer {
    ledgers: Vec<HashMap<Page, (u64, usize)>>,
    max_loc: usize,
    params: Params,
    now: u64,
    counters: Vec<Counter>,
}

impl Buffer {
    fn with_params(params: Params) -> Self {
        debug_assert!(
            params
                .buffer_sizes_qt
                .iter()
                .map(|(qmin, _, _)| qmin)
                .sum::<usize>()
                <= params.buffer_size_q
        );

        Buffer {
            ledgers: vec![Default::default(); params.num_tenants_n],
            counters: vec![Default::default(); params.num_tenants_n],
            max_loc: 0,
            now: 0,
            params,
        }
    }

    fn len(&self) -> usize {
        debug_assert_eq!(self.max_loc, self.ledgers.iter().map(|x| x.len()).sum());
        self.max_loc
    }

    fn locate(&mut self, op: Operation) -> usize {
        self.now += 1;

        // Op already in the buffer, return its location.
        if let Some((used, loc)) = self.ledgers[op.tenant.index()].get_mut(&op.page) {
            *used = self.now;
            let loc = *loc;
            self.check_invariants();
            self.counters[op.tenant.index()].hits += 1;
            return loc;
        }

        self.counters[op.tenant.index()].misses += 1;

        // Tenant at capacity, must swap with one of its own pages.
        if self.ledgers[op.tenant.index()].len() == self.params.buffer_sizes_qt[op.tenant.index()].2
        {
            let (&evicted, &(_, loc)) = self.ledgers[op.tenant.index()]
                .iter()
                .min_by_key(|(_, (used, _))| used)
                .unwrap();
            self.ledgers[op.tenant.index()].remove(&evicted);
            self.ledgers[op.tenant.index()].insert(op.page, (self.now, loc));
            self.check_invariants();
            self.counters[op.tenant.index()].evictions += 1;
            return loc;
        }

        // Buffer and tenant not at capacity, insert op in empty space.
        if self.len() < self.params.buffer_size_q {
            self.max_loc += 1;
            self.ledgers[op.tenant.index()].insert(op.page, (self.now, self.max_loc));
            self.check_invariants();
            return self.max_loc;
        }

        // Contented, find the least worst page to swap with.
        let (tidx, &evicted, &(_, loc)) = self
            .ledgers
            .iter()
            .zip(self.params.buffer_sizes_qt.iter())
            .enumerate()
            // FIXME: can replace own page if at Qmin (but pages of other tenants only if they're
            // *above* their Qmin).
            .filter(|(_, (ledger, (qtmin, _, _)))| ledger.len() > *qtmin)
            .flat_map(|(tidx, (ledger, _))| ledger.iter().map(move |(k, v)| (tidx, k, v)))
            .min_by_key(|(_, _, (used, _))| used)
            .unwrap();
        self.ledgers[tidx].remove(&evicted);
        self.ledgers[op.tenant.index()].insert(op.page, (self.now, loc));
        self.check_invariants();
        self.counters[tidx].evictions += 1;
        loc
    }

    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        assert!(self.len() <= self.params.buffer_size_q);
        for (tidx, ledger) in self.ledgers.iter().enumerate() {
            let (_, _, qmax) = self.params.buffer_sizes_qt[tidx];
            assert!(ledger.len() <= qmax);
            assert!(ledger.len() <= self.params.db_size_dt[tidx]);
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Tenant(u8);

impl Tenant {
    fn index(&self) -> usize {
        (self.0 - 1) as _
    }
}

/// Page/object `Pi`, where `1 <= i <= M`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Page(u32);

#[derive(Debug, Clone, Copy, Default)]
struct Counter {
    pub hits: u32,
    pub misses: u32,
    pub evictions: u32,
}

#[cfg(test)]
mod tests;
