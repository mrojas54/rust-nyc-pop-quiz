// AC-9: undefined behaviour under both borrow models. `r` is derived from `p`,
// the write through `p` invalidates `r`, and the write through `r` is then UB in
// Stacked Borrows and in Tree Borrows. It prints only before the UB, so Miri's
// output up to the UB can equal the native output exactly.
fn main() {
    let mut x = 0u8;
    let p = &mut x as *mut u8;
    let r = unsafe { &mut *p };
    println!("before");
    unsafe { *p = 1 };
    *r = 2;
}
