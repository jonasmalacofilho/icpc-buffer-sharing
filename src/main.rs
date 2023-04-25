//! Copyright 2023 Jonas Malaco.

use std::collections::HashMap;
use std::io::{self, BufRead, Lines, Write};

fn main() {
    run(io::stdin().lock(), io::stdout().lock());
}

fn run(input: impl BufRead, mut output: impl Write) {
    let mut input = input.lines();

    let mut params = Params::from_lines(&mut input);
    // params.tune(20);

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

const N_MAX: usize = 10;

use lru_list::{LruList, NodeRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    T1,
    T2,
    B1,
    B2,
}

type NodeData = (Operation, u64, usize);

#[derive(Debug)]
struct Buffer {
    arc_dir: [HashMap<Page, (List, NodeRef<NodeData>)>; N_MAX],
    arc_t1: [LruList<NodeData>; N_MAX],
    arc_t2: [LruList<NodeData>; N_MAX],
    arc_b1: LruList<NodeData>,
    arc_b2: LruList<NodeData>,
    arc_p: usize,
    op_time: u64,
    max_loc: usize,

    counters: [Counters; N_MAX],
    params: Params,
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
    pub fn with_params(params: Params) -> Self {
        Buffer {
            arc_dir: std::array::from_fn(|_| HashMap::new()),
            arc_t1: std::array::from_fn(|_| LruList::new()),
            arc_t2: std::array::from_fn(|_| LruList::new()),
            arc_b1: LruList::new(),
            arc_b2: LruList::new(),
            arc_p: 0,
            op_time: 0,
            max_loc: 0,

            counters: [Counters::default(); N_MAX],
            params,
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.max_loc
    }

    pub fn locate(&mut self, op: Operation) -> usize {
        self.op_time += 1;

        let t = op.tenant.index();
        let p = op.page;

        if let Some((list, node_ref)) = self.arc_dir[t].remove(&p) {
            match list {
                List::T1 | List::T2 => {
                    // Cache hit in Arc and Virt. Move to MRU position in T2.
                    self.counters[t].hits += 1;

                    // Move from current list (keeping the buffer location)...
                    let list = match list {
                        List::T1 => &mut self.arc_t1[t],
                        List::T2 => &mut self.arc_t2[t],
                        _ => unreachable!(),
                    };
                    let (_, _, location) = unsafe { list.remove(node_ref) };

                    // ... to MRU position in T2.
                    self.push_mru_update_dir(List::T2, (op, self.op_time, location));

                    location
                }
                List::B1 => {
                    // Cache miss in Arc, but hit in Virt/L1. Move to MRU position in T2.
                    self.counters[t].misses += 1;

                    // Adapt/update `p`.
                    let delta = (self.arc_b2.len() / self.arc_b1.len()).max(1);
                    self.arc_p = (self.arc_p + delta).min(self.params.buffer_size_q);

                    // Evict some other operation and take its buffer location.
                    let location = self.replace(t, List::B1);

                    // Move from B1 to MRU position in T2.
                    let _ = unsafe { self.arc_b1.remove(node_ref) };
                    self.push_mru_update_dir(List::T2, (op, self.op_time, location));

                    location
                }
                List::B2 => {
                    // Cache miss in Arc, but hit in Virt/L2. Move to MRU position in T2.
                    self.counters[t].misses += 1;

                    // Adapt/update `p`.
                    let delta = (self.arc_b1.len() / self.arc_b2.len()).max(1);
                    self.arc_p = self.arc_p.saturating_sub(delta);

                    // Evict some other operation and take its buffer location.
                    let location = self.replace(t, List::B2);

                    // Move from B2 to MRU position in T2.
                    let _ = unsafe { self.arc_b2.remove(node_ref) };
                    self.push_mru_update_dir(List::T2, (op, self.op_time, location));

                    location
                }
            }
        } else {
            // Cache miss in Arc and Virt. Move to MRU position in T1.
            self.counters[t].misses += 1;

            let t1_len: usize = self.arc_t1.iter().map(|x| x.len()).sum::<usize>();
            let t2_len: usize = self.arc_t2.iter().map(|x| x.len()).sum::<usize>();
            let l1_len = t1_len + self.arc_b1.len();
            let at_qmax =
                (self.arc_t1[t].len() + self.arc_t2[t].len()) == self.params.buffer_sizes_qt[t].2;

            let location = if l1_len == self.params.buffer_size_q {
                if t1_len < self.params.buffer_size_q {
                    let (del, _, _) = self.arc_b1.pop_lru().unwrap();
                    self.arc_dir[del.tenant.index()].remove(&del.page);
                    self.replace(t, List::T1) // HACK: using T1 as !B2 (FIXME)
                } else {
                    // T1 == buffer_size_q
                    let (del, _, location) = self.pop_lru(List::T1, t).unwrap();
                    self.arc_dir[del.tenant.index()].remove(&del.page);
                    self.counters[del.tenant.index()].evictions += 1;
                    location
                }
            } else if l1_len + t2_len + self.arc_b2.len() >= self.params.buffer_size_q || at_qmax {
                if t1_len + t2_len + self.arc_b2.len() == 2 * self.params.buffer_size_q {
                    let (del, _, _) = self.arc_b2.pop_lru().unwrap();
                    self.arc_dir[del.tenant.index()].remove(&del.page);
                }
                self.replace(t, List::T1) // HACK: using T1 as !B2 (FIXME)
            } else {
                self.max_loc += 1;
                self.max_loc
            };

            self.push_mru_update_dir(List::T1, (op, self.op_time, location));

            location
        }
    }

    fn replace(&mut self, cur_t: usize, cur_list: List) -> usize {
        let t1_len: usize = self.arc_t1.iter().map(|x| x.len()).sum();

        let prio = if t1_len > 0
            && (t1_len > self.arc_p || (cur_list == List::B2 && t1_len == self.arc_p))
        {
            // Prefer to delete LRU *suitable* page in T1 and move it to MRU position in B1.
            [(List::T1, List::B1), (List::T2, List::B2)]
        } else {
            // Prefer to delete LRU *suitable* page in T2 and move it to MRU position in B2.
            [(List::T2, List::B2), (List::T1, List::B1)]
        };

        for (source, destination) in prio {
            if let Some((old_op, old_time, location)) = self.pop_lru(source, cur_t) {
                self.push_mru_update_dir(destination, (old_op, old_time, usize::MAX));
                self.counters[old_op.tenant.index()].evictions += 1;
                return location;
            }
        }

        unreachable!()
    }

    fn push_mru_update_dir(&mut self, dest: List, data: NodeData) {
        let t = data.0.tenant.index();
        let p = data.0.page;
        let list = match dest {
            List::T1 => &mut self.arc_t1[t],
            List::T2 => &mut self.arc_t2[t],
            List::B1 => &mut self.arc_b1,
            List::B2 => &mut self.arc_b2,
        };
        let new_ref = list.push_mru(data);
        self.arc_dir[t].insert(p, (dest, new_ref));
    }

    fn pop_lru(&mut self, source: List, recipient: usize) -> Option<NodeData> {
        match source {
            List::T1 | List::T2 => {
                let list = match source {
                    List::T1 => &self.arc_t1,
                    List::T2 => &self.arc_t2,
                    _ => unreachable!(),
                };

                if let Some((donor, _)) = list
                    .iter()
                    .zip(self.filter(recipient))
                    .enumerate()
                    .filter(|(_donor, (list, suitable))| *suitable && list.len() > 0)
                    .min_by_key(|(donor, (list, _))| {
                        let used = list.peek_lru().unwrap().1;
                        let qcur = self.arc_t1[*donor].len() + self.arc_t2[*donor].len();
                        let qbase = self.params.buffer_sizes_qt[*donor].1;
                        if qcur >= qbase {
                            (0, used)
                        } else {
                            (1, used)
                        }
                    })
                {
                    let list = match source {
                        List::T1 => &mut self.arc_t1[donor],
                        List::T2 => &mut self.arc_t2[donor],
                        _ => unreachable!(),
                    };
                    list.pop_lru()
                } else {
                    None
                }
            }
            List::B1 | List::B2 => {
                let list = match source {
                    List::B1 => &mut self.arc_b1,
                    List::B2 => &mut self.arc_b2,
                    _ => unreachable!(),
                };

                list.pop_lru()
            }
        }
    }

    fn filter(&self, recipient: usize) -> impl Iterator<Item = bool> + '_ {
        let at_qmax = (self.arc_t1[recipient].len() + self.arc_t2[recipient].len())
            == self.params.buffer_sizes_qt[recipient].2;
        self.arc_t1
            .iter()
            .zip(&self.arc_t2)
            .zip(&self.params.buffer_sizes_qt)
            .enumerate()
            .map(move |(et, ((t1, t2), (qmin, _, _)))| {
                let q = t1.len() + t2.len();
                if at_qmax {
                    et == recipient
                } else if et == recipient {
                    q >= *qmin
                } else {
                    q > *qmin
                }
            })
    }
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

    fn tune(&mut self, magic: usize) {
        let q = self.buffer_size_q;
        let psum: usize = self.priorities_lt.iter().map(|p| *p as usize).sum();
        for ((prio, dbsize), qsizes) in self
            .priorities_lt
            .iter()
            .zip(&self.db_size_dt)
            .zip(&mut self.buffer_sizes_qt)
        {
            let (mut qmin, qbase, qmax) = qsizes.to_owned();
            let magic_size = magic * (*prio as usize) * q / psum / 100;
            qmin = qmin.max(magic_size.min(qbase).min(*dbsize));
            *qsizes = (qmin, qbase, qmax);
        }
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

mod lru_list {
    use std::ptr::NonNull;

    #[derive(Debug)]
    pub struct LruList<T> {
        more: Link<T>,
        less: Link<T>,
        len: usize,
    }

    type Link<T> = Option<NonNull<Node<T>>>;

    #[derive(Debug)]
    struct Node<T> {
        more: Link<T>,
        less: Link<T>,
        data: T,
    }

    #[derive(Debug)]
    pub struct NodeRef<T> {
        node: NonNull<Node<T>>,
    }

    impl<T> LruList<T> {
        pub fn new() -> Self {
            LruList {
                more: None,
                less: None,
                len: 0,
            }
        }

        pub fn push_mru(&mut self, data: T) -> NodeRef<T> {
            let new = Box::into_raw(Box::new(Node {
                more: None,
                less: self.more,
                data,
            }));
            let new = NonNull::new(new).unwrap();
            if let Some(mut more) = self.more {
                unsafe { more.as_mut() }.more = Some(new);
            } else {
                self.less = Some(new);
            }
            self.more = Some(new);
            self.len += 1;
            NodeRef { node: new }
        }

        pub fn pop_mru(&mut self) -> Option<T> {
            self.more.map(|node| {
                let node = unsafe { Box::from_raw(node.as_ptr()) };
                let data = node.data;
                self.more = node.less;
                if let Some(mut more) = self.more {
                    unsafe { more.as_mut() }.more = None;
                } else {
                    self.less = None;
                }
                self.len -= 1;
                data
            })
        }

        #[allow(dead_code)]
        pub fn peek_mru(&self) -> Option<&T> {
            self.more.map(|node| unsafe { &node.as_ref().data })
        }

        #[allow(dead_code)]
        pub fn push_lru(&mut self, data: T) -> NodeRef<T> {
            let new = Box::into_raw(Box::new(Node {
                more: self.less,
                less: None,
                data,
            }));
            let new = NonNull::new(new).unwrap();
            if let Some(mut less) = self.less {
                unsafe { less.as_mut() }.less = Some(new);
            } else {
                self.more = Some(new);
            }
            self.less = Some(new);
            self.len += 1;
            NodeRef { node: new }
        }

        pub fn pop_lru(&mut self) -> Option<T> {
            self.less.map(|node| {
                let node = unsafe { Box::from_raw(node.as_ptr()) };
                let data = node.data;
                self.less = node.more;
                if let Some(mut less) = self.less {
                    unsafe { less.as_mut() }.less = None;
                } else {
                    self.more = None;
                }
                self.len -= 1;
                data
            })
        }

        pub fn peek_lru(&self) -> Option<&T> {
            self.less.map(|node| unsafe { &node.as_ref().data })
        }

        // SAFETY: caller must ensure `node_ref` belongs to this list and isn't dangling. To be
        // safe, every time a node is popped from the list, the caller should discard the its
        // corresponding `NodeRef`.
        pub unsafe fn remove(&mut self, node_ref: NodeRef<T>) -> T {
            let node = unsafe { Box::from_raw(node_ref.node.as_ptr()) };
            let data = node.data;
            if let Some(mut less) = node.less {
                unsafe { less.as_mut() }.more = node.more;
            } else {
                self.less = node.more;
            }
            if let Some(mut more) = node.more {
                unsafe { more.as_mut() }.less = node.less;
            } else {
                self.more = node.less;
            }
            self.len -= 1;
            data
        }

        // SAFETY: caller must ensure `node_ref` belongs to this list and isn't dangling. To be
        // safe, every time a node is popped from the list, the caller should discard the its
        // corresponding `NodeRef`.
        #[allow(dead_code)]
        pub unsafe fn peek_inside(&self, node_ref: &NodeRef<T>) -> &T {
            unsafe { &node_ref.node.as_ref().data }
        }

        pub fn len(&self) -> usize {
            self.len
        }
    }

    impl<T> Drop for LruList<T> {
        fn drop(&mut self) {
            while self.pop_mru().is_some() {}
        }
    }
}

#[cfg(test)]
mod tests;
