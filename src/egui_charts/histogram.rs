pub struct OnlineHistogram {
    pub bins: Vec<f64>,
    pub bin_width: f64,
    pub max: f64,
    pub min: f64,
}

pub fn find_best_bins(y_count: usize) -> usize {
    let count = y_count as f64;

    (3.222 * count.log10().floor() + 1.0) as usize
}

pub fn get_histogram(values: &[f64]) -> OnlineHistogram {
    let bin_count = find_best_bins(values.len()) * 3;

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for item in values {
        if *item < min {
            min = *item;
        }
        if *item > max {
            max = *item;
        }
    }
    let bin_width = (max - min) / bin_count as f64;

    let mut resulting_bins = vec![0.0; bin_count];
    for item in values {
        let index = ((item - min) / bin_width).floor() as usize;
        resulting_bins[index.min(bin_count - 1)] += 1.0;
    }

    let len = values.len() as f64;
    resulting_bins.iter_mut().for_each(|e| *e /= len);

    OnlineHistogram {
        bins: resulting_bins,
        max,
        min,
        bin_width,
    }
}
