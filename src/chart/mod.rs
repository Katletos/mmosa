use std::ops::Range;

use plotters::prelude::*;

pub struct Histogram<'a> {
    pub y_data: Vec<f32>,
    //the count of batch
    pub bins: usize,
    pub title: &'a str,
}

impl<'a> Histogram<'a> {
    pub fn from_y_data(title: &'a str, data: Vec<f32>) -> Self {
        let y_count = data.len();

        Self {
            y_data: data,
            bins: find_best_bins(y_count),
            title,
        }
    }

    #[allow(unused)]
    pub fn set_bins(&'a mut self, bins: usize) -> &'a Self {
        self.bins = bins;

        self
    }

    pub fn save(&'a self, file_name: &str) -> std::io::Result<()> {
        let plot_samples = prepare_plot_samples(&self.y_data, self.bins);
        let total_count = self.y_data.len();

        let root =
            BitMapBackend::new(file_name, (1024, 1024)).into_drawing_area();

        let max_y = plot_samples
            .iter()
            .map(|(_range, count)| *count)
            .max()
            .unwrap() as f32 / total_count as f32;

        let min_x = plot_samples
            .iter()
            .map(|(range, _count)| range.start)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();

        let max_x = plot_samples
            .iter()
            .map(|(range, _count)| range.end)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(self.title, ("Arial", 50).into_font())
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(min_x..max_x, 0.0f32..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(plot_samples.into_iter().map(|(range, count)| {
                let x0 = range.start;
                let x1 = range.end;
                let y0 = count as f32 / total_count as f32;
                let y1 = 0.0f32;
                Rectangle::new([(x0, y0), (x1, y1)], BLUE.filled())
            }))
            .unwrap()
            .label("Histogram");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        Ok(())
    }
}

fn prepare_plot_samples(
    samples: &[f32],
    batch_count: usize,
) -> Vec<(Range<f32>, usize)> {
    let min_y = *samples.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let max_y = *samples.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
    let batch_value_step = (max_y - min_y) / batch_count as f32;

    let mut values = Vec::<(Range<f32>, usize)>::new();

    for i in 0..batch_count {
        let min_value = min_y + batch_value_step * i as f32;
        let max_value = min_value + batch_value_step;
        let range = min_value..max_value;

        let batch_size = samples.iter().filter(|v| range.contains(*v)).count();

        values.push((range, batch_size));
    }

    values
}

fn find_best_bins(y_count: usize) -> usize {
    let count = y_count as f32;

    (count.log10().floor() + 1.0) as usize
}
