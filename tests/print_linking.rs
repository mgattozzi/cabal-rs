extern crate cabal_rs;
use cabal_rs::Cabal;

#[test]
fn build() {
    if let Err(e) = Cabal::src("test_build_pkg").build() {
        panic!("{}", e);
    }
}
