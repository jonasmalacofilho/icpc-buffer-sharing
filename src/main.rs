//! Copyright 2023 Jonas Malaco.

use std::io::{self, BufRead, Lines, Write};
use std::num::NonZeroUsize;

fn main() {
    run(io::stdin().lock(), io::stdout().lock());
}

fn run(input: impl BufRead, mut output: impl Write) {
    let mut input = input.lines();

    let params = Params::from_lines(&mut input);
    let mut buffer = Buffer::with_params(params.clone());

    while let Some(op) = Operation::from_lines(&mut input) {
        debug_assert!((1..=params.num_tenants_n).contains(&(op.tenant.0 as usize)));
        debug_assert!((1..=params.db_size_dt[op.tenant.index()]).contains(&op.page.0));

        let loc = buffer.locate(op);
        debug_assert!((1..=params.buffer_size_q).contains(&loc));

        writeln!(&mut output, "{loc}").unwrap();
        output.flush().unwrap();
    }
}

#[derive(Debug, Clone)]
struct Buffer {
    // Tenant -> Page -> (used, loc)
    ledgers: Vec<Vec<(u64, Option<NonZeroUsize>)>>,
    // Tenent -> pages in buffer
    ledger_sizes: Vec<usize>,
    max_loc: usize,
    params: Params,
    now: u64,
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
            // FIXME: don't allocate more memory than necessary.
            ledgers: vec![vec![(0, None); 100_000]; params.num_tenants_n],
            ledger_sizes: vec![0; params.num_tenants_n],
            max_loc: 0,
            now: 0,
            params,
        }
    }

    fn len(&self) -> usize {
        self.max_loc
    }

    fn locate(&mut self, op: Operation) -> usize {
        self.now += 1;

        // Op already in the buffer, return its location.
        if let (used, Some(loc)) = &mut self.ledgers[op.tenant.index()][op.page.index()] {
            *used = self.now;
            let loc = *loc;
            self.check_invariants();
            return loc.into();
        }

        // // Tenant at capacity, must swap with one of its own pages.
        if self.ledger_sizes[op.tenant.index()] == self.params.buffer_sizes_qt[op.tenant.index()].2
        {
            let (evict_page, &(evict_used, loc)) = self.ledgers[op.tenant.index()]
                .iter()
                .enumerate()
                .filter(|(_page, (_used, loc))| loc.is_some())
                .min_by_key(|(_page, (used, _loc))| used)
                .unwrap();
            let loc = loc.unwrap();
            eprintln!(
                "// replacing own {evict_page:?}, last used {evict_used} (now is {})",
                self.now
            );
            self.ledgers[op.tenant.index()][evict_page].1 = None;
            self.ledgers[op.tenant.index()][op.page.index()] = (self.now, Some(loc));
            self.check_invariants();
            return loc.into();
        }

        // Buffer and tenant not at capacity, insert op in empty space.
        if self.len() < self.params.buffer_size_q {
            self.max_loc += 1;
            self.ledgers[op.tenant.index()][op.page.index()] =
                (self.now, Some(NonZeroUsize::new(self.max_loc).unwrap()));
            self.ledger_sizes[op.tenant.index()] += 1;
            self.check_invariants();
            return self.max_loc;
        }

        // // Contented, find the least worst page to swap with.
        let (evict_tenant, evict_page, evict_used, loc) = self
            .ledgers
            .iter()
            .zip(&self.ledger_sizes)
            .zip(self.params.buffer_sizes_qt.iter().map(|q| q.0))
            .enumerate()
            .filter(|(t, ((_ledger, qt), qmin))| {
                *qt > qmin || (*t == op.tenant.index() && *qt == qmin)
            })
            .map(move |(t, ((ledger, _), _))| {
                let (p, &(used, loc)) = ledger
                    .iter()
                    .enumerate()
                    .filter(|(_page, (_used, loc))| loc.is_some())
                    .min_by_key(|(_page, (used, _loc))| used)
                    .unwrap();
                (t, p, used, loc)
            })
            .min_by_key(|(_t, _p, used, _loc)| *used)
            .unwrap();
        let loc = loc.unwrap();
        eprintln!(
            "// replacing {evict_tenant:?}'s {evict_page:?}, last used {evict_used} (now is {})",
            self.now
        );
        self.ledgers[evict_tenant][evict_page].1 = None;
        self.ledger_sizes[evict_tenant] -= 1;
        self.ledgers[op.tenant.index()][op.page.index()] = (self.now, Some(loc));
        self.ledger_sizes[op.tenant.index()] += 1;
        self.check_invariants();
        loc.into()
    }

    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        debug_assert_eq!(
            self.len(),
            self.ledgers
                .iter()
                .map(|x| x.iter().filter(|x| x.1.is_some()).count())
                .sum()
        );

        assert!(self.len() <= self.params.buffer_size_q);

        for (t, ledger) in self.ledgers.iter().enumerate() {
            let qt = ledger.iter().filter(|x| x.1.is_some()).count();
            assert_eq!(self.ledger_sizes[t], qt);

            let (_, _, qmax) = self.params.buffer_sizes_qt[t];
            assert!(qt <= qmax);
            assert!(qt <= self.params.db_size_dt[t]);
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
struct Page(usize);

impl Page {
    fn index(&self) -> usize {
        (self.0 - 1) as _
    }
}

#[cfg(test)]
mod tests;
