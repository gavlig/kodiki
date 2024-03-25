use std::ops::Range;

pub fn create_range<T: std::iter::Step>(
	range: Range<T>,
    rev: bool,
) -> itertools::Either<impl Iterator<Item = T>, impl Iterator<Item = T>> {
    if !rev {
        itertools::Either::Left(range)
    } else {
        itertools::Either::Right((range).rev())
    }
}

