use std::iter::Zip;

macro_rules! impl_zip {
    // The macro takes a variable number of arguments, specified by the pattern.
    // $struct_name: The name of the struct to be defined.
    // $zip_t: The type of the inner iterator, typically a chain of `.zip()` calls.
    // $tuple_to_flat: An expression used to map the nested tuple structure to a flat tuple.
    // $($T: ident),*: A variadic list of generic type parameters representing the iterator types.
    ($struct_name: ident, $zip_t: ty, $tuple_to_flat: expr, $($T: ident),*) => {
         // Define a new struct with the name provided in $struct_name.
        pub struct $struct_name<A: Iterator, $($T: Iterator,)*> {
            // The struct contains a single field `inner` which holds the iterator.
            inner: $zip_t,
        }
        // Define a constructor method `new` for the struct.
        // This method takes an iterator for each type parameter and constructs the `inner` iterator.
        // Implement methods for the newly defined struct.
        impl<A: Iterator, $($T: Iterator,)*> $struct_name<A, $($T,)*> {
            #[allow(non_snake_case)]
            pub fn new (A: A, $($T: $T,)*) -> Self {
                Self {
                    // Construct the `inner` iterator by chaining `.zip()` calls on the provided iterators.
                    inner: A$(.zip($T))*
                }
            }
        }

        impl<A: Iterator, $($T: Iterator,)*> Iterator for $struct_name<A, $($T,)*> {
            // Specify the type of elements the iterator will yield.
            // This is a flat tuple containing an item from each of the wrapped iterators.
            type Item = (A::Item, $($T::Item,)*);

            #[inline(always)]
            fn next(&mut self) -> Option<Self::Item> {
                // Call `next` on the inner iterator and use `map` with $tuple_to_flat to transform
                // the nested tuple structure into a flat tuple.
                self.inner.next().map($tuple_to_flat)
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

    };
}

impl_zip! {Zip3Items, Zip<Zip<A, B>, C>, |((a, b), c)| {(a, b, c)}, B, C}
impl_zip! {Zip4Items, Zip<Zip<Zip<A, B>, C>, D>, |(((a, b), c), d)| {(a, b, c, d)}, B, C, D}
impl_zip! {Zip5Items, Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, |((((a, b), c), d), e)| {(a, b, c, d, e)}, B, C, D, E}
impl_zip! {Zip6Items, Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, |(((((a, b), c), d), e), f)| {(a, b, c, d, e, f)}, B, C, D, E, F}
impl_zip! {Zip7Items, Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, |((((((a, b), c), d), e), f), g)| {(a, b, c, d, e, f, g)}, B, C, D, E, F, G}
impl_zip! {Zip8Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, |(((((((a, b), c), d), e), f), g), h)| {(a, b, c, d, e, f, g, h)}, B, C, D, E, F, G, H}
impl_zip! {Zip9Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, |((((((((a, b), c), d), e), f), g), h), i)| {(a, b, c, d, e, f, g, h, i)}, B, C, D, E, F, G, H, I}
impl_zip! {Zip10Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, |(((((((((a, b), c), d), e), f), g), h), i), j)| {(a, b, c, d, e, f, g, h, i ,j)}, B, C, D, E, F, G, H, I, J}
impl_zip! {Zip11Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, |((((((((((a, b), c), d), e), f), g), h), i), j), k)| {(a, b, c, d, e, f, g, h, i ,j, k)}, B, C, D, E, F, G, H, I, J, K}
impl_zip! {Zip12Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, |(((((((((((a, b), c), d), e), f), g), h), i), j), k), l)| {(a, b, c, d, e, f, g, h, i ,j, k, l)}, B, C, D, E, F, G, H, I, J, K, L}
impl_zip! {Zip13Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, |((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m )}, B, C, D, E, F, G, H, I, J, K, L, M}
impl_zip! {Zip14Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, |(((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n )}, B, C, D, E, F, G, H, I, J, K, L, M, N}
impl_zip! {Zip15Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, |((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O}
impl_zip! {Zip16Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, |(((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P}
impl_zip! {Zip17Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, |((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q}
impl_zip! {Zip18Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, |(((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R}
impl_zip! {Zip19Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, |((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S}
impl_zip! {Zip20Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, |(((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T}
impl_zip! {Zip21Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>,  |((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U}
impl_zip! {Zip22Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>, V>, |(((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u), v)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u, v)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V}
impl_zip! {Zip23Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>, V>, W>, |((((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u), v), w)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u, v, w)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W}
impl_zip! {Zip24Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>, V>, W>, X>, |(((((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u), v), w), x)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u, v, w, x)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X}
impl_zip! {Zip25Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>, V>, W>, X>, Y>,|((((((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u), v), w), x), y)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y}
impl_zip! {Zip26Items, Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<Zip<A, B>, C>, D>, E>, F>, G>, H>, I>, J>, K>, L>, M>, N>, O>, P>, Q>, R>, S>, T>, U>, V>, W>, X>, Y>, Z>, |(((((((((((((((((((((((((a, b), c), d), e), f), g), h), i), j), k), l), m), n), o), p), q), r), s), t), u), v), w), x), y), z)| {(a, b, c, d, e, f, g, h, i ,j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z)}, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z}

#[doc(hidden)]
pub struct ChainedIterator<I: Iterator> {
    current_iter: Option<I>,
    iterators: Vec<I>,
}

impl<I: Iterator> ChainedIterator<I> {
    #[doc(hidden)]
    pub fn new(mut iterators: Vec<I>) -> Self {
        let current_iter = iterators.pop();
        Self {
            current_iter,
            iterators,
        }
    }
}

impl<I: Iterator> Iterator for ChainedIterator<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Chain the iterators together.
        // If the end of one iterator is reached go to the next.

        match self.current_iter {
            Some(ref mut iter) => match iter.next() {
                None => {
                    self.current_iter = self.iterators.pop();
                    if let Some(ref mut iter) = self.current_iter {
                        iter.next()
                    } else {
                        None
                    }
                }
                item => item,
            },
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut min = 0;
        let mut max = 0;

        if let Some(current_iter) = &self.current_iter {
            let (i_min, i_max) = current_iter.size_hint();
            min += i_min;
            max += i_max.unwrap();
        }

        for i in self.iterators.iter() {
            let (i_min, i_max) = i.size_hint();
            min += i_min;
            max += i_max.unwrap();
        }
        (min, Some(max))
    }
}