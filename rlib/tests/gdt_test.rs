use rlib::gdt::{GdtBuilder, Mode};

#[test]
pub fn test() {
   let mut bd =  GdtBuilder::default();
    bd.limit(0xffffffff).present(true).rw(false).executable(true)
        .mode(Mode::Protect).privilege(0)
        .lim_4k(true)
        .system(false);

    let r = bd.build();

    assert_eq!(r, 58432445946593279);
}