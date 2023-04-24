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
fn arc_tenant_at_qmax_fully_in_t1() {
    let params = Params {
        num_tenants_n: 2,
        buffer_size_q: 10,
        priorities_lt: vec![1; 2],
        db_size_dt: vec![10; 2],
        buffer_sizes_qt: vec![(1, 1, 1); 2],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));

    // Increase `p`.
    buffer.locate(op(2, 1));
    buffer.locate(op(2, 1));
    buffer.locate(op(2, 1));

    assert_eq!(buffer.locate(op(1, 2)), a);
    assert_eq!(buffer.len(), 2);
}

#[test]
fn arc_tenant_at_qmax_fully_in_t2() {
    let params = Params {
        num_tenants_n: 2,
        buffer_size_q: 10,
        priorities_lt: vec![1; 2],
        db_size_dt: vec![10; 2],
        buffer_sizes_qt: vec![(1, 1, 1); 2],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));

    // Move `op(1, 1)` to T2.
    buffer.locate(op(1, 1));

    // Decrease `p`.
    buffer.locate(op(2, 1));
    buffer.locate(op(2, 2));
    buffer.locate(op(2, 3));

    assert_eq!(buffer.locate(op(1, 2)), a);
    assert_eq!(buffer.len(), 2);
}

#[test]
fn arc_tenant_at_qmax_hit_in_b1() {
    let params = Params {
        num_tenants_n: 1,
        buffer_size_q: 10,
        priorities_lt: vec![1; 1],
        db_size_dt: vec![10; 1],
        buffer_sizes_qt: vec![(1, 1, 1); 1],
    };
    let mut buffer = Buffer::with_params(params);
    let a = buffer.locate(op(1, 1));

    // Move `op(1, 1)` to B1.
    buffer.locate(op(1, 2));

    assert_eq!(buffer.locate(op(1, 1)), a);
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

mod lru_list {
    use crate::lru_list::LruList;

    #[test]
    fn smoke() {
        let mut list = LruList::new();

        list.push_mru(1);
        list.push_mru(2);
        list.push_mru(3);
        list.push_lru(4);
        list.push_lru(5);
        assert_eq!(list.len(), 5);
        assert_eq!(list.peek_mru(), Some(&3));
        assert_eq!(list.peek_lru(), Some(&5));

        assert_eq!(list.pop_mru(), Some(3));
        assert_eq!(list.pop_mru(), Some(2));
        assert_eq!(list.pop_lru(), Some(5));
        assert_eq!(list.pop_lru(), Some(4));
        assert_eq!(list.pop_lru(), Some(1));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_mru(), None);
        assert_eq!(list.pop_lru(), None);
        assert_eq!(list.peek_mru(), None);
        assert_eq!(list.peek_lru(), None);

        let a = list.push_mru(1);
        let b = list.push_mru(2);
        list.push_mru(3);
        assert_eq!(unsafe { list.peek_inside(&a) }, &1);
        assert_eq!(unsafe { list.peek_inside(&b) }, &2);

        assert_eq!(unsafe { list.remove(b) }, 2);
        assert_eq!(unsafe { list.remove(a) }, 1);
        assert_eq!(list.len(), 1);
        assert_eq!(list.peek_mru(), Some(&3));
        assert_eq!(list.peek_lru(), Some(&3));
    }
}
