// AC-9: the borrow models disagree. The raw pointer is derived from a reference
// to one element and writes the next one: Stacked Borrows reports UB, Tree
// Borrows does not. Declared UB, it is rejected because the models disagree.
fn main() {
    let mut v = vec![1u8, 2];
    let p = &mut v[0] as *mut u8;
    unsafe { *p.add(1) = 9 };
    println!("{:?}", v);
}
