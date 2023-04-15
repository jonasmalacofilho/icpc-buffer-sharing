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
