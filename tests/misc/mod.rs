#[test]
fn is_deterministic_hasher() {
    use slotted_egraphs::HashSet;

    let s1 = vec![1, 2, 3, 4, 5, 6]
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let s2 = vec![1, 2, 3, 4, 5, 6]
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let s3 = vec![1, 2, 3, 4, 5, 6]
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
    assert_eq!(s2, s3);
}
