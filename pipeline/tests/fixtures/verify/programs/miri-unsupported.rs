// Miri cannot check it: reading the working directory is refused under Miri's
// isolation, which is not UB and not a clean run either.
fn main() {
    let dir = std::env::current_dir().unwrap();
    println!("{}", dir.is_absolute());
}
