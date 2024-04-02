pub fn get_two_mutable<T>(slice: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    if first < second {
        let (a, b) = slice.split_at_mut(second);
        let f = &mut a[first];
        let s = &mut b[0];
        (f, s)
    } else {
        let (a, b) = slice.split_at_mut(first);
        let f = &mut b[0];
        let  s = &mut a[second];
        (f, s)
    }
}