use nalgebra::{DMatrix, DVector};
use plotters::prelude::*;

pub struct Linear<'a> {
    pub title: &'a str,
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,

    pub y_data_appx: Vec<f64>,
}

impl<'a> Linear<'a> {
    pub fn from_data(title: &'a str, x: Vec<f32>, y: Vec<f32>) -> Self {
        assert!(x.len() == y.len());

        let y_data = y.into_iter().map(|v| v as f64).collect::<Vec<f64>>();
        let x_data = x.into_iter().map(|v| v as f64).collect::<Vec<f64>>();

        Self {
            title,
            y_data_appx: find_approximation(&x_data, &y_data),
            x_data,
            y_data,
        }
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

        chart
            .draw_series(
                LineSeries::new(
                    self.x_data
                        .iter()
                        .zip(self.y_data_appx.iter())
                        .map(|(x, y)| (*x, *y)),
                    RED,
                )
                .point_size(5),
            )
            .unwrap()
            .label("Approximated solution");

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
