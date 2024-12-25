use nalgebra::{DMatrix, DVector};
use plotters::prelude::*;

use crate::{statistic::StatsConfig, Stats};

pub struct Linear<'a> {
    pub title: &'a str,
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,

    pub use_approximation: bool,
    pub config: Option<&'a StatsConfig>,
}

impl<'a> Linear<'a> {
    pub fn from_data(title: &'a str, x: Vec<f32>, y: Vec<f32>) -> Self {
        assert!(x.len() == y.len());

        let y_data = y.into_iter().map(|v| v as f64).collect::<Vec<f64>>();
        let x_data = x.into_iter().map(|v| v as f64).collect::<Vec<f64>>();

        Self {
            title,
            x_data,
            y_data,
            use_approximation: false,
            config: None,
        }
    }

    pub fn use_approximation(&mut self, use_it: bool) -> &mut Self {
        self.use_approximation = use_it;

        self
    }

    pub fn set_config(&mut self, config: &'a StatsConfig) -> &mut Self {
        self.config = Some(config);

        self
    }

    pub fn save(&'a self, file_name: &str) -> std::io::Result<()> {
        let chart_name = format!("{file_name}.png");

        let min_x = self.x_data.iter().fold(f64::MAX, |a, b| a.min(*b));
        let max_x = self.x_data.iter().fold(f64::MIN, |a, b| a.max(*b));

        let min_y = self.y_data.iter().fold(f64::MAX, |a, b| a.min(*b));
        let max_y = self.y_data.iter().fold(f64::MIN, |a, b| a.max(*b));

        let root =
            BitMapBackend::new(&chart_name, (1024, 1024)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(self.title, ("Arial", 50).into_font())
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(min_x..max_x, min_y..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(
                LineSeries::new(
                    self.x_data
                        .iter()
                        .zip(self.y_data.iter())
                        .map(|(x, y)| (*x, *y)),
                    BLUE,
                )
                .point_size(4),
            )
            .unwrap()
            .label("Data");

        if let Some(config) = self.config {
            let stats = Stats::new(&self.y_data, config);

            chart
                .draw_series(
                    LineSeries::new(
                        self.x_data
                            .iter()
                            .map(|x| (*x, stats.value_range.start)),
                        GREEN,
                    )
                    .point_size(4),
                )
                .unwrap()
                .label(format!("Low = {}", stats.value_range.start));

            chart
                .draw_series(
                    LineSeries::new(
                        self.x_data.iter().map(|x| (*x, stats.value_range.end)),
                        GREEN,
                    )
                    .point_size(4),
                )
                .unwrap()
                .label(format!("High = {}", stats.value_range.end));

            chart
                .draw_series(
                    LineSeries::new(
                        self.x_data.iter().map(|x| (*x, stats.mean)),
                        RED,
                    )
                    .point_size(4),
                )
                .unwrap()
                .label(format!("Mean = {}", stats.mean));
        }

        if self.use_approximation {
            let y_data_apprx = find_approximation(&self.x_data, &self.y_data);

            chart
                .draw_series(LineSeries::new(
                    self.x_data
                        .iter()
                        .zip(y_data_apprx.iter())
                        .map(|(x, y)| (*x, *y)),
                    RED,
                ))
                .unwrap()
                .label("Approximated solution");
        }

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        Ok(())
    }
}
fn find_approximation(x: &[f64], y: &[f64]) -> Vec<f64> {
    let x_matrix = DMatrix::from_columns(&[
        DVector::from_element(x.len(), 1.0),
        DVector::from_row_slice(x),
    ]);
    let y = DVector::from_row_slice(y);

    let eps = 1e-14;
    let results =
        lstsq::lstsq(&x_matrix, &y, eps).expect("Failed to solve lst sqt");

    let y_pred = x_matrix * &results.solution;

    y_pred.as_slice().to_vec()
}
