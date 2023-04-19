use super::*;

fn op(ti: u8, pi: u32) -> Operation {
    Operation {
        tenant: Tenant(ti),
        page: Page(pi),
    }
}

#[test]
fn page_hit() {
    let params = Params {
        num_tenants_n: 2,
        buffer_size_q: 10,
        priorities_lt: vec![1; 2],
        db_size_dt: vec![10; 2],
        buffer_sizes_qt: vec![(1, 1, 1); 2],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));
    let b = buffer.locate(op(2, 10));
    assert_ne!(a, b);

    assert_eq!(buffer.locate(op(2, 10)), b);
    assert_eq!(buffer.locate(op(1, 1)), a);
    assert_eq!(buffer.len(), 2);
}

#[test]
fn tenant_at_qmax() {
    let params = Params {
        num_tenants_n: 1,
        buffer_size_q: 10,
        priorities_lt: vec![1; 1],
        db_size_dt: vec![10; 1],
        buffer_sizes_qt: vec![(1, 1, 1); 1],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));

    assert_eq!(buffer.locate(op(1, 2)), a);
    assert_eq!(buffer.len(), 1);
}

#[test]
fn buffer_and_tenant_bellow_capacity() {
    let params = Params {
        num_tenants_n: 1,
        buffer_size_q: 10,
        priorities_lt: vec![1; 1],
        db_size_dt: vec![10; 1],
        buffer_sizes_qt: vec![(1, 2, 3); 1],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 3));
    let b = buffer.locate(op(1, 2));
    let c = buffer.locate(op(1, 1));

    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(c, a);
    assert_eq!(buffer.len(), 3);
}

#[test]
fn tenant_bellow_qmin() {
    let params = Params {
        num_tenants_n: 2,
        buffer_size_q: 4,
        priorities_lt: vec![1; 2],
        db_size_dt: vec![10; 2],
        buffer_sizes_qt: vec![(2, 2, 4); 2],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));
    buffer.locate(op(2, 1));
    buffer.locate(op(2, 2));
    buffer.locate(op(2, 3));
    let b = buffer.locate(op(1, 2));
    assert_eq!(buffer.len(), 4);

    assert_ne!(a, b);
}

#[test]
fn foo() {}

#[test]
fn smoke() {
    let inp = include_bytes!("../input.txt").as_slice();
    let mut out = vec![];
    run(inp, &mut out);
    assert_eq!(out.lines().count(), 10);
}

#[test]
fn parse_maximums() {
    let inp = b"\
        10 1000000 1000000\n\
        10 10 10 10 10 10 10 10 10 10\n\
        100000 100000 100000 100000 100000 100000 100000 100000 100000 100000\n\
        100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000 100000\
    ".as_slice();
    Params::from_lines(&mut inp.lines());
}

#[test]
fn lru2_ord() {
    use std::cmp::Ordering::*;

    const NEVER: u64 = 0;

    fn hist(hist: [u64; 2]) -> Lru2Entry{
        Lru2Entry(Page(42), hist)
    }

    // Because of `std::collections::BinaryHeap` is always a max-heap, `Lru2Entry` implements `Ord`
    // in terms of backward distances.

    assert_eq!(hist([10, 2]).cmp(&hist([8, 4])), Greater);
    assert_eq!(hist([10, 2]).cmp(&hist([8, NEVER])), Less);
    assert_eq!(hist([10, NEVER]).cmp(&hist([8, 2])), Greater);
    assert_eq!(hist([10, NEVER]).cmp(&hist([8, NEVER])), Less);
}

#[test]
fn lru2_crp() {
    const NEVER: u64 = 0;

    let crp = 3;
    let mut hist = [2, NEVER];

    update_lru2_history(&mut hist, 3, crp);
    assert_eq!(hist, [2, NEVER]);

    update_lru2_history(&mut hist, 4, crp);
    assert_eq!(hist, [2, NEVER]);

    update_lru2_history(&mut hist, 5, crp);
    assert_eq!(hist, [5, 2]);

    update_lru2_history(&mut hist, 6, crp);
    assert_eq!(hist, [5, 2]);

    update_lru2_history(&mut hist, 7, crp);
    assert_eq!(hist, [5, 2]);

    update_lru2_history(&mut hist, 8, crp);
    assert_eq!(hist, [8, 5]);
}
