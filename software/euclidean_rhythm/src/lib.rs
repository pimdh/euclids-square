#![cfg_attr(not(std), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

use arrayvec::ArrayVec;

pub fn euclidean_rhythm<const MAXLEN: usize>(hits: usize, steps: usize) -> ArrayVec<u8, MAXLEN> {
    let mut pattern: ArrayVec<u8, MAXLEN> = ArrayVec::new();
    pattern.clear();

    assert!(hits <= steps);
    assert!(steps <= MAXLEN);

    if hits == 0 {
        for _ in 0..steps {
            pattern.push(0);
        }
        return pattern;
    }

    let mut divisor = steps - hits;

    let mut level = 0;
    let mut counts = ArrayVec::<usize, MAXLEN>::new();
    let mut remainders = ArrayVec::<usize, MAXLEN>::new();

    remainders.push(hits);

    // Run the euclid algorithm, store all the intermediate results
    loop {
        counts.push(divisor / remainders[level]);
        let r = remainders[level];
        remainders.push(divisor % r);

        divisor = remainders[level];
        level += 1;

        if remainders[level] <= 1 {
            break;
        }
    }
    counts.push(divisor);

    // Build the pattern
    fn build<const MAXLEN: usize>(
        counts: &[usize],
        pattern: &mut ArrayVec<u8, MAXLEN>,
        remainders: &[usize],
        level: isize,
    ) {
        if level == -1 {
            pattern.push(0);
        } else if level == -2 {
            pattern.push(1);
        } else {
            for _ in 0..counts[level as usize] {
                build(counts, pattern, remainders, level - 1);
            }
            if remainders[level as usize] != 0 {
                build(counts, pattern, remainders, level - 2);
            }
        }
    }

    build(
        &counts,
        &mut pattern,
        &remainders,
        level as isize,
    );

    // Put a 1 on the first step
    let index_first_one = pattern.iter().position(|&x| x == 1).unwrap();
    pattern.rotate_left(index_first_one);
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let data = [
            ((1, 2), vec![1, 0]),
            ((5, 8), vec![1, 0, 1, 1, 0, 1, 1, 0]),
            ((5, 16), vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0])
        ];
        for ((hits, steps), rythm) in &data {
            let res = euclidean_rhythm::<64>(*hits, *steps);
            let res_vec: std::vec::Vec<_> = res.into_iter().collect();
            assert!(res_vec == *rythm);
        }
    }
}