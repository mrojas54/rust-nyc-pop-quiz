// AC-11: fails to compile, but a parse error carries no error[E....] code, so
// there is nothing to record. Declared does-not-compile, it is rejected.
fn main() {
    let x = ;
}
