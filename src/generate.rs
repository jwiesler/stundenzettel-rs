use rand::distributions::uniform::{UniformInt, UniformSampler};
use rand::Rng;

pub fn partition_inner<R: Rng>(
    n: u32,
    k: u32,
    max: u32,
    target: &mut Vec<u32>,
    offsets: &mut Vec<u32>,
    r: &mut R,
) -> bool {
    let dist = UniformInt::<u32>::new(0, n);
    offsets.clear();
    offsets.resize_with(k as usize - 1, || dist.sample(r));
    offsets.sort_unstable();

    target.clear();
    target.reserve(k as usize);
    let mut last = 0;
    for &mut i in offsets {
        let p = i - last;

        if p > max {
            return false;
        }
        target.push(p);
        last = i;
    }

    let p = n - last;
    if p > max {
        return false;
    }
    target.push(p);
    true
}

pub fn partition<R: Rng>(n: u32, k: u32, max: u32, r: &mut R) -> Vec<u32> {
    let mut result = Vec::new();
    let mut offsets = Vec::new();
    while !partition_inner(n, k, max, &mut result, &mut offsets, r) {}
    result
}

#[derive(Debug)]
pub struct Time {
    pub from: u32,
    pub to: u32,
}

pub struct Parameters {
    pub hours: u32,
    pub days: u32,
    pub from: u32,
    pub to: u32,
    pub max_per_day: u32,
}

pub fn generate_times<R: Rng>(parameters: Parameters, r: &mut R) -> Vec<Option<Time>> {
    let Parameters {
        hours,
        days,
        from,
        to,
        max_per_day,
    } = parameters;

    let max_total = days * max_per_day;
    assert!(hours <= max_total);
    let distribute_free_time = max_total / 2 <= hours;
    let distribute = if distribute_free_time {
        max_total - hours
    } else {
        hours
    };
    let durations = partition(distribute, days, max_per_day, r);

    durations
        .iter()
        .map(|&duration| {
            if distribute_free_time {
                max_per_day - duration
            } else {
                duration
            }
        })
        .map(|duration| {
            if duration == 0 {
                None
            } else {
                let dist = UniformInt::<u32>::new_inclusive(from, to - duration);
                let from = dist.sample(r);
                let to = from + duration;
                Some(Time { from, to })
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::generate::{generate_times, Parameters};
    use rand::thread_rng;

    #[test]
    fn test_partition() {
        let mut rng = thread_rng();
        let values = generate_times(
            Parameters {
                hours: 40,
                days: 20,
                from: 8,
                to: 20,
                max_per_day: 8,
            },
            &mut rng,
        );
        dbg!(values);
    }
}
