// AC-10: Miri runs clean but prints something the native binary does not.
fn main() {
    if cfg!(miri) {
        println!("under Miri");
    } else {
        println!("native");
    }
}
