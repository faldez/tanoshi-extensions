use super::*;

#[test]
fn test_get_manga_list() {
    let mangasee = Mangasee::default();

    let res = mangasee.get_manga_list(Param::default());

    assert_eq!(res.data.is_some(), true);
    assert_eq!(res.error.is_none(), true);
}

#[test]
fn test_get_manga() {
    let mangasee = Mangasee::default();

    let res = mangasee.get_manga_info("/manga/Kanojo-mo-Kanojo".to_string());

    assert_eq!(res.data.is_some(), true);
    assert_eq!(res.error.is_none(), true);
}
