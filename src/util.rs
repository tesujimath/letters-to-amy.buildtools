use std::cmp::Ordering;

/// compare slices element-wise where shorter and otherwise equal means less than
pub fn slice_cmp<T>(this: &[T], other: &[T]) -> Ordering
where
    T: Ord,
{
    use Ordering::*;

    for i in 0..this.len().max(other.len()) {
        match (this.get(i), other.get(i)) {
            (Some(s0), Some(s1)) => {
                let cmp_i = s0.cmp(s1);
                if cmp_i != Equal {
                    return cmp_i;
                }
            }
            (None, Some(_)) => {
                return Less;
            }
            (Some(_), None) => {
                return Greater;
            }
            (None, None) => {
                return Equal;
            }
        }
    }
    Equal
}

mod tests;
