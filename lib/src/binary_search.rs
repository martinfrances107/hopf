#[derive(Debug, PartialEq)]
pub(crate) enum Bracket {
    Exact(usize),
    Between(usize, usize),
    OutOfBounds,
}

use std::fmt::Debug;

pub(crate) fn binary_search<T>(arr: &[T], desired: &T) -> Bracket
where
    T: PartialOrd + Debug,
{
    let mut low = 0;
    let mut high = arr.len() - 1;

    if arr[low] > *desired || arr[high] < *desired {
        println!(
            "Out of bounds: desired {:#?}, low {:#?}, high {:#?}",
            *desired, arr[low], arr[high]
        );
        return Bracket::OutOfBounds;
    }

    while low <= high {
        let mid = (low + high) / 2;

        if arr[mid] < *desired {
            low = mid + 1;
        } else if arr[mid] > *desired {
            if mid == 0 {
                break; // Prevent underflow
            }
            high = mid - 1;
        } else {
            return Bracket::Exact(mid);
        }
    }
    Bracket::Between(high, low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_found() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(binary_search(&arr, &3), Bracket::Exact(2));
        assert_eq!(binary_search(&arr, &1), Bracket::Exact(0));
        assert_eq!(binary_search(&arr, &5), Bracket::Exact(4));
    }

    #[test]
    fn test_binary_search_not_found() {
        let arr = [1, 2, 3, 5];
        assert_eq!(binary_search(&arr, &6), Bracket::OutOfBounds);
        assert_eq!(binary_search(&arr, &0), Bracket::OutOfBounds);
    }

    #[test]
    fn test_binary_search_bracketing() {
        let arr = [1, 3, 5, 7, 9];
        assert_eq!(binary_search(&arr, &4), Bracket::Between(1, 2)); // 4 is between 3 and 5
        assert_eq!(binary_search(&arr, &8), Bracket::Between(3, 4)); // 8 is between 7 and 9
    }
}
