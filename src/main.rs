//! Copyright 2023 Jonas Malaco.

use std::collections::{BinaryHeap, HashMap};
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
            "tenant {:>2}: {:>7} /{:>3.0}% hits, {:>7} /{:>3.0}% misses, {:>7} evictions",
            t + 1,
            c.hits,
            100. * c.hits as f32 / (c.hits as f32 + c.misses as f32),
            c.misses,
            100. * c.misses as f32 / (c.hits as f32 + c.misses as f32),
            c.evictions
        );
        hits += c.hits;
        misses += c.misses;
        evictions += c.evictions;
    }
    eprintln!(
        "    total: {:>7} /{:>3.0}% hits, {:>7} /{:>3.0}% misses, {:>7} evictions",
        hits,
        100. * hits as f32 / (hits as f32 + misses as f32),
        misses,
        100. * misses as f32 / (hits as f32 + misses as f32),
        evictions,
    );
}

#[derive(Debug, Clone)]
struct Buffer {
    params: Params,
    maps: Vec<HashMap<Page, (u64, usize)>>,
    heaps: Vec<BinaryHeap<HeapEntry>>,
    counters: Vec<Counters>,
    max_loc: usize,
    now: u64,
}

#[derive(Debug, Clone, Copy, Eq)]
struct HeapEntry(Page, u64);

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.1.cmp(&self.1)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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
            maps: vec![Default::default(); params.num_tenants_n],
            heaps: vec![Default::default(); params.num_tenants_n],
            counters: vec![Default::default(); params.num_tenants_n],
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
        if let Some((used, loc)) = self.maps[op.tenant.index()].get_mut(&op.page) {
            self.counters[op.tenant.index()].hits += 1;
            *used = self.now;
            let loc = *loc;
            self.check_invariants();
            return loc;
        }

        self.counters[op.tenant.index()].misses += 1;

        let at_capacity =
            self.maps[op.tenant.index()].len() == self.params.buffer_sizes_qt[op.tenant.index()].2;

        // Buffer and tenant not at capacity, insert op in empty space.
        if !at_capacity && self.len() < self.params.buffer_size_q {
            self.max_loc += 1;
            self.maps[op.tenant.index()].insert(op.page, (self.now, self.max_loc));
            self.heaps[op.tenant.index()].push(HeapEntry(op.page, self.now));
            self.check_invariants();
            return self.max_loc;
        }

        // Reinsert any entries at the front of the heaps with outdated `used` values.
        for (heap, map) in self.heaps.iter_mut().zip(&self.maps) {
            while let Some(&HeapEntry(p, used)) = heap.peek() {
                let &(last_used, _) = &map[&p];
                if used == last_used {
                    break;
                }
                heap.pop();
                heap.push(HeapEntry(p, last_used));
            }
        }

        // Contented, find the least worst page to swap with. If tenant is at capacity, it must
        // swap with one of its own pages.
        let (evict_owner, _) = self
            .maps
            .iter()
            .zip(&self.params.buffer_sizes_qt)
            .enumerate()
            .filter(|(t, (map, (qmin, _, _)))| {
                if at_capacity {
                    *t == op.tenant.index()
                } else if *t == op.tenant.index() {
                    map.len() >= *qmin
                } else {
                    map.len() > *qmin
                }
            })
            .min_by_key(|(t, (map, (_, qbase, _)))| {
                let &HeapEntry(_p, used) = self.heaps[*t].peek().unwrap();
                if map.len() > *qbase {
                    (0, used)
                } else {
                    (1, used)
                }
            })
            .unwrap();
        let HeapEntry(evict_page, used) = self.heaps[evict_owner].pop().unwrap();
        let &(last_used, loc) = &self.maps[evict_owner][&evict_page];
        debug_assert_eq!(used, last_used);
        self.maps[evict_owner].remove(&evict_page);
        self.counters[evict_owner].evictions += 1;
        self.maps[op.tenant.index()].insert(op.page, (self.now, loc));
        self.heaps[op.tenant.index()].push(HeapEntry(op.page, self.now));
        self.check_invariants();
        loc
    }

    #[cfg(debug_assertions)]
    fn check_invariants(&self) {
        assert_eq!(self.max_loc, self.maps.iter().map(|x| x.len()).sum());

        assert!(self.len() <= self.params.buffer_size_q);

        for (t, (map, (_, _, qmax))) in self
            .maps
            .iter()
            .zip(&self.params.buffer_sizes_qt)
            .enumerate()
        {
            assert!(map.len() <= *qmax);
            assert!(map.len() <= self.params.db_size_dt[t]);
        }

        for (map, heap) in self.maps.iter().zip(&self.heaps) {
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
    pub evictions: u32,
}

#[cfg(test)]
mod tests;
