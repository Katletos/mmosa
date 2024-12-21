use statrs::{
    distribution::{ChiSquared, ContinuousCDF, Normal},
    statistics::Statistics,
};

#[derive(serde::Serialize)]
pub struct Stats {
    pub mean: f64,
    pub std_dev: f64,
    pub chi_test: Option<ChiTest>,
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

impl Stats {
    pub fn new(data: &[f64], bins: usize, config: &StatsConfig) -> Self {
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

        Self {
            mean,
            std_dev,
            chi_test,
            bins,
        }
    }
}

pub fn chi_test<N: ContinuousCDF<f64, f64>>(
    samples: &[f64],
    bins: usize,
    d: N,
) -> f64 {
    let min_y = *samples.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let max_y = *samples.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

    let batch_value_step = (max_y - min_y) / bins as f64; //that's tricky
    let total_count = samples.len() as f64;

    let mut chi_values = Vec::<f64>::new();

    for i in 0..bins {
        let min_value = min_y + batch_value_step * i as f64;
        let max_value = min_value + batch_value_step;

        let interval_size = samples
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
