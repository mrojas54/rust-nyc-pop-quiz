// A panic is an answer (option kind `panic`): the verifier accepts an identical
// non-zero exit across every run and records it.
fn main() {
    let v: Vec<u32> = Vec::new();
    println!("counting");
    let first = v[0];
    println!("{}", first);
}
