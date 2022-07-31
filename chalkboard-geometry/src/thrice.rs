//               Copyright John Nunley, 2022.
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

use core::{iter::FusedIterator, mem};

/// An iterator that produces three items, at most.
#[derive(Debug, Clone)]
pub(crate) enum Thrice<T> {
    Three(T, T, T),
    Two(T, T),
    One(T),
    Empty,
}

impl<T> Thrice<T> {
    pub(crate) fn empty() -> Self {
        Thrice::Empty
    }

    pub(crate) fn one(item: T) -> Self {
        Thrice::One(item)
    }

    pub(crate) fn two(first: T, second: T) -> Self {
        Thrice::Two(first, second)
    }

    pub(crate) fn three(first: T, second: T, third: T) -> Self {
        Thrice::Three(first, second, third)
    }
}

impl<T> Iterator for Thrice<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match mem::replace(self, Self::Empty) {
            Thrice::Three(first, second, third) => {
                *self = Thrice::Two(second, third);
                Some(first)
            }
            Thrice::Two(first, second) => {
                *self = Thrice::One(second);
                Some(first)
            }
            Thrice::One(first) => {
                *self = Thrice::Empty;
                Some(first)
            }
            Thrice::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Thrice::Three(_, _, _) => (3, Some(3)),
            Thrice::Two(_, _) => (2, Some(2)),
            Thrice::One(_) => (1, Some(1)),
            Thrice::Empty => (0, Some(0)),
        }
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<T> FusedIterator for Thrice<T> {}

impl<T> ExactSizeIterator for Thrice<T> {}

impl<T> DoubleEndedIterator for Thrice<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match mem::replace(self, Self::Empty) {
            Thrice::Three(first, second, third) => {
                *self = Thrice::Two(first, second);
                Some(third)
            }
            Thrice::Two(first, second) => {
                *self = Thrice::One(first);
                Some(second)
            }
            Thrice::One(first) => {
                *self = Thrice::Empty;
                Some(first)
            }
            Thrice::Empty => None,
        }
    }
}
