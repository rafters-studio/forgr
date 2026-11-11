use store::forgr::notes;
#[test]
fn notes_from_the_owners_schema() {
    let conn = store::forgr::empty_store();
    let n = notes::factory().id("n1").owner("ana").updated_at("t").kind("memo").insert(&conn).unwrap();
    assert_eq!(n.owner, "ana");
    assert_eq!(notes::mock(4).insert(&conn).unwrap().len(), 4);
    use store::forgr::Override::{Fixed, Pick};
    let m = notes::mock(6).with(notes::Overrides { owner: Fixed("bo".into()), kind: Pick(vec!["todo".into()]), ..Default::default() }).insert(&conn).unwrap();
    assert!(m.iter().all(|r| r.owner == "bo" && r.kind == "todo"));
}
