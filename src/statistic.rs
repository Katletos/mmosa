use std::ops::Range;

use statrs::{
    distribution::{ChiSquared, ContinuousCDF, Normal, StudentsT},
    statistics::Statistics,
};

#[derive(serde::Serialize)]
pub struct Stats {
    pub mean: f64,
    pub std_dev: f64,

    pub value_range: Range<f64>,
    pub t_stat: f64,

    pub chi_test: Option<ChiTest>,
    pub ks_test: Option<KsTest>,
    pub bins: usize,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StatsConfig {
    pub alpha: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ChiTest {
    //effective and estimated
    Passed(f64, f64),
    Failed(f64, f64),
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum KsTest {
    Passed(f64, f64),
    Failed(f64, f64),
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum StudentTest {
    Passed(f64, f64),
    Failed(f64, f64),
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum FisherTest {
    Passed(f64, f64),
    Failed(f64, f64),
}

impl Stats {
    pub fn new(data: &[f64], config: &StatsConfig) -> Self {
        let mean = data.mean();
        let std_dev = data.std_dev();
        let bins = data.len() - 1;

        let (value_range, t_stat) = {
            let t_dist = StudentsT::new(0.0, 1.0, bins as f64).unwrap();
            let t_critical = t_dist.inverse_cdf(1.0 - config.alpha);

            let t_margin = t_critical * std_dev / data.len() as f64;

            ((mean - t_margin)..(mean + t_margin), t_margin)
        };

        Self {
            mean,
            std_dev,
            value_range,
            bins,
            t_stat,
            chi_test: None,
            ks_test: None,
        }
    }

    pub fn new_normal(data: &[f64], bins: usize, config: &StatsConfig) -> Self {
        let mean = data.mean();
        let std_dev = data.std_dev();

        let normal_distribution = Normal::new(mean, std_dev).unwrap();

        let chi_test = if bins > 3 {
            let chi_distribution = ChiSquared::new((bins - 3) as f64).unwrap();

            let chi = chi_test(data, bins, normal_distribution);
            let chi_critical = chi_distribution.inverse_cdf(1.0 - config.alpha);

            if chi < chi_critical {
                ChiTest::Passed(chi, chi_critical)
            } else {
                ChiTest::Failed(chi, chi_critical)
            }
            .into()
        } else {
            log::warn!("Not enougth bins ({bins}) to estimate chi^2");
            None
        };

        let ks_test = if data.len() > 40 {
            let ks_critical = (-0.5 * (config.alpha / 2.0).ln()).sqrt();
            let ks = ks_test(data, bins, normal_distribution);

            if ks < ks_critical {
                KsTest::Passed(ks, ks_critical)
            } else {
                KsTest::Failed(ks, ks_critical)
            }
            .into()
        } else {
            log::warn!("Too small data size for KS test ({})", data.len());
            None
        };

        let (value_range, t_stat) = {
            let t_dist = StudentsT::new(0.0, 1.0, bins as f64).unwrap();
            let t_critical = t_dist.inverse_cdf(1.0 - config.alpha / 2.0);

            let t_margin = t_critical * std_dev / data.len() as f64;

            ((mean - t_margin)..(mean + t_margin), t_margin)
        };

        Self {
            t_stat,
            mean,
            std_dev,
            value_range,
            bins,
            chi_test,
            ks_test,
        }
    }
}

pub fn chi_test<N: ContinuousCDF<f64, f64>>(
    data: &[f64],
    bins: usize,
    d: N,
) -> f64 {
    let min_y = *data.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let max_y = *data.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

    let batch_value_step = (max_y - min_y) / bins as f64; //that's tricky
    let total_count = data.len() as f64;

    let mut chi_values = Vec::<f64>::new();

    for i in 0..bins {
        let min_value = min_y + batch_value_step * i as f64;
        let max_value = min_value + batch_value_step;

        let interval_size = data
            .iter()
            .filter(|v| (min_value..max_value).contains(*v))
            .count();

        if interval_size <= 4 {
            log::warn!("Small real batch size for chi test: {interval_size}");
        }

        let effective_prob = interval_size as f64 / total_count;
        let estimated_prob = d.cdf(max_value) - d.cdf(min_value);

        let chi = (estimated_prob - effective_prob).powi(2) / estimated_prob;

        chi_values.push(chi);
    }

    chi_values.into_iter().sum()
}

pub fn ks_test<N: ContinuousCDF<f64, f64>>(
    data: &[f64],
    bins: usize,
    d: N,
) -> f64 {
    let min_y = *data.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let max_y = *data.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

    let batch_value_step = (max_y - min_y) / bins as f64; //that's tricky
    let total_count = data.len() as f64;

    let v = (0..bins)
        .map(|i| {
            let min_value = min_y + batch_value_step * i as f64;
            let max_value = min_value + batch_value_step;

            let interval_size = data
                .iter()
                .filter(|v| (min_value..max_value).contains(*v))
                .count();

            if interval_size <= 4 {
                log::warn!(
                    "Small real batch size for chi test: {interval_size}"
                );
            }

            let effective_prob = interval_size as f64 / total_count;
            let estimated_prob = d.cdf(max_value) - d.cdf(min_value);

            (effective_prob - estimated_prob).abs()
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();

    v * total_count.sqrt()
}

pub fn t_test(data_1: &[f64], data_2: &[f64]) -> StudentTest {
    let n1 = data_1.len() as f64;
    let n2 = data_2.len() as f64;

    let mean1 = data_1.iter().sum::<f64>() / n1;
    let mean2 = data_2.iter().sum::<f64>() / n2;

    let variance1 =
        data_1.iter().map(|&x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let variance2 =
        data_2.iter().map(|&x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);

    let pooled_variance =
        ((n1 - 1.0) * variance1 + (n2 - 1.0) * variance2) / (n1 + n2 - 2.0);
    let t = (mean1 - mean2) / (pooled_variance * (1.0 / n1 + 1.0 / n2)).sqrt();
    let t_critical = t;

    StudentTest::Passed(t, t_critical)
}

pub fn f_test(data_1: &[f64], data_2: &[f64]) -> FisherTest {
    let variance1 = data_1.variance();
    let variance2 = data_2.variance();

    let f = variance1 / variance2;
    let f_critical = f;

    FisherTest::Passed(f, f_critical)
}
